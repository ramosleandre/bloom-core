# 0011 · Native Linux window controls

- **Date**: 2026-10-23
- **Status**: accepted
- **Decider**: the user's request for Minimize next to Close in the window titlebar

## Decision

Remove the in-page Minimize button and its Rust command. On Linux, use GTK's standard
client-side titlebar, with minimize, maximize and close controls. Set GTK_CSD and the GTK
decoration layout in Bloom's own process; do not change the desktop's global button layout.
Windows and macOS keep their existing native window decorations. The pie remains undecorated.

Declare gtk 0.18.2 as a direct Linux dependency. Tauri already uses this exact GTK binding;
the dependency gives Bloom typed access to the process-local GTK settings, without custom
web controls or unsafe native pointers.

## Validation

Inspect the actual native titlebar and minimize it with native input, then activate the
desktop shortcut and select an action while verifying that the main window stays minimized.
Browser tests continue to verify background event delivery, without an in-page Minimize control.

## Sources

- [GTK decoration layout](https://docs.gtk.org/gtk3/method.HeaderBar.set_decoration_layout.html)
- [GTK environment variables](https://docs.gtk.org/gtk3/running.html)
