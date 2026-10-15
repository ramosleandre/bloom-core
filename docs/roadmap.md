# Roadmap

> Source of truth for Bloom's progress. If a kanban exists, it copies this file, and this file wins in case of disagreement.
> A checked box = the definition of "done" in `AGENTS.md` is met.
> Tags: who `[A]` `[B]` `[C]` `[D]` `[All]`; evidence for the track `{O1}` `{O2}` `{PERF}` `{EXP}`. Track deadlines are in bold; evidence is filed in [`evidence/`](evidence/README.md).

## Where we are

- Current version: **v0.0, foundation**
- Next track deadline: **end of October**, the hard-problem note and the tech watch setup {O1}
- Last updated: 2026-10-15

## The principle

A vertical slice that works, then we measure, then we secure, then we integrate the best measured solution. A version does not start until the previous one's demo runs.

---

## v0.0 · Foundation (October 15 → 31)

Goal: everyone can code, CI guards the door, both October deliverables go out.

- [x] [A] Repository skeleton: core, mocks of the 4 blocks, tests, decision bench, CI, knowledge base
- [ ] [A] GitHub repository created, `main` protected, CI active {O2}
- [ ] [All] `make setup`, `make check` and `make run` green on all 4 machines
- [ ] [All] Review ADRs 0001 to 0006, at least one comment each, then move them to "accepted" {O2}
- [ ] [All] The reference use case written in one sentence, and the 8 actions listed with their floor
- [ ] [A] Measure the naive approach (a generative LLM ranks the 8 actions): 30 calls, p50 and p95 {O1}
- [ ] [A] Finalize [`hard-problem.md`](hard-problem.md) with quantified thresholds, send it to the mentor {O1}
- [ ] [All] Finalize [`evidence/research-setup.md`](evidence/research-setup.md): sources, tools in place, cadence, split of work {O1}
- [ ] [All] A first tech watch entry each in `evidence/research-log.md` {O1}
- [ ] [C] Test the Accessibility permission from a Python sidecar launched by Tauri (risk #1 of the architecture)

**End demo**: `make check` and `make run` pass for everyone; the mentor has received the hard-problem note.

## v0.1 · The core runs on fake blocks (November 1 → 15)

Goal: everyone can develop their block alone, against the others' mocks.

- [ ] [A] `contracts.py` v1 frozen, reviewed by all 4 (label `contract`)
- [ ] [B] [C] [D] Each block has its mock, faithful to the contract, and its conformance tests
- [ ] [A] A single active run at a time: a new hotkey cancels the previous one
- [ ] [A] `RunFinished` and `Feedback` events, added when their first consumer exists
- [ ] [All] Internal sharing ritual in place, first presentation logged {O1}
- [ ] [All] First review that changes a PR, logged in `evidence/reviews.md` {O2}

**End demo**: the integration test covers the four outcomes of a run, and each block runs alone against the mocks.

## v0.2 · Real vertical slice (November 16 → December 20)

Goal: on the reference Mac, hotkey, menu, action executed, for real.

- [ ] [C] Real `observe()` on macOS: application, window, selection
- [ ] [C] The 8 actions with their floor; real `execute()` for at least 3, including one `EXTERNAL`
- [ ] [B] Real context: device and short-term fragments, labeled, trimmed to budget, no network
- [ ] [A] A System 1 backend (Jev or Laya) behind `DecisionPort`, alongside the rules
- [ ] [A] `generate()` (LLM) wired up for an action that writes
- [ ] [D] Tauri shell: global hotkey, pie menu (fixed positions + contextual slots), approval window
- [ ] [D] IPC bridge (local WebSocket, token, Origin) and TypeScript types generated from the contracts
- [ ] [A] Architecture diagram and data diagram up to date in `architecture.md` · **end of November** {O2}
- [ ] [A] [All] State-of-the-art note: at least 3 approaches compared, dated sources · **end of December** {O1}
- [ ] [D] README, API doc (IPC messages) and operations doc, first version · **end of December** {O2}
- [ ] [A] Baseline quality report: `make quality` · **December** {O2}
- [ ] [A] [D] PERF choice note: axis, indicators, target · **end of December** {PERF}
- [ ] [A] Note on the 2 or 3 topics to have challenged by experts · **end of December** {EXP}

**End demo**: the slice runs on the reference Mac.
**Go / no-go**: on 20 real situations, is the intended action in the top 3 at least 7 times out of 10? If not, we rethink the approach before going further.

## v0.3 · Measured decision (January → February)

Goal: choose the decision backend based on measurements, not on a preference.

- [ ] [All] A set of at least 50 real situations, annotated and anonymized (`lab/datasets/`)
- [ ] [A] Decision prototype: at least 2 backends, same set, same protocol, measurements side by side · **end of January** {O1}
- [ ] [A] [D] PERF measurement protocol written and tooled (hotkey → menu latency) · **end of January** {PERF}
- [ ] [A] Technical decision (ADR): chosen backend, rejected approaches, consequences · **end of February** {O1}
- [ ] [A] [D] Baseline measurement and tooled diagnosis: where it gets stuck · **end of February** {PERF}
- [ ] [B] Exploration prototype launched, hypothesis written beforehand · **February → April** {O1}
- [ ] [A] Expert outreach: at least 6 contacts between January and April, follow-ups included {EXP}

**End demo**: the backend comparison table, produced by `make bench`.

## v0.4 · Trust (March)

Goal: nothing risky goes through without the user, and we prove it.

- [ ] [A] Approval that expires, serializable run state
- [ ] [B] Routing by sensitivity under real conditions (`LOCAL_ONLY` fragments)
- [ ] [C] Undo (`undo`) of `REVERSIBLE` actions
- [ ] [All] Adversarial set: an injection in an email triggers nothing without approval
- [ ] [C] Critical path covered by tests, coverage rate published · **end of March** {O2}
- [ ] [All] At least 3 regressions caught and documented · **end of March** {O2}
- [ ] [D] Reproducibility test by someone outside the team · **end of March** {O2}
- [ ] [A] [D] Optimization loop 1: one change, a new measurement, the difference · **end of March** {PERF}
- [ ] [A] First exchange with an expert (brief sent 48 h before, minutes within 48 h) · **end of March** {EXP}

## v1.0 · Greenlight (April → May)

Goal: the chosen solution is integrated and measured, and every objective has its evidence.

- [ ] [A] Chosen solution integrated on `main`, before / after measurement on the October criteria · **end of April** {O1}
- [ ] [B] Memory: `remember()` and measured personalization (top-1 gain with vs. without memory)
- [ ] [B] Exploration prototype concluded, even if negative · **April** {O1}
- [ ] [A] [D] Optimization loop 2 · **end of May** {PERF}
- [ ] [A] Second exchange with an expert, end-of-year memo · **end of May** {EXP}
- [ ] [A] Final quality report `make quality` · **May** {O2}
- [ ] [All] At least 5 public contributions, including one external · **May** {O1}
- [ ] [All] Jury rehearsal: scripted demo, all evidence in `evidence/` up to date

**End demo**: the Greenlight jury.

## Later (Tech 5, 2027-28)

None of this before the Greenlight.

- long-term memory that restructures itself (the research contribution)
- second surface (the browser): this is where we will extract the generic abstractions
- automatic triggers (scheduling, webhook), hence runs that are suspended and resumed
- MCP connectors
- more actions, extensible catalog

## Parking

Ideas proposed, deliberately not done. Format: date · idea · who · why not now.

- 2026-10-15 · generic workflow engine (DAG) · A · not before two real use cases (ADR 0006)
- 2026-10-15 · mobile, AR, voice · All · out of V1 scope
