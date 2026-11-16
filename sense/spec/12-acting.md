# 12 · Acting: hotkey, menu placement, inserting text

Sense reads. A product like Bloom must also **act**: react to a global hotkey, put a menu where the
pointer is, give focus back, and insert text into the application the user was in. This file
specifies that service. Decision record: ADR 0018.

## Why it lives in the same daemon, and how it stays separate

Acting needs exactly what reading needs: the Accessibility grant on macOS, the connection to the
GNOME Shell extension or the KWin script, the Wayland protocols. A second process would need a
second grant, a second extension connection, and its own copy of the per-session logic. So the
acting service ("Hands") is a module of `sensed`.

It is kept apart by construction:

- its own crate (`hands-core`, plus the platform crates implementing its trait) and its own API
  namespace, `act.*`;
- its own scope, `act`, granted only to first-party binaries listed at install time. It cannot be
  granted through the grants file to an unknown executable, and **it is not exposed by the MCP
  server**;
- a **closed list of effects**. There is no method to press an arbitrary key, click at a
  coordinate, or run a command, and none will be added;
- every effect is written to the access ledger and signalled by the indicator.

Sense's read-only guarantees (`README`, invariants) are unchanged: nothing in the `sense-*` crates
acts.

## The API

| Method | Effect |
|---|---|
| `act.hotkey.bind { id, accelerator }` → events `act.hotkey.pressed { id, at }` | Register a global hotkey. `at` is stamped as close to the key event as the platform allows. |
| `act.hotkey.unbind { id }` | |
| `act.window.place { window: own, at: Pointer \| Center, size }` | Move the client's own window to the pointer (or screen centre) and raise it. Only windows of the calling client's `own_pids`. |
| `act.focus.restore { target }` | Give keyboard focus back to the window identified by a snapshot's `target`. |
| `act.insert_text { target, text, mode: Replace \| After }` | Insert text into the target, replacing the selection or after it. |
| `act.selection.copy { target }` → `{ text }` | Read the selection of the target by sending it the copy shortcut and reading the clipboard, then restoring the clipboard. The exact-selection fallback for applications that expose it no other way. |
| `act.clipboard.set { text }` | Put text on the clipboard, marked so clipboard managers and history skip it where the platform allows. |
| `act.capabilities` | Which of the above work in this session, and how (see fallbacks). |

Nothing else. Opening a URL or a file, moving a file to the trash, writing a note, creating a
calendar entry: these need no privileged access and are done by the client with ordinary library
calls.

### Rules for every effect

1. **Target check.** `insert_text` and `focus.restore` carry a `target` from a snapshot. The
   service verifies that the window still exists and still belongs to the same application before
   acting; otherwise `unknown_target`. Text is never typed into "whatever is focused now".
2. **No effect on excluded or secure targets.** An excluded window or a secure field refuses
   `insert_text`.
3. **The service does not decide.** Whether an insertion needs the user's approval is the
   client's policy (Bloom's criticality floor and approval gate). The service executes what a
   first-party client asks, once, and reports precisely what happened.
4. **Clipboard hygiene.** When insertion goes through the clipboard: save the current content,
   set the new one with the platform's "transient / do not record" marker, paste, restore the
   saved content after the target has consumed the paste (default 300 ms), and report
   `via: "paste"`. If the restore fails, say so in the result.
5. **One at a time.** Effects are serialized per target. A hotkey press while an insertion is in
   flight is delivered after it.

### Result

```json
{ "ok": true, "via": "accessibility" | "paste" | "portal", "elapsed_ms": 41,
  "clipboard_restored": true, "warning": null }
```

## Reading the selection by copying

Some applications expose their selection through no readable channel: no accessibility, no
extension, and on GNOME 50 and later no PRIMARY selection either (primary paste is off by
default there). The only universal way left is the one the user would use: copy.

`act.selection.copy` saves the clipboard, sends the copy shortcut to the target, waits for the
clipboard to change (up to 150 ms), reads it, and restores the saved content. It is an effect, not
a source, because it sends a key and touches the clipboard:

- it runs only inside a user-initiated capture, **before** the client shows any window, while the
  target still has focus; the engine calls it when the client's `capture` asks for
  `selection_fallback: "copy"` and every non-intrusive channel came back unavailable;
- it needs the `act` scope, so third-party assistants never trigger it;
- **it is never sent to an application whose copy shortcut is not known to be safe.** Ctrl+C in a
  terminal interrupts the running program. `app-profiles` carries `copy_shortcut` per application;
  the `terminal` category defaults to Ctrl+Shift+C, and an application with no profile and an
  unknown category gets the platform default only if its focused element is a text role;
- if the clipboard does not change in time, the result is "nothing selected or not copyable", the
  clipboard is left untouched, and the snapshot says `selection: unavailable (app_not_accessible)`;
- the result has quality `Exact` and `via: "copy"`.

Cost: 30 to 150 ms and a visible flicker in clipboard managers that ignore the transient marker.
That is why it is last in the ladder and optional per client.

## Inserting text: the ladder

Always the least intrusive mechanism that works.

| Step | Mechanism | Used when |
|---|---|---|
| 1 | **Application extension**: the browser or editor extension inserts into its own focused field or buffer | the target belongs to an application with a connected extension that offers insertion |
| 2 | **Accessibility**: set the selected text of the focused element | the element is editable and the attribute is settable |
| 3 | **Paste**: clipboard, synthesized paste shortcut, clipboard restore | everything else |

Per platform:

| | Step 2 | Step 3 |
|---|---|---|
| macOS | set `AXSelectedText` on the focused element when `AXUIElementIsAttributeSettable` says so | `NSPasteboard` with the transient type marker, then ⌘V as a `CGEvent` posted to the target's process |
| Windows | rarely possible: UI Automation's value pattern replaces a whole field and its text pattern is read-only. Use only for single-line fields with `Replace` of the entire value | clipboard with the exclude-from-history formats, then Ctrl+V through `SendInput`. Fails on elevated windows (report it) |
| Linux, any family | AT-SPI `EditableText` (`InsertText`, `DeleteText`) where the application answers | per family, below |
| X11 | | own the CLIPBOARD selection, send the paste shortcut with XTest |
| GNOME | | in the Shell extension: `St.Clipboard.set_text`, activate the target window, send the shortcut through a Clutter virtual keyboard device (the mechanism Kando's extension uses), restore |
| KDE | | clipboard through the data-control protocol; keys through the RemoteDesktop portal (a consent dialog once, remembered with a restore token) |
| wlroots | | clipboard through data-control; keys through `virtual-keyboard-unstable-v1` |

The paste shortcut is not universal: terminals use Ctrl+Shift+V. `app-profiles` gets a
`paste_shortcut` field per application; the default is the platform's standard one.

Synthesized key events use physical key codes for the active layout. The service sends only the
paste shortcut, never free text as key presses: typing text key by key is slow, layout-dependent
and breaks on non-Latin input.

## Global hotkey

| Platform | Mechanism |
|---|---|
| macOS | Carbon hot-key registration. No permission. |
| Windows | `RegisterHotKey`. |
| X11 | key grab on the root window. |
| GNOME | the GlobalShortcuts portal on GNOME 48 and later (the desktop asks the user to confirm the binding, and may change the key); on 46 and 47, the Shell extension grabs the accelerator (`global.display.grab_accelerator`). Block D's prototype already does this on GNOME 46, with a sound ownership rule worth keeping: one owner at a time, released when the owner's connection disappears. |
| KDE, Hyprland | the GlobalShortcuts portal. |
| Other wlroots compositors | no portal on some: fall back to the universal method below. |
| **Any desktop, universal fallback** | the user binds a key in their desktop's settings to the command `sense trigger <id>`, which delivers `act.hotkey.pressed` to the daemon over the socket. Works everywhere, costs one manual step, and is what onboarding offers when nothing else is available. |

On portal-based sessions the accelerator requested is a preference: the user has the last word.
Show the effective binding in settings.

The extensions site forbids shipping default keyboard shortcuts that act on clipboard data. Ours
opens the client's menu and is bound at run time at the client's request, not shipped as a default
in the extension's schema.

## Placing the menu and keeping focus

The ideal menu never takes keyboard focus from the application. Then the selection stays alive,
insertion needs no focus hand-back, and the whole sequence is faster.

| Platform | Menu window | Focus |
|---|---|---|
| macOS | a non-activating panel (`NSPanel`, non-activating style) at the pointer | the application keeps focus; `focus.restore` is a no-op |
| Windows | a top-most tool window with the no-activate extended style | same |
| X11 | an override-redirect or utility window positioned by the client itself; the pointer from the snapshot | may take focus; `focus.restore` uses `_NET_ACTIVE_WINDOW` |
| GNOME | A native Wayland client cannot position its own window. **Proven route** (block D's prototype, branch `feature/tauri-ui`, GNOME 46): run the menu window through XWayland (`GDK_BACKEND=x11`), which may position itself, and take the pointer position from the Shell extension, mapped from stage to X11 root coordinates. Fallback for a native Wayland window: `act.window.place` asks the extension to move and raise it. | the window takes focus; `focus.restore` activates the target window through the extension before insertion |
| KDE, wlroots | a layer-shell surface on the overlay layer, covering the output, drawing the menu at the pointer position received on entry; or `act.window.place` through the KWin script | keyboard interactivity on demand; released on close, focus returns to the previous window |

`act.capabilities` reports, for the session: `menu_placement: pointer | center`,
`menu_steals_focus: bool`, `hotkey: native | portal | manual`, `insert: [accessibility, paste, …]`.
The client designs its degraded modes from that one answer (centre of the screen when the pointer
is unknown; keyboard-only navigation when it cannot track the pointer).

## The sequence for a menu product

```
key press ──▶ act.hotkey.pressed {at}
client    ──▶ capture {deadline 50 ms}            → snapshot + target      (reading, Sense)
client    ──▶ act.window.place {at: Pointer}      → menu visible
user picks an action; client resolves it, asks for approval if its policy says so
client    ──▶ content.read … (if the action needs depth)
client closes its menu
client    ──▶ act.insert_text {target, text}      → focus restored if needed, text inserted
```

The capture happens before the menu is shown, and the engine ignores the client's own windows, so
the order is safe on every platform.

## Safety notes

- This service can put text into other applications. Its whole protection is the short list of
  effects, the `act` scope restricted to first-party binaries, the target check, and the ledger.
  Reviewers should treat any proposal to add a general "press keys" or "click" effect as a
  different product with a different threat model.
- On GNOME the key event is injected from inside the compositor. The extension accepts the
  request only over its connection to the daemon, only for the paste shortcut of the active
  profile, and only immediately after activating the verified target window.
- Text to insert may have been written by a model from untrusted content. What gets inserted is
  the client's responsibility; the service inserts exactly the bytes it was given, after stripping
  the same invisible characters as on the reading side.

## Testing

Conformance scenarios, same fixture applications as `05-testing.md`:

- A1 · bind a hotkey, press it → event within 30 ms, timestamp before the event's delivery.
- A2 · `insert_text` into the fixture's text field with a selection → the field holds the new text;
  clipboard content before and after is identical.
- A3 · same into a field that rejects accessibility insertion → falls to paste, `via: "paste"`.
- A4 · close the target window before inserting → `unknown_target`, nothing typed anywhere.
- A5 · target is the password field or an excluded window → refused.
- A6 · `window.place` at the pointer → window within 8 px of the pointer on sessions that report
  `menu_placement: pointer`.
- A8 · `selection.copy` on the fixture field with a selection → the selected text; clipboard
  identical before and after. On the fixture terminal → the profile's shortcut is used and the
  running program is not interrupted. With nothing selected → unavailable, clipboard untouched.
- A7 · a client without the `act` scope calls any `act.*` method → `scope_denied`; the MCP server
  lists no acting tool.
