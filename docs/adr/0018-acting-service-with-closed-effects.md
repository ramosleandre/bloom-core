# 0018 · Acting on the desktop: a closed list of effects, in the same daemon, never exposed to third parties

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

Bloom must react to a global hotkey, show its menu at the pointer, and insert text into the
application the user was in. These are operating-system operations with the same per-OS
difficulties as reading context, and on Wayland they depend on the same compositor-side helpers.
[ADR 0012](0012-context-engine-as-a-standalone-daemon.md) left them out. Today they are split by
plan between the Tauri shell (hotkey, window) and Python (`execute`).

Software that can type into other applications is far more dangerous than software that reads.
The design must give Bloom what it needs and nothing more.

## Options considered

1. Leave it as planned: Tauri plugins for the hotkey and window, Python calling each OS for
   execution.
2. A general automation layer in the daemon (press keys, click, move windows), usable by any client.
3. A small acting service inside the context daemon, with a closed list of effects (bind a hotkey,
   place the client's own window, restore focus, insert text, set the clipboard, read the selection by copying), its own API
   namespace and scope, restricted to first-party clients and absent from the MCP server.

## Chosen option

3. Specification: [`sense/spec/12-acting.md`](../../sense/spec/12-acting.md).

## Rationale

- Acting needs the same Accessibility grant, the same Shell extension connection and the same
  session detection as reading. A second process would duplicate all three, and on macOS would
  require a second grant from the user.
- Tauri's global-shortcut support on Linux depends on X11, and a Wayland client cannot position
  its own window. Option 1 cannot work on the default Linux desktops without the compositor
  helpers that the daemon already owns.
- A closed list makes the risk reviewable: there is no method by which a client, or a model
  speaking through a client, can press arbitrary keys or click. Insertion is bound to a target
  captured earlier and verified again before acting.
- Keeping it off the MCP server means third-party assistants can read (with consent) but never act.
- Policy stays where it is: Bloom's core decides whether an action needs approval. The service
  executes and reports.

## Accepted consequences

- The "read-only" guarantee applies to the Sense crates and API, not to the daemon process as a
  whole. This must be said plainly wherever the daemon is described.
- Since GNOME 50 disables the primary selection by default, reading the selection by copying is
  the only universal fallback there. It sends a key to another application, so it is an effect of
  this service and never a background source; it must never send Ctrl+C to a terminal.
- Insertion through the clipboard briefly replaces the user's clipboard; it is restored, and
  clipboard managers are asked to ignore the transient entry, which not all of them honour.
- On portal-based Wayland sessions the user confirms the hotkey and may change it; on some
  compositors the only option is a manual binding to a command.
- On GNOME, key events are injected from inside the compositor by our extension. This is the most
  sensitive code in the project and needs the strictest review.
- Requests to add general automation ("just one click effect") will come. They are a different
  product with a different threat model and need a new ADR.
