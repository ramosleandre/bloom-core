# 0012 · Context acquisition is a standalone local engine ("Sense"), written in Rust

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

Bloom is only as good as what it knows about what the user is doing. Today that knowledge is one
method, `Surface.observe()`, planned as Python code calling the accessibility API of each system
from inside the daemon ([ADR 0004](0004-python-daemon-tauri-shell.md),
[ADR 0007](0007-linux-and-macos.md)). Three problems:

- All the per-OS work of the project is concentrated there, mixed with the action catalog.
- macOS attaches the Accessibility permission to the code identity of the process that makes the
  calls; whether a Python child of the shell inherits it is unreliable (the open risk of ADR 0004).
- The native APIs involved (Objective-C frameworks, COM and WinRT, D-Bus, X11 and Wayland
  protocols) have mature Rust bindings; from Python they pull system packages into the install.

Reading the user's context is also a problem of its own, independent of what Bloom does with it.

## Options considered

1. Keep `observe()` in the Python surface block, one backend file per OS.
2. Put the OS code in the Tauri shell (the plan B of ADR 0004) and send an observation to the daemon.
3. A separate component, Sense: a small daemon that owns all observation of the user, with a
   documented data model and a local API; the shell and the Python daemon are its clients.

## Chosen option

3, written in Rust. Specification: [`sense/`](../../sense/README.md).

## Rationale

- One process, with one stable code identity, holds every OS permission. The inheritance question
  disappears.
- The engine must be resident for the session, not only while the shell's window is open
  ([ADR 0013](0013-read-on-request-ambient-opt-in.md)). Tying it to the interface
  process (option 2) ties its lifetime and its failures to the interface.
- A component with its own API can be built, run and tested alone (`sense capture` prints JSON),
  before the shell exists and without the Python daemon.
- Rust: maintained bindings for all three API families, the same toolchain as the Tauri shell,
  per-OS code excluded at compile time, cross-checkable from any machine with `cargo check --target`.
- Bloom's Python daemon stops calling the OS at all, so it behaves identically on every machine.

## Accepted consequences

- A third process and a second language in the surface block's territory. Rust is new for part of
  the team.
- The surface block splits: Sense (observation) on one side; the action catalog, `resolve` and
  `execute` stay in Python. Acting on the system (paste, hotkey, menu placement) is decided in
  [ADR 0018](0018-acting-service-with-closed-effects.md).
- `Surface.observe()` becomes a call to Sense's `capture`; the `Observation` contract will change
  to carry provenance (a `contract` pull request).
- Packaging grows: a signed helper on macOS, package identity on Windows for some sources.
- New dependencies (Rust crates for each platform), listed in the specification.
- Supersedes the "plan B" of ADR 0004 and the "one Python backend per system" consequence of ADR 0007.
