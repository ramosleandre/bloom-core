# 0006 · No generic workflow engine in V1

- **Date**: 2026-10-15
- **Status**: proposed (to be accepted by the team in v0.0). Revisits the direction considered in March 2026.
- **Deciders**: A

## Context and problem

In March 2026, the target architecture considered for Bloom included a graph-based (DAG) workflow orchestrator, written in TypeScript on Node, to run branches in parallel. Since then, V1 has been pinned down: a single flow, from hotkey to action.

## Options considered

1. Build a generic workflow engine right away (typed nodes, graph, parallel execution), in Node or in Python.
2. Hard-code the flow in a function (`pipeline.handle_trigger`), and extract an abstraction only when a second real use case requires it.

## Chosen option

2.

## Rationale

- V1 has only one path: a generic engine would have no second use to validate its abstractions.
- A framework is extracted from real cases; invented in advance, it gets a good share of its details wrong.
- A single language in the daemon (Python): fewer bridges, less to learn.

## What changed our mind

The move from a demo project to a bounded V1 (8 actions, one trigger), and the realization that the main risk is decision quality, not orchestration.

## Accepted consequences

- The day Bloom has automatic triggers or a second surface, the pipeline will need refactoring; this is planned for Tech 5.
- No parallel execution of branches in V1. If a measurement shows it is needed, it will be a documented PERF optimization loop.
