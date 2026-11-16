# 0014 · One source contract for native APIs and extensions; one schema for every language

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

Context comes through very different mechanisms: native OS APIs called in-process, and code that
must run inside another program (a browser extension, an editor extension, a shell hook, a GNOME
Shell extension, a KWin script). Each OS needs a different mix. Without a common shape, every new
mechanism adds special cases to the engine and to its clients.

## Options considered

1. One large interface per operating system ("the macOS backend", "the Linux backend"), each
   responsible for producing a whole observation.
2. Many small **sources**, each observing one thing through one mechanism, all behind the same
   contract; extensions implement that contract over a local protocol by connecting out to the
   daemon. The engine merges what they report with a precedence table.
3. Adopt an existing cross-platform accessibility library as the abstraction.

## Chosen option

2. Types are defined once in Rust and exported as JSON Schema; Python and TypeScript types are
generated.

## Rationale

- Per-OS differences shrink to "which sources exist here". Nothing above the source layer branches
  on the OS. Adding an OS, a Linux desktop or an application is adding sources.
- Sources of different quality can coexist for the same signal (the compositor knows the window,
  accessibility knows the element, the browser extension knows the page). Option 1 forces each
  backend to re-implement that merging.
- Extensions that connect out expose no service that other local processes could call, and are
  authenticated at one place. A browser extension is identical on all three OSes.
- Every value carries its source, time and quality, so clients can weigh a selection read from the
  focused element differently from the Linux PRIMARY selection.
- Option 3 covers only one channel (accessibility) and is aimed at UI automation. We use such
  libraries as references and bind to the OS directly, so that deadlines, secure-field handling
  and event subscriptions stay under our control.

## Accepted consequences

- A reducer with precedence and staleness rules: more design than a single read, tested once.
- A wire protocol to version and keep compatible.
- Several small installable pieces (browser extension, native-messaging bridge, Shell extension,
  shell hooks), each with its own distribution channel.
- Sense's schema is Rust-first while Bloom's contracts are Python-first. The boundary is one
  mapping in the surface block.
