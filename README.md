# Bloom

One hotkey, a pie menu, the right action. Bloom looks at what you are doing on macOS, suggests the three most useful actions and runs them, asking for your approval when it is risky.

Epitech EIP project (Technical Track, 2026-27). Status: **v0.0, foundation**. The core runs; the blocks are still mocked. Detailed progress: [`docs/roadmap.md`](docs/roadmap.md).

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
shell/                block D: Tauri app (from v0.2)
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
