# 0004 · A Python daemon for the brain, a Tauri shell for the interface

- **Date**: 2026-10-11 (direction already set in March 2026)
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: Everyone

## Context and problem

Bloom must capture a global hotkey, display an overlay menu and read the screen on macOS, while calling AI models.

## Options considered

1. Everything in Python, interface included (PyQt, or a menu bar app).
2. A native Swift application.
3. A Python daemon for the core and blocks A, B, C; a Tauri shell (TypeScript and Rust) for the interface; a local IPC bridge between the two.

## Chosen option

3.

## Rationale

- The AI ecosystem (local models, SDKs, measurement tools) is in Python.
- Tauri gives a lightweight interface, quick to evolve with web technologies the team knows, and portable later.
- Swift integrates better with macOS, but nobody on the team uses it, and the AI code would stay in Python: an IPC bridge would be needed anyway.

## Accepted consequences

- Two processes, and an IPC bridge to secure: random token, Origin header check.
- Uncertainty about the macOS Accessibility permission when Python runs as a Tauri sidecar. To be tested in v0.0; plan B: read the screen on the Rust side and send the `Observation` to the daemon, without changing the contract.
- More complex packaging (bundling Python into the app).
