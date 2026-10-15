# 0003 · A core and four blocks in a star, exchanging only typed contracts

- **Date**: 2026-10-11
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: Everyone

## Context and problem

Four part-time people must develop in parallel without waiting on each other, and the result must remain testable end to end.

## Options considered

1. Blocks that call each other directly.
2. Route everything through an event bus.
3. A star: a core that orchestrates the run and calls each block; the blocks exchange only types defined in `contracts.py`.

## Chosen option

3.

## Rationale

- Everyone codes against the mocks of the other three blocks from the first week.
- The full run is tested in a single integration test.
- The rules are checked automatically (import-linter): a block that imports another one breaks CI.
- A bus alone makes the order of steps implicit, and therefore hard to follow and to test for a team new to the project.

## Accepted consequences

- The core is a mandatory chokepoint, owned by A.
- Changing a contract costs a review by the owners concerned.
- A bit of indirection: when an action needs to write text, block C receives block A's generator by injection instead of importing it.
