# 0013 · Read context on request by default; ambient awareness is opt-in; never record

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

The current design reads the screen once, when the hotkey is pressed, from a process that starts
cold. Two mechanical problems: opening Bloom's menu changes the focus it wants to read, and every
source's latency is paid at the moment the menu must appear. A third limit: operating systems
expose much context only as events (focus changed, clipboard changed, tab activated), so the recent
past cannot be queried afterwards.

The opposite extreme, recording everything continuously, exists in the market and was badly
received: Microsoft Recall was withdrawn and reissued as opt-in with heavy safeguards, and
applications such as Signal block it. Products that read only when the user asks (ChatGPT's "Work
with Apps", Android's Assist API, Apple's onscreen awareness) are accepted. Local trackers of
activity metadata (ActivityWatch) are accepted when self-installed. Review:
[`sense/spec/07-prior-art-and-acceptability.md`](../../sense/spec/07-prior-art-and-acceptability.md).

## Options considered

1. Pull everything at the hotkey from a cold start.
2. Record continuously, content included, into a searchable history.
3. A resident engine with observation levels chosen by the user:
   - level 1, **on request** (default): the engine only tracks which application and window are in
     front; everything else is read when the user invokes a client, and not retained;
   - level 2, **session awareness** (opt-in): ambient metadata (titles, tabs, selection and
     clipboard events, files, commands) kept in a bounded in-memory timeline;
   - at every level, content in depth (document text, accessibility subtree, pixels) only on an
     explicit request, and no observed context written to disk.

## Chosen option

3.

## Rationale

- A resident engine already knows the target that was in focus before Bloom's window appeared, and
  ignores its clients' own windows. The ordering problem is solved on every OS, at level 1 already.
- Sources are warm (connections open, permissions checked), so a capture fits the menu's latency
  budget instead of paying start-up costs.
- The default matches what users already accept from comparable assistants. The recent past, a
  strong signal for ranking and memory, is available to those who opt in, as metadata only.
- Option 2 is rejected: it collects far more than any action needs, burdens other applications
  (accessibility trees, screen capture), creates a stored history that is a liability, and is the
  design the public rejected.

## Accepted consequences

- At the default level there is no timeline: features that rely on recent activity must degrade
  cleanly and say what enabling level 2 would add.
- Every source needs two paths: watch, and report once on request.
- A process is resident for the session, which requires the privacy model of
  [ADR 0016](0016-privacy-model-of-the-context-engine.md), including a persistent visible
  indicator.
- Resource budgets must be held and measured: idle CPU and memory of the daemon.
- Long-term memory remains the context block's responsibility, fed by subscription, under its own
  consent and retention rules.
