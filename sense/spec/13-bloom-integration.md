# 13 · How Bloom uses Sense

Sense does not depend on Bloom. This file is the other direction: what changes in Bloom if the team
accepts ADRs 0012 to 0018. Nothing here has been applied; `contracts.py`, `architecture.md` and the
roadmap are untouched. It is the material for one `contract` pull request and one roadmap update.

## Existing work this must build on

Block D's prototype (branch `feature/tauri-ui`, not merged at the time of writing) already
contains working code in the same territory. This proposal must extend it, not replace it.

| Already there | Relation to this proposal |
|---|---|
| A Tauri shell with the pie overlay, approval window and a WebSocket bridge to the Python daemon | Unchanged. It is the first-party client described below. |
| A GNOME Shell extension (`bloom-desktop@bloom.local`, GNOME 46) giving the pointer position and a Ctrl+Space hotkey over a session D-Bus service, with single-owner and release-on-disconnect rules | This *is* the hotkey and pointer part of the acting service on GNOME, already built and tested. Keep it. |
| Menu placement on GNOME by running the overlay through XWayland | Better than what this proposal first specified (moving the window from the extension). Adopted in `12-acting.md`. |
| Global hotkey and pointer on X11 (`tauri-plugin-global-shortcut`, `x11rb`) | Same mechanisms as specified; no change needed. |
| Rust pinned to 1.86; CI jobs for the shell and the Tauri crate | `sense/` declares the same minimum Rust version and keeps its build and CI in separate files, so the two do not conflict. |

What the prototype does not cover, and Sense adds: which application and window are in front,
their title, the selection, the document, the clipboard, inserting text into the target, the
privacy rules, and everything on macOS.

Two extensions for now, one later. Sense's extension (`sense@bloom.local`) uses a different id,
binds no shortcut and exports nothing on D-Bus, so both can be installed together without
interfering. Merging them is a decision for the team (D9 in `14-open-questions.md`). The point to
settle is the transport: a D-Bus service is fine for a pointer position and a hotkey; once the
extension can also return window titles and clipboard content, or paste, any process in the
session could ask it the same, which is why Sense's extension connects out to a user-only socket.

## Processes

```
            ┌────────────────────┐   launches, passes session token   ┌──────────────────────┐
 hotkey ──▶ │ Tauri shell (D)    │ ──────────────────────────────────▶│ Python daemon         │
            │ menu, approval     │◀──────── WebSocket IPC (as today) ─│ core, A, B, C         │
            └─────────┬──────────┘                                    └──────────┬───────────┘
                      │ act.hotkey.*, act.window.place                           │ capture, content.read,
                      │                                                          │ act.insert_text
                      ▼                                                          ▼
            ┌───────────────────────────────────────────────────────────────────────────────┐
            │ sensed (Sense + acting service) ── sources, extensions, indicator             │
            └───────────────────────────────────────────────────────────────────────────────┘
```

- The shell starts `sensed` first, then the Python daemon. It is the parent of both.
- **Session token.** A scope keyed by executable path cannot identify the Python daemon (its
  executable is a generic interpreter). So `sensed`, when started with `--parent-pid`, hands its
  parent a one-time first-party token over an inherited pipe; the shell passes it to the Python
  daemon in its environment. A connection presenting it gets the first-party scopes, including
  `act`. The token dies with the daemon.
- Both clients register the shell's process id in `own_pids`, so Bloom's menu never becomes "what
  the user is doing".
- Who calls what: the shell (block D) owns the hotkey and the menu window; the Python surface
  block (C) owns reading and inserting. Blocks still never talk to each other; both talk to
  `sensed`, as they would to the OS.

## The run, revised

| Step | Today (planned) | With Sense |
|---|---|---|
| hotkey | Tauri global-shortcut plugin | `act.hotkey.pressed {at}` → `Trigger.at` |
| observe | Python calls AT-SPI / macOS accessibility | `Surface.observe` = `capture { want: focus, selection, document; deadline_ms: 50 }` |
| menu | Tauri window at the cursor | shell shows its window; `act.window.place { at: Pointer }` where the session needs it |
| assemble | B builds fragments from `Observation` | unchanged; B reads provenance and sensitivity from the richer `Observation` |
| available | catalog | catalog filtered by each action's `requires` against what the observation holds |
| resolve | fills arguments | may call `content.read` for the action's `wants` (surrounding text, document) |
| execute | Python calls the OS | `act.insert_text` / `act.clipboard.set`; ordinary library calls for the rest |

`Surface`'s four methods keep their signatures. The core, the policy and blocks A and D's Python
side do not change.

## Contract changes (for the `contract` pull request)

Bloom's contracts stay Bloom-shaped: small, Python-first. They do not mirror Sense's schema; the
surface block maps one to the other in a single file.

```python
class Seen(Frozen):
    """A value read from the user's desktop, with how it was obtained."""

    text: str
    via: str  # Sense source id, e.g. "ext.browser"
    quality: Literal["exact", "ambient", "inferred"]
    sensitivity: Literal["system", "personal", "content"]
    redacted: bool = False
    truncated: bool = False


class AppRef(Frozen):
    raw_id: str  # what the OS reports
    name: str
    canonical: str | None = None  # "firefox", same on every OS
    category: str = "other"  # browser, mail, editor, terminal, …


class Observation(Frozen):
    trigger_id: UUID
    captured_at: datetime
    platform: str  # "linux/gnome", "macos", "windows"
    app: AppRef | None = None
    window_title: Seen | None = None
    selection: Seen | None = None  # Seen(text="") = nothing selected
    url: Seen | None = None
    path: Seen | None = None
    focus_role: str | None = None  # "text_field", "document", …
    focus_editable: bool | None = None
    excluded: bool = False  # the user or the application opted out
    missing: dict[str, str] = {}  # facet → reason: "selection": "app_not_accessible"
    target: str | None = None  # opaque handle for resolve and execute
    timings_ms: dict[str, int] = {}


class ActionDescriptor(Frozen):
    ...  # existing fields
    requires: list[str] = []  # facets without which the action is not offered
    wants: list[str] = []  # facets resolve() will try to read in depth
```

Replaced fields: `app_bundle_id` → `app.raw_id` (and `app.canonical` for the decision model, which
should not learn three names for one application); `selected_text` → `selection`; `url` keeps its
name and gains provenance.

`None` versus `missing`: a facet that is `None` **and** listed in `missing` could not be read; a
facet that is `None` and not listed does not apply. This keeps "nothing selected" and "could not
tell" apart, which the menu needs.

### Mapping, snapshot → `Observation`

| `Observation` | From the snapshot |
|---|---|
| `platform` | `session.os` (+ `/` + `session.desktop` on Linux) |
| `app` | `focus.app`: `raw_id`, `name`, `canonical`, `category` |
| `window_title` | `focus.window.title` |
| `selection` | `selection.text` |
| `url`, `path` | `document.url`, `document.path` |
| `focus_role`, `focus_editable` | `focus.element.role`, `.editable` |
| `excluded` | any `excluded` on the focused app, window or document |
| `missing` | every `{"unavailable": reason}` among the fields above, except `not_applicable` |
| `target`, `timings_ms` | same names |

### Mapping for block B (fragments)

| Fragment source | From | `trust` | `sensitivity` (default; B's policy may raise it) |
|---|---|---|---|
| `device.app` | `app.canonical` or `name` | `SYSTEM` | `PUBLIC` |
| `device.window` | `window_title` | `UNTRUSTED` (titles often carry third-party text) | `PERSONAL` |
| `device.selection` | `selection` | `UNTRUSTED` | `PERSONAL`; `LOCAL_ONLY` if `redacted` |
| `device.url` | `url` | `UNTRUSTED` | `PERSONAL` |
| `device.path` | `path` | `SYSTEM` | `PERSONAL` |

A selection of `ambient` quality should be given a lower confidence by B and described as possibly
stale; that is the reason provenance crosses the contract.

## Code changes in Bloom

| Where | Change |
|---|---|
| `src/bloom/core/contracts.py` | the types above |
| `src/bloom/surface/sense.py` (new) | `SenseSurface`: the `Surface` facade over the Python client; the mapping table above, in one function |
| `src/bloom/surface/replay.py` (new) | `ReplaySurface`: the same mapping over recorded snapshots, for tests and the bench |
| `src/bloom/surface/mock.py` | updated to the new `Observation`; scenes for Linux and macOS ids |
| `src/bloom/surface/catalog.py`, `actions/` | `requires` / `wants` on each action; `available()` filters on them |
| `src/bloom/context/` | fragments from `Seen` values |
| `src/bloom/decision/adapters/rules.py`, `lab/datasets/` | use `app.canonical` and `category`, not macOS bundle ids |
| `src/bloom/app.py` | choose `SenseSurface` when the token is present, the mock otherwise |
| `shell/` | start `sensed`, pass the token, bind the hotkey and place the window through `act.*` |
| `pyproject.toml` | the generated Sense client as a dependency of the surface block only |
| import rules | unchanged: the surface block still imports only contracts and policy from the core |

## Documents to update once accepted

- `docs/architecture.md`: three processes; block C "observes through Sense"; the failure table
  (permission revoked, extension missing, daemon down → degraded menu from `missing`).
- ADR 0004: superseded in part by 0012 (plan B is no longer needed).
- ADR 0007: amended by 0015.
- `AGENTS.md` code map: add `sense/`; rule "only `sense/` observes or acts on the OS".
- The surface and interface block sheets in the team's knowledge base.

## Effect on the roadmap (proposal, for the team to decide)

| Roadmap item | Becomes |
|---|---|
| v0.0 [C] test screen reading on Linux; [A] test the macOS permission from a Python sidecar | the verification spike (`14-open-questions.md`) |
| v0.1 [A] `contracts.py` v1 frozen | includes the `Observation` change above |
| v0.2 [C] real `observe()` on Linux (AT-SPI) | Sense stage 0, then `linux.gnome` and `linux.x11` |
| v0.2 [A][C] real `observe()` on macOS | Sense `macos.workspace` + `macos.ax`, on the team's Mac |
| v0.2 [C] real `execute()` for 3 actions on both systems | acting service: `insert_text` on GNOME and macOS |
| v0.2 [D] Tauri shell: global hotkey, pie menu | hotkey and placement through `act.*` |
| "Later: second surface (the browser)" | moves earlier: the browser extension is part of the minimum slice |

What the track gets from it: an architecture decision with measured alternatives (O2), a hard
cross-platform problem with a spike and written conclusions (O1), public contributions (the
extensions), and latency numbers per source for PERF (`timings_ms` is already in the snapshot).

## The mock stays first-class

Everything above is behind the `Surface` facade. `make run` and `make check` must keep working
with no daemon, no Rust toolchain and no desktop, exactly as today. A team member who never
touches Sense is not slowed down by it.
