# 0002 · An internal interface for decision models, one adapter per provider

- **Date**: 2026-09-30
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: A

## Context and problem

"System 1" decision models (Jev, released on September 15, 2026, then Laya, Von and others) address exactly our hard problem: choosing quickly among given options, with probabilities. But the category is less than a month old, the figures are reported by the vendors themselves, and Jev suspended sign-ups one week after its release.

## Options considered

1. Call Jev's SDK directly from Bloom's code.
2. Define an internal interface (`DecisionPort`: a question of type choice, score or yes/no; an answer with probabilities), with one adapter per backend: hosted, local, rules.

## Chosen option

2.

## Rationale

- Avoid depending on a provider in early access, with no service commitment.
- Local alternatives already exist: they avoid sending personal context off the machine (GDPR).
- The decision prototype required by the track compares several backends on the same protocol: they must be interchangeable.

## Accepted consequences

- One more layer, and a lowest-common-denominator interface: we give up provider-specific features.
- One adapter to maintain per backend.
- Rule enforced by CI: no model SDK is imported anywhere other than `src/bloom/decision/adapters/`.
