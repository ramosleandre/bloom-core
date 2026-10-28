# 0007 · An interactive shell over the existing Interface port

- **Date**: 2026-10-23
- **Status**: accepted for the Linux-first UI prototype
- **Deciders**: D, with the user's implementation request

## Context and problem

The terminal demo already produces ActionPlan and ApprovalRequest values, but MockInterface
chooses and approves automatically. D needs a real pie menu, approval dialog and activity
window without changing the decision, context, surface or core contracts.

## Decision

- Implement the existing Interface protocol with a local authenticated WebSocket bridge in D.
- Add aiohttp for WebSocket transport and serving the compiled UI from the same loopback origin.
  It avoids introducing separate HTTP and WebSocket server dependencies.
- Build the shell with React, React DOM, TypeScript and Vite. React components separate the
  activity window, pie geometry and approvals; Vite provides the development and build tools.
- Use Tauri 2 and its API/CLI for the desktop wrapper, following ADR 0004. The browser can run
  the identical UI when native development libraries are unavailable.
  Keep the Rust crate and JavaScript API/CLI on the same minor release (2.11 for Rust 1.86).
- Generate TypeScript models from Pydantic JSON Schema using json-schema-to-typescript.
  Never maintain a second handwritten copy of the core contracts.
- Use Playwright for meaningful browser integration tests of the actual Python-to-UI round trip.
- Use Prettier to keep the frontend source consistently formatted.

## Alternatives considered

- Parsing printed logs: these are final output, not an interactive protocol.
- Polling the audit file: useful for history, but cannot suspend a run for a user's choice.
- A new UI-specific decision engine: duplicates A and disconnects the prototype from Bloom.

## Consequences

The bridge binds only to 127.0.0.1, validates Host and Origin, and requires a random launch
token before exposing any data or accepting commands. Tokens travel in the launch URL
fragment and the first WebSocket message, never in request query strings or the audit.
One run is active at a time. Replies are matched to the pending request; disconnection and
approval expiry cancel safely. The prototype does not expose argument editing.

The desktop frontend is the default UI entry point (`make ui` / `make desktop`). It uses
Tauri's Vite development server for hot reload and a separate Python bridge. The bridge
and desktop development processes are shut down together. `make ui-preview` is an explicit
browser-only development option. This follows the user's clarification that D's deliverable
is the Tauri frontend.

The first desktop launch uses a development Python process and Vite-served frontend. Bundling a
Python runtime, macOS Accessibility, system-wide shortcuts and OS focus restoration remain
separate milestones. Linux-native builds require GTK/WebKit development packages.
