# 0007 · Bloom targets Linux and macOS

- **Date**: 2026-10-29
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: Everyone

## Context and problem

Until now Bloom targeted macOS only ([ADR 0004](0004-python-daemon-tauri-shell.md), `docs/architecture.md`). Three of the four team members work on Linux and have no Mac. With a macOS-only product, the surface block (screen reading) and the Tauri shell could only be tested for real on a single machine, and most of the team would only ever run mocks.

## Options considered

1. macOS only, Linux for development: the core, the mocks, the tests and the bench run on Linux; everything that touches the real system is tested on the team's only Mac.
2. Linux and macOS: the same product on both systems; the surface block has one backend per system behind the same `Surface` facade.
3. Linux only.

## Chosen option

2.

## Rationale

- Every team member can run and test their block for real on their own machine.
- The architecture already isolates the operating system in one place: only block C (surface) touches the system, behind the `Surface` facade. The core, the contracts and blocks A, B and D do not change.
- Tauri ([ADR 0004](0004-python-daemon-tauri-shell.md)) already supports Linux and macOS, and CI already runs on Linux.
- Option 1 leaves three people unable to test what they build. Option 3 drops the Mac for no reason.

## Accepted consequences

- Block C gets two backends, one per system, chosen at startup by `app.py`. They share the action catalog and the per-action logic. More code to maintain and to test.
- On Linux, screen reading goes through AT-SPI (the accessibility bus). Its Python bindings depend on system packages (GObject introspection), not on PyPI alone: the Linux setup will need a documented list of system packages.
- Wayland restricts what an application may do: global hotkeys, knowing the active window, placing an overlay at the cursor. V1 targets an X11 session on Linux; what still works under Wayland is a risk to test in v0.0, not a promise.
- The contract field `Observation.app_bundle_id` holds the platform's application identifier (a bundle id on macOS, a desktop application id on Linux). Renaming it is a contract change (label `contract`), to discuss in v0.1.
- The rules backend and the sample dataset only know macOS application ids: Linux applications get no hint from the rules until some are added.
- Performance is measured on a single reference machine (one system, noted with its hardware), so that measurements stay comparable.
- macOS-specific work (the Accessibility permission, the macOS backend) is done on the team's only Mac.
- The plan B of ADR 0004 (reading the screen on the Rust side) must now hold on both systems.
