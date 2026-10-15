# End-to-end measurements (PERF)

Status: to be built in v0.3 (protocol written before the end of January). The basic tooling already exists: the core times each step of a run and writes it to `RunState.timings_ms`, and therefore into every line of `.bloom/audit.jsonl`:

`observe`, `assemble`, `rank`, `menu_latency`, `resolve`, `assess`, `execute`

This is what will make the diagnosis required by PERF ("where is the bottleneck?") possible without guesswork.

## Protocol (to complete before the first official measurement)

- **Machine**: the reference Mac (`lab/README.md`)
- **Scenario**: <the situations played, for example 10 situations × 10 repetitions>
- **Repetitions**: 100 runs; the first 5 excluded (warm-up)
- **Decision backend**: <name and version>
- **When**: <for example after a restart, with only the scenario's applications open>
- **Computation**: p50 and p95 of `menu_latency` over the valid runs; same computation per step for the diagnosis

`menu_latency` compares two clock readings: the press (taken by the shell) and the display (also taken by the shell). Both come from the same process, so the measurement does not depend on the clock offset between the shell and the daemon.

## Script

`latency.py` (to be written in v0.3): reads the audit, keeps the measurement window, computes p50 and p95 per step, writes the result to `lab/results/YYYY-MM-DD_latency/`.
