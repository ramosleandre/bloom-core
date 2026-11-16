# 0015 · On Linux, target session families detected at run time, not an X11 session

- **Date**: 2026-11-16
- **Status**: proposed (amends [ADR 0007](0007-linux-and-macos.md))
- **Deciders**: Everyone

## Context and problem

ADR 0007 states that "V1 targets an X11 session on Linux" and treats Wayland as a risk to test.
That premise no longer holds: GNOME removed its X11 session in version 50, so the default desktop
of current Fedora and Ubuntu is Wayland-only, and KDE Plasma is on the same path. An X11-only
Bloom would not run on most new Linux installations.

Under Wayland, an ordinary client cannot learn which window is focused, read another client's
selection, or see the pointer outside its own surface. Each compositor decides what to expose, and
they decided differently. "Linux support" therefore has no single implementation, and the
distribution (Fedora, Ubuntu, Debian, Arch) is not the variable that matters.

## Options considered

1. X11 only (the current text).
2. One Wayland desktop only.
3. Cross-desktop portals only, accepting what they lack (no portal reports the focused window).
4. Four session families (X11, GNOME, KDE, wlroots compositors), one binary, sources selected at
   run time by probing the session; compositor-side helpers of our own where the compositor offers
   no other door.

## Chosen option

4. Details: [`sense/spec/platform-linux.md`](../../sense/spec/platform-linux.md).

- GNOME: our own Shell extension, connecting out to the daemon.
- KDE: a KWin script loaded at run time, plus the data-control protocol.
- wlroots compositors: standard protocols (foreign-toplevel, data-control).
- X11: EWMH and XFixes; kept because it is the smallest provider, serves Xfce, Cinnamon and
  similar desktops, and runs headless in CI as the reference for conformance tests.
- Everything that does not depend on the family (AT-SPI, MPRIS, login1, PipeWire, portals) is
  shared.

Build order: X11 (for CI), GNOME, then KDE, then wlroots.

## Rationale

- It matches how the platform is actually divided, so each provider is small and the capability
  report is honest per session.
- The restrictions of Wayland are a security model, not defects to patch. The legitimate way
  through is code the user installs into the compositor, or a protocol the compositor offers.
- Detection by probing (bus names, Wayland globals) keeps working on distributions and desktops
  nobody on the team uses.
- Option 3 leaves GNOME without application identity, the most basic signal.

## Accepted consequences

- A GNOME Shell extension to maintain: one review per GNOME release (every six months), a re-login
  at first install on Wayland, distribution through the GNOME extensions site.
- Without the extension, GNOME runs in a declared degraded mode.
- Sense must be installed unsandboxed on Linux (no Flatpak or Snap confinement).
- Accessibility (AT-SPI) is not switched on globally by default, because it costs performance in
  Chromium and Electron applications; browser and editor context comes from extension sources.
- Never `uinput`, `/dev/input` or anything needing elevated rights.
