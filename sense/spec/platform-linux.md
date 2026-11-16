# Platform catalog · Linux

Crate: `sense-platform-linux`. Bindings: `zbus` (D-Bus), `x11rb` (X11), `wayland-client` and
`wayland-protocols` (Wayland), `atspi` or direct D-Bus for accessibility.

## The model: session families, detected at run time

Linux is not one platform. What decides which code works is the **session**: the display server
and compositor the user logged into. The distribution (Fedora, Ubuntu, Debian, Arch) only decides
packaging and which version of a desktop is present. One binary must work on all of them.

On X11, any client may read the active window, the selection and the pointer. Wayland removed
those abilities from ordinary clients by design and left each compositor to decide what to expose.
So there are four families:

| Family | Detect by | Typical desktops |
|---|---|---|
| `x11` | `XDG_SESSION_TYPE=x11`, a reachable X display, no Wayland compositor | Xfce, Cinnamon, MATE, i3, older installs |
| `gnome` | Wayland session and the name `org.gnome.Shell` on the session bus | GNOME (default on Fedora, Ubuntu, Debian) |
| `kde` | Wayland session and the name `org.kde.KWin` | KDE Plasma |
| `wlroots` | Wayland session exposing `ext-foreign-toplevel-list` or `wlr-foreign-toplevel-management` | Sway, Hyprland, niri, river, labwc, COSMIC (partly) |

`SessionEnv` holds: session type, `XDG_CURRENT_DESKTOP`, desktop version, the session bus names
present, the Wayland globals advertised, and whether the process is sandboxed. Sources `probe`
against it. Never branch on distribution name.

A session that matches no family still gets every family-independent source below; window and
focus signals come back `Unavailable(NoSource)` unless AT-SPI can supply them.

Packaging note: Sense must run **unsandboxed** (native package, or a tarball/AppImage). A Flatpak
or Snap sandbox hides AT-SPI, other processes and most of the session bus.

Linux has no cross-desktop flag by which a window refuses capture. Exclusion here relies on
`app-profiles`, the user's own list, private-window detection and provider hints.

## Family-independent sources

| Source id | Tier | Signals | Interface | Notes |
|---|---|---|---|---|
| `linux.atspi` | A + B + C | `focus.element`, `focus.window` (accessible apps), `selection`, `document.url` in some apps; reads: Selection, Element, Surrounding, Visible, Structure | AT-SPI2 over D-Bus (the accessibility bus from `org.a11y.Bus`): events `window:activate`, `object:state-changed:focused`, `object:text-selection-changed`, `object:text-caret-moved`; interfaces `Text`, `Document`, `Component` | Coverage depends on the application's toolkit; see below |
| `linux.mpris` | A | `media` | `org.mpris.MediaPlayer2.*` on the session bus | Works everywhere |
| `linux.notifications` | A | `notification.posted` | Become a bus monitor (`org.freedesktop.DBus.Monitoring.BecomeMonitor`) matching `org.freedesktop.Notifications.Notify` | Needs an unsandboxed session bus connection |
| `linux.login1` | A | lock, sleep, `idle` hint | `org.freedesktop.login1` (session `LockedHint`, `IdleHint`, `PrepareForSleep`) | |
| `linux.idle` | A | `idle_ms` | Wayland `ext-idle-notify-v1`; on GNOME `org.gnome.Mutter.IdleMonitor`; on X11 the screen saver extension | pick by probe |
| `linux.recent` | A | recent documents, `file.touched` | `~/.local/share/recently-used.xbel` (written by GTK and Qt/KDE apps), watched with inotify | |
| `linux.fs` | A | `file.touched` in configured folders | inotify | |
| `linux.proc` | B | `terminal.cwd`, `terminal.foreground_command`, `apps[].exe` | `/proc/<pid>/{exe,cwd,cmdline,stat}`: from the terminal's pid, follow the foreground process group of its pty | Same-user processes only; `Inferred` quality. `ext.shell` is preferred |
| `linux.pipewire` | A | `in_call` | PipeWire registry: an active stream linked to a microphone or camera node | |
| `linux.calendar` | A | `calendar` | Evolution Data Server over D-Bus (GNOME and others); Akonadi on KDE later | optional |
| `linux.system` | A | `session.*` | locale env, `org.freedesktop.portal.Settings` (colour scheme), UPower, NetworkManager, XKB layout | |
| `linux.portal.screen` | C | reads: Pixels, Ocr | `org.freedesktop.portal.ScreenCast` with a restore token, frames over PipeWire; OCR with a bundled engine | Consent dialog on first use; the fallback on every Wayland family |

### `linux.atspi` in detail

- AT-SPI is the same on every desktop, Wayland included. It does not give screen coordinates on
  Wayland, and it does not see applications that have not built their tree.
- **Coverage by toolkit.** GTK applications answer. Qt applications answer when accessibility is
  active. Chromium, Electron and Firefox build their tree only when they believe an assistive tool
  is present (the `org.a11y.Status` `IsEnabled` / `ScreenReaderEnabled` properties, or per-app
  flags such as `--force-renderer-accessibility`).
- **Do not set `IsEnabled` globally by default.** It makes every Chromium and Electron application
  build and maintain a full tree, all the time. Offer it as an explicit user option
  ("deep accessibility"), off by default. Browser and editor context comes from extension sources.
- Subscribe to events for the focused application only (register and deregister on focus change);
  a global `object:` subscription floods the bus.
- Role `password text` → set `secure`, read nothing.
- Every D-Bus call carries a timeout (100 ms default).

## Family: `x11`

One source, `linux.x11`, covers everything. It is the simplest provider and the only one that runs
headless (under a virtual X server), which makes it the reference for automated conformance tests.

| Signal | Interface |
|---|---|
| windows, focus, titles, app ids, workspace | EWMH on the root window: `_NET_CLIENT_LIST_STACKING`, `_NET_ACTIVE_WINDOW`, `_NET_WM_NAME`, `WM_CLASS`, `_NET_WM_PID`, `_NET_WM_DESKTOP`, `_NET_WM_STATE`; `PropertyNotify` for changes |
| `selection` (PRIMARY) and `clipboard` | XFixes selection-owner notifications, then a selection conversion. The owner window is known, so `selection.owner_window` is exact and stale selections can be detected |
| `pointer` | pointer query |
| `idle_ms` | screen saver extension |
| reads: Pixels | composite extension / `GetImage` on the window |

`raw_id` is the `WM_CLASS` class; map to a desktop file id through `StartupWMClass` when unique.

## Family: `gnome`

Mutter exposes nothing about other applications' windows to clients, and offers no data-control
protocol. The window signals therefore require **our GNOME Shell extension** (`ext/gnome-shell`),
which runs inside the compositor and connects out to the daemon as the source `linux.gnome`.

| Signal | Inside the extension |
|---|---|
| windows, focus, titles, app ids, workspace, fullscreen | `global.display` (`notify::focus-window`, `window-created`), `Meta.Window` accessors (`get_title`, `get_wm_class`, `get_pid`, `get_gtk_application_id`, `get_sandboxed_app_id`, `get_frame_rect`, `get_workspace`), `Shell.WindowTracker` for the desktop file id |
| `clipboard` | `Meta.Selection` `owner-changed`, content through `St.Clipboard` (verified in the spike) |
| `selection` | **Not through PRIMARY.** See below |
| `pointer` | `global.get_pointer()` |
| reads: Pixels | `Shell.Screenshot` for one window, without a consent dialog |

Verified against primary sources (November 2026): Mutter does not implement
`ext-data-control-v1` (protocol support tables, Mutter 51); `St.Clipboard` reads both `PRIMARY`
and `CLIPBOARD` (`st-clipboard.h`); the pie menu Kando ships an MIT-licensed companion extension
that reads the focused window's title and class from `Meta.Window` and the pointer from
`global.get_pointer()`, with version-specific fallbacks for the pointer on newer Shells. Read its
source before writing ours. It exports a D-Bus service; ours deliberately does not.

Rules for the extension:

- It is a **client** of the daemon socket (`Gio.SocketClient`, Unix address). It exports nothing on
  D-Bus. The extensions site's review guidelines are silent on local sockets, so expect a
  case-by-case review; the alternative distribution is our own package.
- **Review rules that bind us** (extensions.gnome.org guidelines): clipboard access must be
  declared in the description; clipboard data must not be shared with a third party without
  explicit user interaction; no default keyboard shortcut for clipboard data; no bundled binaries.
  Consequence, at every observation level: the extension sends selection or clipboard **content**
  only in answer to a read that originates from a user-initiated `capture`. Its ambient events
  carry lengths and kinds, never content.
- It contains no logic beyond translating Shell objects into observations. Reconnect with backoff.
- Supported Shell versions: 46 to current (extensions use the same module format since 45).
  Declare each version in `metadata.json`; budget a review at each GNOME release (every six months).
- **The selection on GNOME does not come from PRIMARY.** Since GNOME 50, primary paste is
  disabled by default, and GTK applications then publish no primary selection at all (confirmed
  upstream and in the spike, `sense/spike/gnome-shell/`). Users can re-enable it, so the extension
  still reports a PRIMARY owner when one exists, with quality `Ambient`, but nothing may depend on
  it. The selection ladder on GNOME is: browser or editor extension (exact) → `linux.atspi`
  (exact, accessible applications) → copy-based read through the acting service
  (`12-acting.md`, exact, any application, only inside a user-initiated capture).
- Installing an extension needs a re-login on Wayland. Onboarding must say so.

Without the extension (`NeedsInstall`): `linux.atspi` supplies focus for accessible applications,
everything family-independent still works, and window signals are otherwise unavailable. This is
a declared degraded mode, visible in `capabilities`.

## Family: `kde`

| Signal | Interface |
|---|---|
| windows, focus, titles, app ids, pointer | A **KWin script** (`ext/kwin`), loaded at run time through `org.kde.KWin /Scripting` (no install step, no re-login). It listens to `workspace.windowActivated`, `windowAdded`, `windowRemoved`, reads `caption`, `resourceClass`, `desktopFileName`, `pid`, and reports with `callDBus` to an object owned by the daemon |
| `selection`, `clipboard` | Wayland `ext-data-control-v1` (KWin implements it): change events and content for both CLIPBOARD and PRIMARY |
| reads: Pixels | `linux.portal.screen` |

KWin scripts cannot open sockets, so this is the one inbound D-Bus path. The daemon's object
accepts only the fixed set of report calls and treats their content as untrusted (it can describe
windows, nothing more).

## Family: `wlroots`

Standard protocols, no helper needed. Source `linux.wlr`.

| Signal | Interface |
|---|---|
| windows, focus, titles, app ids | `ext-foreign-toplevel-list-v1` (identity and titles) plus `wlr-foreign-toplevel-management-unstable-v1` (activated state). Where richer data is wanted, the compositor's own IPC: sway/i3 IPC `subscribe` to window events, Hyprland's event socket, niri's event stream |
| `selection`, `clipboard` | `ext-data-control-v1`, falling back to `wlr-data-control-unstable-v1` |
| reads: Pixels | `ext-image-copy-capture-v1`, falling back to `wlr-screencopy`; else the portal |

Bind each protocol only if the registry advertises it; each one maps to independent signals, so a
compositor that offers some of them gets partial capability rather than none.

## Signal coverage by family

| Signal | x11 | gnome | kde | wlroots |
|---|---|---|---|---|
| focus, windows | `linux.x11` | `linux.gnome` (extension) | `linux.kwin` (script) | `linux.wlr` |
| selection (ambient, PRIMARY) | `linux.x11` | only if the user re-enabled primary paste | data-control | data-control |
| selection (copy-based) | acting service | acting service (extension) | acting service | acting service |
| selection (exact), element | `linux.atspi`, extensions | same | same | same |
| clipboard | `linux.x11` | `linux.gnome` | data-control | data-control |
| pixels | X11 | extension, portal | portal | screencopy, portal |
| everything else | family-independent sources | | | |

## Forbidden here

`/dev/input`, `uinput`, `evdev` and any other path that needs elevated rights or sees raw keys;
`LD_PRELOAD` or ptrace on other processes; reading browser profile databases.
