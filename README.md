# Bloom

One hotkey, a pie menu, the right action. Bloom looks at what you are doing on macOS, suggests the three most useful actions and runs them, asking for your approval when it is risky.

Epitech EIP project (Technical Track, 2026-27). Status: **v0.0, foundation**. The core and
interactive UI run; context capture and actions are still mocked, with rule-based decisions.
Detailed progress: [`docs/roadmap.md`](docs/roadmap.md).

## Requirements

- macOS for the full product. Linux or macOS is enough for the core, the tests and the measurement bench.
- [uv](https://docs.astral.sh/uv/): it also installs the right Python version (3.13).
- `make`

## Installation

```bash
make setup     # dependencies
make check     # everything must be green
make run       # a demo run
```

## Interactive interface

Install Node.js 20.19+, npm, Rust/Cargo 1.86+ and the native development libraries described
in [`shell/README.md`](shell/README.md), then run:

```bash
make ui-setup
make ui
```

The Tauri app opens an activity window connected to the Python demo. Click **Start a demo**,
choose an action in the pie menu, then approve or reject if requested. Your choice controls
the actual pipeline; context and action execution are simulated. The interface also has
history, run explanations, a pie preview and local preferences.

The native pie is a transparent radial overlay. Ctrl+Space opens it around the cursor on
X11; macOS uses Command+Shift+Space (not yet tested on a Mac). Native Wayland currently uses
compositor-controlled placement and an in-app shortcut. To test the XWayland path, run
`GDK_BACKEND=x11 make ui`; global shortcuts there depend on the compositor.
GNOME Wayland has an optional desktop adapter: run `make gnome-setup` once, follow any
logout/login instruction, then use `make ui-gnome`. See [GNOME setup](shell/gnome/README.md).

`make desktop` is an alias for the desktop workflow. UI edits reload inside the app through
Vite. `make ui-preview` opens an optional browser preview. The native Linux app has been
compiled and smoke-tested with the main window, floating pie menu and approval flow.
Setup, component boundaries, IPC and checks: [`shell/README.md`](shell/README.md).

## Configuration

Copy `.env.example` to `.env`. No variable is required to get started.

| Variable | Role | Default |
|---|---|---|
| `BLOOM_DECISION_BACKENDS` | decision backends, in order of preference | `rules` |
| `BLOOM_AUDIT_PATH` | local run log (personal data, never committed) | `.bloom/audit.jsonl` |
| `JEV_API_KEY`, `LLM_API_KEY` | hosted model keys (from v0.2) | empty |

## Commands

`make help` lists them all: `setup`, `check`, `fix`, `test [b=block]`, `bench`, `run`, `quality`.

## Structure

```
src/bloom/core/       the core: contracts, safety rules, run pipeline, audit
src/bloom/decision/   block A: action ranking, risk, generation
src/bloom/context/    block B: context and memory
src/bloom/surface/    block C: screen reading, action catalog and execution
src/bloom/interface/  block D: bridge to the shell
src/bloom/app.py      wires the blocks together
shell/                block D: interactive UI and Tauri development wrapper
tests/                one folder per block, plus integration/ (the critical path)
lab/                  measurements: decision models, performance
docs/                 architecture, hard problem, roadmap, decisions, evidence
```

## Tests

`make test` runs everything, `make test b=decision` a single block. `tests/integration/` covers the critical path: a full run, with its four possible outcomes (executed, cancelled, rejected, failed).

## Documentation

- [`docs/architecture.md`](docs/architecture.md): diagrams, the core and the four blocks, contracts, safety rules, data, failure modes
- [`docs/hard-problem.md`](docs/hard-problem.md): the hard problem and its quantified success criteria
- [`docs/roadmap.md`](docs/roadmap.md): progress, version by version
- [`docs/adr/`](docs/adr/): architecture decisions
- [`docs/evidence/`](docs/evidence/README.md): evidence for the Epitech follow-up (tech watch, reviews, regressions, measurements…)
- [`AGENTS.md`](AGENTS.md): instructions for AI agents · [`CONTRIBUTING.md`](CONTRIBUTING.md): commits, PRs, review
