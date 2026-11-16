# 03 · Client API

Clients talk to `sensed` over one local socket. The same socket carries the source protocol
(`02-sources.md`); the first message decides the role.

## Transport

| OS | Endpoint |
|---|---|
| Linux | `$XDG_RUNTIME_DIR/sense/sense.sock` (Unix socket, mode 0600) |
| macOS | `~/Library/Application Support/Sense/sense.sock` (Unix socket, mode 0600) |
| Windows | `\\.\pipe\sense-<user SID>` (named pipe, ACL: the user only) |

Framing: newline-delimited JSON-RPC 2.0, UTF-8. One connection may carry many requests and one
event stream. There is no TCP listener, ever.

## Handshake

```
→ client.hello { name, version, role: "reader" | "source", token?, own_pids?: [..] }
← { protocol: "1", daemon_version, granted_scopes: [...] }
```

- `own_pids`: the client's processes that open windows. Focus changes to their windows are ignored
  (`01-data-model.md`), so a client's own interface never becomes "what the user is doing".
- Scopes limit what a reader may ask for: `state`, `timeline`, `content.selection`,
  `content.document`, `content.pixels`, `clipboard.text`, `admin`, and `act` (the acting service,
  `12-acting.md`; first-party only). See `04-privacy-security.md`.
- `token`: sources present their install-time token; first-party clients present the session
  token their parent received when it launched the daemon (`11-runtime.md`).

## Methods

| Method | Purpose |
|---|---|
| `capabilities.get` | What this session can observe, per signal: the providing source, or the reason none does, and the remedy. |
| `state.get { paths? }` | The current state, or the listed subtrees. Answered from memory. |
| `capture { want?, recent?, content?, deadline_ms, selection_fallback? }` | A consistent snapshot. Runs tier B reads for the current focus; may include deep reads. |
| `content.read { target, kind, max_bytes, deadline_ms, max_channel? }` | One deep read against a target (the content ladder). |
| `timeline.query { since_seq? , since?, kinds?, limit }` | Recent events. |
| `events.subscribe { kinds?, from_seq? }` | Stream of events as notifications. Resumable by `seq`. |
| `permissions.request { source }` | Triggers the OS prompt or opens the right settings pane. Only on a user action. |
| `privacy.get` / `privacy.set` | Observation level, exclusions, retention (scope `admin`). Raising the level is only accepted from the first-party settings interface, after its consent screen. |
| `privacy.pause { for_s? }` / `privacy.resume` | Stops every source and clears state and timeline. |
| `access.log { since?, client? }` | The access ledger: who read which kinds from which application, when. No content (`04-privacy-security.md`). |
| `inspect.stream` | Everything the engine sees, as the privacy filter left it. For the transparency view. |

### `capture`

The call a client makes when it needs "the context, now".

```json
{ "method": "capture", "params": {
    "want":    ["focus", "selection", "document", "clipboard", "editor", "terminal", "media", "calendar"],
    "recent":  { "kinds": ["focus.changed", "clipboard.changed", "tab.activated"], "window_s": 300, "limit": 30 },
    "content": [ { "kind": "surrounding", "before": 500, "after": 200 } ],
    "deadline_ms": 50 } }
```

Semantics:

1. The engine fixes a `seq`. Everything returned is consistent with that point.
2. Subtrees in `want` are copied from the state.
3. Tier B sources are asked to refresh the focused element, in parallel, within the deadline.
4. Each `content` item runs as a `content.read` on the focus target, within the same deadline.
5. Whatever missed the deadline comes back `Unavailable(Timeout)`. `capture` never returns late.
6. The response carries `target`, valid for follow-up reads.

At observation level 1, step 2 has little to copy: the engine asks the sources for the wanted
signals through their `Current` read, inside the same deadline, and discards the values once the
response is sent. `recent` comes back empty with reason `level`. `capture` should be called in
response to a user action; the indicator signals every capture and names the client.

`selection_fallback: "copy"` lets the engine read the selection by copying when no other channel
has it (`12-acting.md`). It requires the `act` scope and adds up to 150 ms.

`capture` and `content.read` accept `format: "json" | "card"`. `card` is the compact text
rendering meant to be placed in a model's context (`09-exposing-context-to-agents.md`).

A client with a tight budget calls `capture` with a small deadline and no `content`, shows its
interface, then calls `content.read` for what the user's choice turns out to need.

### `capabilities.get`

```json
{ "signals": {
    "focus.window":   { "status": "available", "via": "linux.gnome" },
    "selection.text": { "status": "available", "via": ["ext.browser", "linux.atspi", "linux.gnome"],
                        "note": "ambient quality outside accessible apps" },
    "document.url":   { "status": "needs_install", "component": "ext.browser",
                        "how": "Install the Sense browser extension" },
    "content.pixels": { "status": "needs_permission", "source": "linux.portal.screencast",
                        "how": "Approve the screen sharing prompt" },
    "notifications":  { "status": "unavailable", "reason": "not exposed on this platform" } },
  "sources": [ { "id": "linux.gnome", "state": "running", "version": "…" } ] }
```

This one document drives onboarding ("grant this to get that"), degraded modes in clients, and
bug reports.

## Errors

JSON-RPC errors with stable codes: `approval_pending` and `approval_denied` (per-application
approval), `scope_denied`, `unknown_target` (the window is gone),
`paused`, `deadline_exceeded` (only for `content.read`; `capture` degrades instead), `bad_request`.

## Versioning

`protocol` is a single integer. Additive changes (new optional fields, new event kinds, new
methods) do not bump it; clients must ignore what they do not know. Removing or renaming does.

## Other assistants: the MCP server

Third-party assistants do not use this API. They use `sense-mcp`, a Model Context Protocol server
that maps resources and tools onto the methods above (`08-open-protocol.md`). It is off by default.

## Client libraries

`clients/python` and `clients/typescript` contain generated types plus a thin connection class
(`connect`, `capture`, `read`, `subscribe`). No logic. The CLI `sense` is the reference client:

```
sense caps                     # capabilities.get, as a table
sense state [path]             # state.get
sense capture --want focus,selection --deadline 50
sense read --kind document     # content.read on the current target
sense watch [--kinds ...]      # events.subscribe
sense record out.jsonl         # anonymized stream, for fixtures
sense inspect                  # live transparency view in the terminal
sense emit <kind> <json>       # used by shell hooks
```
