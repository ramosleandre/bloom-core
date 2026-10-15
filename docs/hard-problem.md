# Hard problem: characterization note (O1)

> Deliverable O1, to be approved by the mentor **before the end of October 2026**.
> Status: draft of 2026-10-15. The thresholds are proposals to be defended in session; measuring the naive approach is still to be done.

## Problem statement

When the user presses the hotkey, Bloom must display, almost instantly, the three most relevant actions for what is on their screen, then reliably decide whether the chosen action requires their approval. Today, each of these small decisions goes through a call to a generative LLM: too slow for a menu, and the result is neither stable nor quantified.

## What makes it hard

- **Time.** A menu must appear within a few hundred milliseconds at most. Beyond about one second, the user loses the thread and is faster doing it by hand (classic HCI orders of magnitude: 0.1 s feels instantaneous, 1 s is the limit for the flow of thought).
- **Reliability.** A bad suggestion costs trust; an underestimated risk lets a dangerous action through.
- **Privacy.** The context (emails, documents) is personal. Sending it to an API hosted outside the EU raises a GDPR issue: deciding locally must be possible.
- **Resources.** Locally, it has to run on a laptop, without a dedicated graphics card.

## What the naive approach yields

The obvious approach: one call to a hosted generative LLM, with the context and the list of 8 actions, which returns the best three as JSON.

- **Measured latency: to do before the end of October.** 30 calls, same prompt, same machine; p50 and p95. (Roadmap task v0.0 [A].)
- **What is wrong, even without the measurement:** free-form output to parse, no usable probabilities to set thresholds, a cost on every call, and the context sent off the machine.

## Success criteria (proposed)

| Criterion | Threshold | Measured by |
|---|---|---|
| Latency hotkey → menu displayed | p95 ≤ 300 ms on the reference Mac | `RunState.timings_ms["menu_latency"]`, `lab/bloom/` protocol |
| Relevance | an intended action in the top 3 for at least 80% of a set of at least 50 annotated real situations | `make bench` |
| Safety | 100% of the `EXTERNAL` or `DESTRUCTIVE` actions in the adversarial set go through an approval | tests, in `make check` |
| Privacy | 100% of decisions on a `LOCAL_ONLY` context made by a local backend | tests, in `make check` |

## Result (to be filled in at the end of April)

For each criterion: the value before, the value after, the difference, what the integration replaced, the known limitations.
