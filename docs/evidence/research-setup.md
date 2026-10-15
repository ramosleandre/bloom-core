# Tech watch setup (O1)

> O1 deliverable to be described **before end of October 2026**, then maintained until May.
> Status: draft of 2026-10-15. The guide requires a setup that is "described and demonstrable, not just asserted": every ticked tool must be shown in a follow-up session.

## Who follows what

| Area | Who | Why this area |
|---|---|---|
| System 1 decision models, evaluation, calibration | A | it is the hard problem |
| Agent memory and context (mem0, Letta…) | B | their block, and the research direction |
| macOS automation, agent security (injection), MCP | C | their block, and the risks |
| Radial menu UX, Tauri, perceived latency | D | their block, and the PERF axis |

## Sources

| Source | Type | Who | What it provides | Why we keep it |
|---|---|---|---|---|
| `NandhaKishorM/laya` repository (releases) | repository | A | open-source, local System 1 model | candidate for the decision prototype |
| `wfzyx/von` repository (releases) | repository | A | System 1 model on CPU | another local candidate |
| `unikcc/awesome-system1-decision-models` list | list | A | survey of the category | not missing a release |
| TypeSafe (Jev) documentation and blog | official docs | A | API, pricing, limits | candidate hosted backend |
| arXiv alerts (decision model, calibration, agent memory, prompt injection) | papers | A, B, C | published methods | dated state of the art |
| mem0 and Letta repositories (releases) | repositories | B | state of the art in agent memory | comparing ourselves to existing work |
| OWASP Top 10 for LLM Applications | security reference | C | catalog of attacks, including injection | checklist for the adversarial set |
| Apple Accessibility documentation + pyobjc | official docs | C | reading the screen across applications | the architecture's #1 risk |
| MCP specification (releases) | spec | C | V2 connectors | not reinventing connectors |
| Tauri (releases, documentation) | official docs | D | the desktop shell | direct dependency |
| HCI literature on pie menus and marking menus | papers | D | what makes a radial menu effective | well-argued UX decisions |

A source that has produced no entry in two months is dropped from the list; the removal is recorded in the log.

## Tools and automation

- [ ] Discord channel `#tech-watch`, fed automatically by an RSS bot (arXiv feeds, blogs, releases)
- [ ] "Watch → Custom → Releases" on GitHub for each followed repository
- [ ] Shared notes base: [`research-log.md`](research-log.md), in the repository, reviewed in PRs
- [ ] With each report, a screenshot of the channel and the alerts, as evidence that the setup is running

## Cadence

- 30 minutes per week per person
- at least one log entry every two weeks per person (about sixty over the year, for a minimum of 20)
- one internal presentation per month from November, in rotation ([`presentations.md`](presentations.md))

## What counts as a valid entry

Date, source, topic, what it changes or could change for Bloom, and **what we did about it, or why we did nothing**. An entry without a conclusion does not count.
