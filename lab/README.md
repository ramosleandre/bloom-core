# Lab: everything that gets measured

Kept separate from the product: the lab imports `bloom`, the product never imports the lab. Two families of measurements, present from day one because two of the track's objectives rely on them.

| Folder | Purpose | Track objective |
|---|---|---|
| `decision/` | compare decision backends (rules, Jev, Laya, Von…) on the same set and the same protocol | O1: decision prototype, state of the art |
| `bloom/` | measure Bloom end to end: hotkey → menu latency, gain from personalization | PERF |
| `explore/` | exploration prototype (created in February): hypothesis written beforehand, result kept even if negative | O1 |
| `datasets/` | the annotated situations, their format and their rules | O1, PERF |
| `results/` | the outputs: trial runs in `local/` (ignored by git), official measurements committed | all |

## Rules

1. The protocol is written **before** the measurement: machine, dataset, scenario, number of repetitions, when the measurement is taken.
2. Whatever is compared is measured under the same conditions: same set, same machine, same protocol.
3. A backend is never tuned on the set used to evaluate it. Rules and prompts are written before seeing the evaluation set; otherwise the score no longer measures anything.
4. A measurement cited in a deliverable is official: `make bench out=lab/results/YYYY-MM-DD_name`, then commit. Trial runs stay in `lab/results/local/`.
5. Each official measurement adds a line to `docs/evidence/perf.md`, including when it shows no gain.

## Commands

```bash
make bench                                     # rules, on the sample set
make bench backends=rules,laya data=lab/datasets/situations.jsonl out=lab/results/2027-01-20_decision
```

The bench always prints a **random** line: the score of a random ranking on the same situations. A backend that does not clearly beat it is useless.

## Metrics

| Name | Formula | Tool | Conditions |
|---|---|---|---|
| `top1`, `top3` | share of situations where an expected action is ranked 1st, or in the top 3 | `make bench` | annotated set, same available actions |
| `decision_latency` | duration of a call to `Ranker.rank()`, p50 and p95 | `make bench` | after 1 unmeasured warm-up call |
| `menu_latency` | menu display time minus hotkey press time, p50 and p95 | `RunState.timings_ms` in the audit | reference Mac, protocol in `bloom/README.md` |
| `personalization_lift` | `top1` with memory minus `top1` without memory | `make bench` (v1.0) | situations with user history |

## Reference Mac

To fill in once: model, chip, RAM, macOS version, plugged into AC power, applications open during measurements.
