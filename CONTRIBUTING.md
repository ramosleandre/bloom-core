# Contributing to Bloom

Everything in this repository is written in English: code, comments, docs, commits, PRs and reviews.

## Branches

- `main` is protected: changes land only through a PR, with green CI and a review.
- One branch per topic, named `<scope>/<topic>`: `decision/laya-adapter`, `surface/observe-mac`.

## Commits

First line: `type(scope): summary`, in English, imperative mood, 72 characters at most.

| | Values |
|---|---|
| Types | `feat` (feature) · `fix` (bug fix) · `test` · `docs` · `refactor` · `lab` (measurements) · `chore` (tooling, dependencies) |
| Scopes | `core` · `decision` · `context` · `surface` · `interface` · `lab` · `ci` · `docs` · `adr` |

- One commit = one coherent change. The message body, optional, explains the why.
- No `Co-Authored-By` lines.
- Examples: `feat(decision): run the Laya adapter locally` · `fix(core): write the audit even if a block crashes` · `docs(adr): 0007 chosen decision backend`

## Pull requests

- Small: one scope, one topic, ideally under 300 lines excluding tests.
- The PR template takes a minute to fill in.
- `make check` locally before pushing; CI runs exactly the same thing.
- If `src/bloom/core/contracts.py` changes: `contract` label, review by the owners of the affected blocks.

## Review

- Nobody merges their own PR.
- A review contains at least one comment. An approval without a comment is not a review.
- What we look at: is the contract respected? Does a test cover the change? Is it as simple as possible? Do the docs follow?
- If the review changes the PR: one line in `docs/evidence/reviews.md`.

## CI

`.github/workflows/ci.yml` runs `make check` then `make bench` on every PR and every push to `main`. When it blocks a broken PR: one line in `docs/evidence/regressions.md`.
