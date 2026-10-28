# Optional GNOME desktop adapter

This small GNOME Shell extension supplies global cursor coordinates and Ctrl+Space to the
Bloom Tauri app. It solves GNOME Wayland's restricted XWayland cursor reporting when the
pointer is over another app, the desktop, or Bloom's main window is minimized. The overlay
still renders in Tauri through XWayland. Windows, macOS and full X11 sessions do not need it.
The initial supported Shell version is GNOME 46.

## Install and run

Close Bloom before installing. From the repository root:

```bash
make gnome-setup
```

The installer builds the archive in `.bloom/gnome/`, installs it for your user and attempts
to enable it. If GNOME has not discovered this new extension yet, **log out and back in once**,
then run:

```bash
gnome-extensions enable bloom-desktop@bloom.local
```

Check and launch:

```bash
make gnome-status
make ui-gnome
```

`make ui-gnome` checks the live adapter before building or launching, and explicitly selects
XWayland (`GDK_BACKEND=x11`) so the native overlay can be positioned. In Preferences, Quick
launch should report that it uses the GNOME adapter.
Restart Bloom after enabling or updating the extension. Ctrl+Space works outside Bloom,
including while its main window is minimized. Use the native titlebar's Minimize control
next to Maximize and Close to keep Bloom running. Press Ctrl+Space again to cancel the pie.

The extension reserves Ctrl+Space only while a connected Bloom instance enables Quick launch.
Turning it off, exiting/crashing Bloom, or disabling the extension releases the shortcut.
If another desktop shortcut already owns Ctrl+Space, Preferences reports the conflict.
This helper does not read screen contents or execute actions; those still use Bloom's
existing Python pipeline and approval flow.

## Adapter unavailable

`The name is not activatable` means the Shell service is not running. It is not a build
failure. Run `make gnome-status` for the actual state and the relevant activation steps.
GNOME can disable every user extension through a master switch even when the individual
`gnome-extensions enable` command succeeds. Check it with:

```bash
gsettings get org.gnome.shell disable-user-extensions
```

If it prints `true`, turn extension support on in GNOME Extensions, or run:

```bash
gsettings set org.gnome.shell disable-user-extensions false
gnome-extensions enable bloom-desktop@bloom.local
make gnome-status
make ui-gnome
```

The master switch also permits previously enabled extensions to run. Bloom's installer does
not change it silently. A ready adapter reports `State: ACTIVE` and protocol 1; a successful
extension-install or enable command alone is insufficient. If the extension reports ERROR,
inspect `journalctl --user -b` for Bloom errors. Restart Bloom after restoring the service.

## Remove

```bash
make gnome-remove
```

Only `bloom-desktop@bloom.local` is removed. Other extensions and desktop settings are unchanged.

## Interface and validation

The local session-bus service is `com.bloom.Desktop` at `/com/bloom/Desktop`:

| Member               | Data                                                                     |
| -------------------- | ------------------------------------------------------------------------ |
| `GetVersion`         | protocol version 1                                                       |
| `GetPointer`         | x, y, stage width and stage height                                       |
| `SetShortcutEnabled` | enable/disable for the calling connection; returns registered state      |
| `Trigger`            | pointer coordinates and stage dimensions captured at shortcut activation |

The native adapter converts stage coordinates to X11 root units, then clamps the overlay to
the monitor work area. Mixed-scale physical monitor setups still need desktop validation.
The service watches its shortcut owner's D-Bus connection, so a crashed client cannot leave
the binding reserved. A second app connection cannot disable an existing owner's binding.

```bash
npm --prefix shell run test:gnome        # ownership, capture timing and cleanup (Node)
cd shell
npm run test:gnome-native               # Rust ↔ actual GJS service on a private D-Bus
```

The native test requires GJS and `dbus-run-session`. It does not install an extension or
change desktop keybindings. CI runs it on Linux. The extension's Shell hooks require an
unlocked GNOME desktop for end-to-end validation.
