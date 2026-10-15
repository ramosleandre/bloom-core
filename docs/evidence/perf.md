# Performance (PERF)

> Complementary objective D of the Epitech follow-up: go from "it works" to "it holds up, and here is by how much". Choose an axis, measure the starting point, diagnose, change one specific thing, measure again, read the difference. One row per official measurement, including changes that gained nothing.

## Choice note (draft, to finalize before end of December)

**Axis: response time.** A pie menu is only worth something if it appears right away. If it takes one or two seconds, the user is faster doing it by hand and Bloom is useless. It is also our hard problem ([`../hard-problem.md`](../hard-problem.md)): these measurements serve twice.

**Choice quality is the safeguard.** Gaining speed by proposing worse actions is not a gain: `top3` must not drop from one loop to the next.

| Indicator | Formula | Tool | Conditions |
|---|---|---|---|
| `menu_latency` (main) | menu display time minus hotkey press time; p50 and p95 | `RunState.timings_ms` in the audit, script in `lab/bloom/` | reference Mac, 100 runs with the first 5 (warm-up) excluded, fixed scenario |
| `decision_latency` | duration of one call to `Ranker.rank()`; p50 and p95 per backend | `make bench` | same set, 1 warm-up call |
| `top3` (safeguard) | share of situations where an expected action is in the top 3 | `make bench` | frozen set of at least 50 situations |

Also tracked, outside the target: `personalization_lift` (top-1 gain with memory), from v1.0 onwards.

**Target (proposed):** `menu_latency` p95 ≤ 300 ms in May, without `top3` falling below 80%. What we base this on: a menu must feel instantaneous (the classic HCI orders of magnitude put "instantaneous" at around 0.1 s and the break in the flow of thought at around 1 s), and hosted decision models advertise 70 to 500 ms per call. Reaching the target will probably require computing the ranking before the key press or going local: this is a hypothesis to be confirmed by the diagnosis, not a decision.

**Optimization leads**, to be chosen only after the diagnosis: ranking ahead of time on application switch, local backend rather than hosted, caching of context assembly, fewer candidates sent to the model.

## Protocol

Rules and indicators: [`lab/README.md`](../../lab/README.md). End-to-end measurement: [`lab/bloom/README.md`](../../lab/bloom/README.md).

## Measurements

| Date | Commit | Indicator | Value | Conditions | Results file |
|---|---|---|---|---|---|

## Optimization loops

| Loop | Hypothesis, written beforehand | Expected gain | Change (commit) | Before | After | Difference (absolute and %) | Hypothesis confirmed? | Accepted trade-off | Non-regression OK? |
|---|---|---|---|---|---|---|---|---|---|
