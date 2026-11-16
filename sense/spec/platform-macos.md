# Platform catalog · macOS

Crate: `sense-platform-macos`. Bindings: `objc2` and its framework crates, `core-foundation`,
`accessibility-sys` (or direct FFI). Minimum target: macOS 13.

macOS is the richest platform: one accessibility API covers nearly every application, and many
applications are scriptable. The price is the permission system (TCC).

## Permissions: read this first

- TCC grants belong to the **code identity of the responsible process**. `sensed` is the only
  binary that makes these calls. A grant survives updates only if the signing identity is stable.
- **Launch model.** Apple's developer support reports that when an application starts a child
  with an ordinary process-launch API, the system follows responsibility from the child to the
  parent and finds the parent's grant. Reported failures involve a helper packaged as a *nested
  application bundle* with its own identity. So: ship `sensed` as a plain executable inside the
  client application's bundle (`Contents/MacOS/` or `Contents/Helpers/`, not a nested `.app`),
  signed with the same identity, and have the client launch it directly. The user then sees one
  grant, under the application's name.
- A daemon that must outlive the client (started at login) would be registered through
  `SMAppService`. How TCC attributes that case is not confirmed; do not build on it before testing
  (`14-open-questions.md`).
- In development, an unsigned or ad-hoc-signed build gets a new identity at each build and loses
  its grants. Sign development builds with one fixed self-signed certificate. When launched from a
  terminal, the terminal may become the responsible process and receive the prompt instead.
- A revoked or missing grant must produce `Unavailable(PermissionDenied)`, never a hang.
  `AXIsProcessTrusted()` is the check; it does not prompt.

## Sources

| Source id | Tier | Signals | Native interface | Permission |
|---|---|---|---|---|
| `macos.workspace` | A | apps, `focus.app`, `app.launched/quit`, active Space change | `NSWorkspace` notifications, `NSRunningApplication` | none |
| `macos.windows` | A | window list, bounds, display, layer | `CGWindowListCopyWindowInfo` (no titles without Screen Recording; titles come from `macos.ax`) | none |
| `macos.ax` | A + B + C | `focus.window`, `focus.element`, window titles, `selection`, `document.path`, `document.url`; reads: Selection, Element, Surrounding, Visible, Structure | `AXUIElement`, `AXObserver` | Accessibility |
| `macos.pasteboard` | A | `clipboard`, `clipboard.changed` | `NSPasteboard.general` `changeCount` (poll, 500 ms, paused when idle) | none; recent macOS versions may alert the user on programmatic reads of content |
| `macos.scripting` | B | `document.url`, tabs, selected items in scriptable apps (Safari, Chrome, Mail, Finder, Music) | Apple Events via `NSAppleScript` / `OSAScript` | Automation, per target app |
| `macos.eventkit` | A | `calendar` | `EKEventStore` | Calendars |
| `macos.spotlight` | A | recent documents, `file.touched` | `NSMetadataQuery` (`kMDItemLastUsedDate`) | none for user folders |
| `macos.fsevents` | A | `file.touched` in configured folders | FSEvents | Files and Folders for protected locations |
| `macos.activity` | A | `idle_ms`, lock, sleep | `CGEventSourceSecondsSinceLastEventType`, distributed notifications for lock, `NSWorkspace` sleep notifications | none |
| `macos.av` | A | `in_call` | CoreAudio `kAudioDevicePropertyDeviceIsRunningSomewhere`, CoreMediaIO equivalent for cameras | none |
| `macos.system` | A | `session.*` | `NSLocale`, Text Input Sources, `NSScreen`, IOKit power sources, `NWPathMonitor`, effective appearance | none |
| `macos.screen` | C | reads: Pixels, Ocr | ScreenCaptureKit (one window or region), Vision `VNRecognizeTextRequest` | Screen Recording |

Not available through public interfaces, and therefore not provided: system-wide now-playing
(the MediaRemote framework is private and restricted; use `macos.scripting` for Music and Spotify),
other applications' notifications, Focus mode details without an entitlement.

## Capture-excluded windows

`macos.windows` reads each window's sharing state from the window list (`kCGWindowSharingState`).
A window whose sharing state is "none" is reported with `excluded: true`; `macos.ax` and
`macos.screen` must not read it.

## `macos.ax` in detail

This source carries most of the value on macOS.

- **Watching.** One `AXObserver` per running application, created on `app.launched` and for the
  frontmost app first. Notifications: `AXFocusedWindowChanged`, `AXFocusedUIElementChanged`,
  `AXSelectedTextChanged`, `AXTitleChanged`, `AXValueChanged` (focused element only). Observers
  need a run loop: the source owns a dedicated thread with a `CFRunLoop`.
- **Reading the focused element.** System-wide element → `AXFocusedUIElement`, then `AXRole`,
  `AXSubrole`, `AXTitle`/`AXDescription`, `AXValue` (length only in tier A), `AXSelectedText`,
  `AXSelectedTextRange`, `AXNumberOfCharacters`.
- **Surrounding text.** Parameterized attribute `AXStringForRange` around the selected range.
- **Document.** `AXDocument` on the focused window gives a file URL in document-based apps.
  In browsers, `AXURL` on the web area gives the page URL.
- **Visible and Structure reads.** Walk the focused window's subtree breadth-first with a node
  budget and the deadline; collect `AXStaticText`, headings, links, text fields.
- **Waking Chromium and Electron.** These build their tree lazily. Set `AXManualAccessibility`
  (Electron) or `AXEnhancedUserInterface` (Chrome) on the application element when the profile
  says `wake_accessibility`. Do this only for the app in focus, and only when no extension source
  already covers it, because it costs that app performance.
- **Secure fields.** Subrole `AXSecureTextField` → set `secure`, read nothing.
- **Timeouts.** `AXUIElementSetMessagingTimeout` on every element used (default 100 ms). A hung
  app must not stall the source.

## Selection fallback

When an app exposes no `AXSelectedText`, the exact selection can only be obtained by simulating
Copy and restoring the pasteboard. That is an action with side effects, so it is **not** a Sense
source. It is the effect `act.selection.copy` of the acting service (`12-acting.md`), used inside
a capture only when the client asks for it; otherwise `selection.text` is
`Unavailable(AppNotAccessible)`.

## Forbidden here

`CGEventTap` and `NSEvent` global monitors for key events; private frameworks; reading other apps'
containers.
