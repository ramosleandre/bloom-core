# 0009 · Read the cursor independently of Bloom's main window

- **Date**: 2026-10-23
- **Status**: accepted for D's desktop overlay correction

## Problem

The original Linux implementation used Tauri/Tao's GTK device position. Outside Bloom this
can be stale or limited to the last XWayland surface. The shortcut captured an anchor, but
the React handler immediately captured it again. Neither behavior guarantees placement at
the point where the user invoked Bloom while another app was active.

## Decision

Declare x11rb 0.13.2 as a direct Linux dependency (already present through the shortcut
plugin). Query the X11 desktop root and use root coordinates directly, independently of
the main window's location, visibility or scale. Preserve the shortcut's original anchor;
only button/in-app launches request a new capture. Other platforms retain Tauri's desktop
cursor API. The overlay is clamped only to monitor work areas.

Hold a Web Lock while each native bridge view is alive, releasing it on teardown. WebKit
can otherwise suspend minimized webviews, which would postpone the main view's existing
shortcut and state handlers. This uses the browser API without a new dependency or a new
Python/core transport. Browser previews do not acquire these desktop locks.

GNOME Wayland may restrict XWayland coordinates even for an X11 root query. Correct global
coordinates there require a compositor-side integration; XWayland alone does not guarantee
them. This is distinct from the native X11 path.

## Sources

- Pinned Tao 0.35.3 `platform_impl/linux/util.rs`: GTK device cursor lookup.
- Pinned x11rb 0.13.2 `protocol/xproto.rs`: QueryPointer returns root and window coordinates.
- [GNOME cursor integration in another radial menu](https://juhlabs.github.io/juhradial-mx/compositor-support/)
