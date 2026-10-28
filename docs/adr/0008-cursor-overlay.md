# 0008 · Cursor-relative pie overlay

- **Date**: 2026-10-23
- **Status**: accepted for the D prototype
- **Deciders**: D, following the user's overlay request

## Decision

Replace the framed pie popup with a transparent, borderless, always-on-top webview containing
only the radial menu. Capture the cursor at invocation and position the menu around it,
clamped to the destination monitor's work area. Keep the existing ActionPlan/choice transport.
Hide on selection, Escape, background click or focus loss. Show the main window for approval.

Add tauri-plugin-global-shortcut 2.3.1, pinned for the current Rust 1.86 toolchain, to register
Ctrl+Space on Linux/X11 and Command+Shift+Space on macOS. Add serde as a direct dependency
for the native shortcut status returned to React. Shortcut failures retain an in-app fallback
and are reported in Preferences. Disabling Quick launch unregisters the native shortcut.

macOS transparency needs Tauri's macos-private-api feature and macOSPrivateApi configuration.
The prototype enables these; macOS packaging, App Store eligibility and platform testing
remain separate work.

## Platform limits

Tauri's cursor lookup and the shortcut backend do not support native Wayland. Do not claim a
global shortcut there or silently change the display backend. The overlay remains transparent,
with compositor-controlled placement and a focused-app shortcut. `GDK_BACKEND=x11 make ui`
lets developers test the XWayland path; interception outside X11 applications depends on the
compositor. This GNOME session exposes no GlobalShortcuts desktop portal. Full Wayland support
requires a compositor integration. Do not change desktop settings automatically.

Hiding the overlay lets the OS choose the next focused window. Bloom explicitly restores its
main window only for approval; restoring an arbitrary
external application is not implemented.

## Sources

- [Tauri window API](https://docs.rs/tauri/2.11.6/tauri/webview/struct.WebviewWindow.html)
- [Global shortcut backend platform support](https://github.com/tauri-apps/global-hotkey)
- Pinned Tao 0.35.3 `platform_impl/linux/util.rs`: cursor lookup returns the placeholder (0, 0)
  on Wayland. Detect this platform explicitly rather than treating it as a real cursor position.
