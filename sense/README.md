# Sense: the context engine

> Status: **proposed design**, not yet accepted by the team (ADRs 0012 to 0018). A first
> slice is implemented to back the proposal with running code: see "What exists" below.
> "Sense" is a working name.

Sense is a local service that knows what the user is doing on their computer. It gathers every piece
of context the operating system and its applications expose, normalizes it into one data model, and
serves it to clients through one API. Bloom is its first client.

It exists as a separate component because the problem is separate: reading context is the same
problem whatever is done with it afterwards, and it is where all the per-OS code lives.

## What exists

A development slice, not a product. `sensed` refuses to start without `--dev-no-indicator`,
because the indicator the specification requires is not written yet.

| Piece | State |
|---|---|
| `sense-schema` | Types for the minimum slice, JSON Schema emitted to `schema/` with a drift test |
| `sense-core` | Privacy filter (levels, sanitizing, secret redaction), reducer with precedence, focus tracking that ignores a client's own windows, exclusions, capture with a deadline, access ledger, the `card` rendering. No OS code |
| `sensed` | User-only Unix socket; client API (`capabilities.get`, `capture`, `privacy.*`, `access.log`, `timeline.query`); source protocol for extensions |
| `sense` (CLI) | `caps`, `capture [--card]`, `level`, `log`, `timeline` |
| `sense-platform-linux` | The `linux.atspi` source: focused element and exact selection over the accessibility bus, tracking focus through events, never reading password fields. Builds, connects and takes part in captures; **not yet proven end to end** (open question S8) |
| `ext/gnome-shell` | The `linux.gnome` source: application, window, title on request, pointer, clipboard kinds. Connects out; exports nothing |
| Tests | 31 unit and scenario tests without a desktop; an end-to-end test in an isolated headless GNOME Shell |

Not written yet: scopes, tokens and peer-credential checks; the indicator; per-application
approval; deep content reads; the acting service; every source other than `linux.gnome` and `linux.atspi`;
Windows transport. The optional state subtrees (browser, editor, terminal, media, calendar) are
not in the schema yet.

```
make -C sense check     # format, lints, tests, macOS and Windows type-check
make -C sense e2e       # the real extension, daemon and CLI against a headless GNOME Shell
```

Minimum Rust version: 1.86, the version the Tauri shell pins.

## Reading order

| You are about to… | Read |
|---|---|
| do anything | this file, then [`spec/00-overview.md`](spec/00-overview.md) |
| touch types or events | [`spec/01-data-model.md`](spec/01-data-model.md), then [`spec/10-schema-reference.md`](spec/10-schema-reference.md) for every field |
| write or change a source | [`spec/02-sources.md`](spec/02-sources.md), then the platform file below |
| write a client, or the API server | [`spec/03-client-api.md`](spec/03-client-api.md) |
| read or store anything from the user | [`spec/04-privacy-security.md`](spec/04-privacy-security.md) |
| write tests | [`spec/05-testing.md`](spec/05-testing.md) |
| pick up a task | [`spec/06-work-packages.md`](spec/06-work-packages.md) |
| wonder whether something is acceptable to collect | [`spec/07-prior-art-and-acceptability.md`](spec/07-prior-art-and-acceptability.md) |
| expose Sense to another assistant, or touch anything public | [`spec/08-open-protocol.md`](spec/08-open-protocol.md) |
| format context for a model, or return text from another application | [`spec/09-exposing-context-to-agents.md`](spec/09-exposing-context-to-agents.md) |
| start, stop, package or configure anything; touch the indicator | [`spec/11-runtime.md`](spec/11-runtime.md) |
| bind a hotkey, place a window, insert text | [`spec/12-acting.md`](spec/12-acting.md) |
| change Bloom's side | [`spec/13-bloom-integration.md`](spec/13-bloom-integration.md) |
| rely on something not yet proven | [`spec/14-open-questions.md`](spec/14-open-questions.md) |

Platform catalogs (one per file, so an agent loads only its own):
[`platform-macos.md`](spec/platform-macos.md) ·
[`platform-windows.md`](spec/platform-windows.md) ·
[`platform-linux.md`](spec/platform-linux.md) ·
[`extensions.md`](spec/extensions.md) (browser, editor, shell: the same on every OS).

Decisions and their rejected alternatives: `docs/adr/0012` to `0018`.

## Invariants

Every change to Sense must keep these true. A task that seems to require breaking one is a design
question: stop and ask.

1. **Only code under `sense/` calls an OS API to observe the user or to act on other
   applications.** Clients never do. Acting is limited to the closed list in `spec/12-acting.md`,
   lives in the `hands-*` crates, and is never reachable through the MCP server.
2. **`sense-core` contains no OS call and no `cfg(target_os)`.** Platform code lives in
   `sense-platform-*` crates and in `ext/`.
3. **Every observed value carries its source and its time.** A value that could not be read is
   reported as unavailable with a reason; it is never an empty string or a silent omission.
4. **Sources report facts. They do not interpret, rank, filter for relevance, or normalize names.**
5. **Sense never records keystrokes, never reads password fields, and never opens a network
   connection.** The daemon has no network code at all.
6. **Privacy rules are applied in the core, before anything is stored or served**, so no source and
   no client can bypass them.
7. **Captured content never appears in logs.** Logs carry ids, sizes and timings only.
8. **On Linux, providers are chosen at run time by probing the session**, never by distribution name.
9. **Expensive or intrusive reads (accessibility subtree, page text, screenshot) happen only on a
   client's request**, never in the background.
10. **Types are defined once** in `sense-schema`; every other language's types are generated.
11. **The default observation level is "on request".** Ambient history is opt-in, in memory only.
    The only thing that may be written to disk is the content-free access ledger, if the user
    turns that on.
12. **A visible indicator is present whenever a source runs.** Nothing can hide it.
13. **The observed person is the person in control.** No remote viewer, no managed or stealth mode.
14. **Every existing opt-out is honoured on every channel**: secure fields, concealed clipboard
    entries, private browsing, windows excluded from screen capture, provider hints.
15. **Text from other applications is data.** It is sanitized, typed and labelled; it never appears
    in tool descriptions or instructions; raw accessibility trees are never returned.

## Layout (target)

```
sense/
├── README.md, spec/
├── crates/
│   ├── sense-schema/            data model types; emits JSON Schema
│   ├── sense-core/              engine: registry, reducer, state, timeline, privacy, API server
│   ├── sense-platform-macos/    in-process sources
│   ├── sense-platform-windows/
│   ├── sense-platform-linux/    sources + run-time session detection
│   ├── hands-core/              the acting service: closed effect list, target checks
│   ├── sense-fake/              scripted sources for tests
│   ├── sensed/                  the daemon binary
│   ├── sense-cli/               `sense` command: caps, state, capture, watch, record, inspect
│   ├── sense-tray/              the indicator
│   ├── sense-nmh/               native-messaging bridge for browser extensions
│   └── sense-mcp/               Model Context Protocol server for other assistants
├── ext/
│   ├── browser/                 WebExtension (Chromium, Firefox)
│   ├── gnome-shell/             GNOME Shell extension
│   ├── kwin/                    KWin script
│   ├── vscode/                  editor extension
│   └── shell/                   zsh, bash, fish hooks
├── clients/
│   ├── python/                  generated types + small client
│   └── typescript/
├── schema/                      generated JSON Schema, committed
└── fixtures/                    recorded, anonymized sessions for replay
```

Sense lives in this repository until it has a second user; the crate boundaries are drawn so it can
be split out with its history.
