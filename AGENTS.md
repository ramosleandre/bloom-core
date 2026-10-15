# AGENTS.md: Bloom

Instructions for AI agents (Claude Code, Codex, Cursor…) and for anyone who touches the code.

## In short

Bloom: on macOS, a hotkey opens a pie menu that suggests the three most useful actions for what is on screen; the chosen action runs, with human approval if it is risky. Overview: [`docs/architecture.md`](docs/architecture.md). Progress: [`docs/roadmap.md`](docs/roadmap.md).

## Commands

| Command | Effect |
|---|---|
| `make setup` | installs everything (uv) |
| `make check` | format, lint, types, architecture rules, tests: **what CI requires** |
| `make fix` | fixes formatting and lint automatically |
| `make test b=decision` | tests for one block (without `b=`: all) |
| `make bench` | decision bench → `lab/results/local/` |
| `make run` | a demo run, mocked blocks |
| `make quality` | dated quality report |

Before saying "done": `make check` passes.

## Code map

| Part | Folder | Role |
|---|---|---|
| Core | `src/bloom/core/` | contracts (`contracts.py`), safety rules (`policy.py`), the run (`pipeline.py`), audit |
| A · Decision | `src/bloom/decision/` | ranks actions, assesses risk; one file per model backend in `adapters/` |
| B · Context | `src/bloom/context/` | assembles the context, labeled by source, sensitivity and trust |
| C · Surface | `src/bloom/surface/` | reads the macOS screen, provides the action catalog, prepares and executes actions |
| D · Interface | `src/bloom/interface/` + `shell/` | bridge to the Tauri shell: hotkey, pie menu, approval |
| Composition | `src/bloom/app.py` | the only place that wires the blocks together |
| Lab | `lab/` | measurements: decision models, performance |

## Rules

Checked by `make check` where possible.

1. Blocks never import each other. From the core, they only import `bloom.core.contracts` and `bloom.core.policy`.
2. The core imports no block. Only `bloom/app.py` wires them.
3. The core and the context block never call the network or a model SDK. Model SDKs live only in `src/bloom/decision/adapters/`.
4. Criticality floor: a model can raise an action's risk level, never lower it. An `EXTERNAL` action always requires approval.
5. Changing `src/bloom/core/contracts.py`: PR with the `contract` label, reviewed by the owners of the affected blocks.
6. No secrets in the repo (local `.env`, see `.env.example`). No real personal data committed.
7. Everything is written in English: code, comments, docs, commit messages.

## Keep it simple

- No generic abstraction without two real uses in the code.
- No new dependency without an ADR (`docs/adr/`).
- One PR = one scope = one topic. If the task touches two blocks, it is probably a contract change: stop and ask.
- No optimization without a measurement before and after (`lab/`).

## Definition of "done"

- [ ] `make check` green
- [ ] a test covers the added or changed behavior
- [ ] `docs/architecture.md` up to date if the structure or a contract changed
- [ ] the item in `docs/roadmap.md` is checked
- [ ] the evidence is recorded in `docs/evidence/` if the change produces some (table below)

## Where to record what

| What happened | Write in |
|---|---|
| a structural decision (technology, split, format, abandonment) | a new ADR `docs/adr/NNNN-title.md` |
| a test caught a regression, or CI blocked a PR | `docs/evidence/regressions.md` |
| a review changed a PR | `docs/evidence/reviews.md` |
| an accepted debt | `docs/evidence/tech-debt.md` |
| an official measurement | `lab/results/YYYY-MM-DD_name/` then `docs/evidence/perf.md` |
| a tech watch source read, with a conclusion | `docs/evidence/research-log.md` |

Commits, PRs and review: [`CONTRIBUTING.md`](CONTRIBUTING.md). Never add `Co-Authored-By` lines to commit messages.
