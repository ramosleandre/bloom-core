# 0001 · A new repository rather than evolving HostaAgent

- **Date**: 2026-09-30
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: A

## Context and problem

Bloom needs a foundation for the core and the blocks. An existing team project, HostaAgent (a minimal ReAct agent framework, MIT license, about 2,100 lines), contains reusable pieces: events, permissions, separation between the engine, the environment and the launch mode.

## Options considered

1. Evolve HostaAgent until it becomes Bloom's foundation.
2. Create a new repository and carry over only its decomposition principles.

## Chosen option

2.

## Rationale

- HostaAgent's thesis ("the loop never grows") runs counter to Bloom's (a chain of decisions of different kinds).
- HostaAgent relies on an LLM library (OpenHosta) that does not allow plugging in a decision model.
- A large part of its code (an 882-line command-line interface, a configuration registry) has no use in Bloom.

## Accepted consequences

- We rewrite the tests and lose the git history of these pieces.
- We reimplement the useful parts (events, permissions) only when a first real use appears.
