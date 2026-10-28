# 0010 · Optional GNOME desktop adapter

- **Date**: 2026-10-23
- **Status**: accepted for the GNOME 46 prototype
- **Decider**: the user's explicit request for the optional GNOME adapter

## Decision

Keep the shared Tauri UI and Python contracts unchanged. On GNOME Wayland, an optional Shell
extension reads global.get_pointer() and owns Ctrl+Space while Bloom requests it. A small,
Linux-only Rust adapter communicates over the session D-Bus interface com.bloom.Desktop.
The overlay continues to use XWayland for absolute window positioning.

Declare dbus 0.9.12 as a direct Linux dependency; it is already present through Tauri. The
adapter keeps its D-Bus connection alive, listens for shortcut signals and queries the
cursor for button launches. The extension watches that connection's unique owner and
releases the shortcut on disconnect, including a Bloom crash. A second client cannot
replace an existing shortcut owner. Disabling the extension removes its signals, exports,
name watch and keybinding.

One worker owns all D-Bus reads and RPCs, avoiding a signal receiver consuming an RPC reply.
Shutdown stops and detaches that worker instead of joining it on GTK's main thread, where
a pending GUI callback could otherwise deadlock. Closing its connection releases the
extension's shortcut ownership.

Pointer messages contain stage coordinates and dimensions. Map these to X11 root dimensions
before positioning, avoiding GTK's window-local/stale pointer cache and accounting for the
global XWayland scale. Monitor work-area clamping still happens in the native overlay.

Without the optional helper, true X11 sessions keep the existing desktop-root path. Native
Wayland windows still cannot be positioned absolutely: use GDK_BACKEND=x11 when launching
Bloom on GNOME Wayland. Installation does not change the user's desktop session or other
extensions. GNOME may require one logout/login to discover a newly installed extension;
the installer reports that requirement without logging the user out.

## Compatibility and validation

The extension initially declares GNOME 46, the installed version. Other GNOME versions need
validation before being added. Windows/macOS do not compile the D-Bus adapter or need this
extension. The pure service policy is tested with Node; the actual GJS D-Bus service and
Rust client are tested on a private session bus, separately from desktop keybindings.

## Sources

- [GNOME extension development](https://gjs.guide/extensions/development/creating.html)
- [GJS D-Bus services](https://gjs.guide/guides/gio/dbus.html)
- [Shell global pointer coordinates](https://gnome.pages.gitlab.gnome.org/gnome-shell/shell/method.Global.get_pointer.html)
