# Platform catalog · Windows

Crate: `sense-platform-windows`. Bindings: the `windows` crate (Win32 and WinRT). Minimum target:
Windows 10 22H2.

Windows is the most open platform: almost everything is readable without a prompt. Two limits
shape the code. A normal process cannot read or receive events from an **elevated** window (UIPI):
report `Unavailable(PermissionDenied)` for those targets. Some WinRT APIs require **package
identity**: the installer must give `sensed` one (a sparse package is enough) for the sources
marked below.

## Sources

| Source id | Tier | Signals | Native interface | Needs |
|---|---|---|---|---|
| `windows.shell` | A | apps, windows, `focus.app`, `focus.window`, titles, bounds, virtual desktop, `app.*`, `window.*` | `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND`, `EVENT_OBJECT_NAMECHANGE`, `EVENT_OBJECT_CREATE/DESTROY` for top-level windows), `GetForegroundWindow`, `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`, app user model id from the window's property store, `IVirtualDesktopManager` | nothing |
| `windows.uia` | A + B + C | `focus.element`, `selection`, `document` where exposed; reads: Selection, Element, Surrounding, Visible, Structure | UI Automation: focus-changed handler, `TextPattern`/`TextPattern2` (`GetSelection`, caret range, `ExpandToEnclosingUnit`), `ValuePattern`, text-selection-changed event | nothing |
| `windows.clipboard` | A | `clipboard`, `clipboard.changed` | `AddClipboardFormatListener` on a message-only window, `GetClipboardOwner`, format enumeration | nothing |
| `windows.explorer` | B | folder shown and items selected in File Explorer, recent documents | `IShellWindows` → `IFolderView`; the Recent items folder | nothing |
| `windows.fs` | A | `file.touched` in configured folders | `ReadDirectoryChangesW` | nothing |
| `windows.media` | A | `media` | WinRT `GlobalSystemMediaTransportControlsSessionManager` | nothing |
| `windows.notifications` | A | `notification.posted` | WinRT `UserNotificationListener` | package identity + user consent |
| `windows.calendar` | A | `calendar` | WinRT `AppointmentStore` (read-only, all calendars) | package identity + consent |
| `windows.activity` | A | `idle_ms`, lock, sleep | `GetLastInputInfo`, `WTSRegisterSessionNotification`, power setting notifications | nothing |
| `windows.av` | A | `in_call` | capability access history under `HKCU\…\CapabilityAccessManager\ConsentStore\{microphone,webcam}` (in use when `LastUsedTimeStop` is 0), watched with `RegNotifyChangeKeyValue` | nothing |
| `windows.system` | A | `session.*` | locale and input language APIs, `EnumDisplayMonitors`, `GetSystemPowerStatus`, Network List Manager, the apps-use-light-theme setting | nothing |
| `windows.screen` | C | reads: Pixels, Ocr | `Windows.Graphics.Capture` (one window), `Windows.Media.Ocr` | nothing (a capture border is shown on older builds) |

## Capture-excluded windows

`windows.shell` calls `GetWindowDisplayAffinity` for each top-level window. Any value other than
`WDA_NONE` means the application refuses capture: report the window with `excluded: true`;
`windows.uia` and `windows.screen` must not read it.

## `windows.uia` in detail

- UI Automation is COM. The source owns a thread initialized as a multithreaded apartment; event
  handlers arrive on COM threads and must only forward into the sink.
- Register **one** focus-changed handler globally. Register text-selection-changed handlers only on
  the focused element, and remove them on focus change. Global structure-changed subscriptions are
  forbidden: they are expensive for every application.
- Use a `CacheRequest` to fetch role, name, `IsPassword`, `IsKeyboardFocusable` and patterns in one
  cross-process call.
- Chromium, Electron and Firefox enable their accessibility tree when a UI Automation client
  queries them. That is automatic here, and it costs those apps some performance; prefer the
  extension sources for them (`prefers` in `app-profiles`) and query them through UIA only on demand.
- `IsPassword` → set `secure`, read nothing.
- Surrounding text: from the selection range, clone and `MoveEndpointByUnit` by characters.
- Document path is rarely exposed. Do not parse it out of window titles in the source; if a
  heuristic is ever wanted it belongs in the core, marked `Inferred`.

## Clipboard etiquette

Honour the registered formats `ExcludeClipboardContentFromMonitorProcessing`,
`CanIncludeInClipboardHistory` and `CanUploadToCloudClipboard`: when present, emit the event with
`concealed: true` and no content.

## Forbidden here

Low-level keyboard hooks (`WH_KEYBOARD_LL`), raw input for keys, DLL injection, reading other
processes' memory.
