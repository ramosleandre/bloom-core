# 0005 · Quality tooling: uv, ruff, pyright, pytest, import-linter, GitHub Actions

- **Date**: 2026-10-15
- **Status**: proposed (to be accepted by the team in v0.0)
- **Deciders**: A

## Context and problem

The track requires a CI that checks the code on every change, quality measurements at the start and end of the year, and tests that catch real regressions. We need a simple toolchain, identical on all four machines and in CI.

## Options considered

1. pip and venv, flake8 and black, mypy.
2. uv (dependencies and Python), ruff (lint and format), pyright (types), pytest, import-linter (architecture rules).
3. Same as 2, with ty (Astral) instead of pyright.

## Chosen option

2.

## Rationale

- A single fast tool for installing (uv) and a single one for linting and formatting (ruff). The `uv.lock` file pins versions: reproducibility comes built in.
- pyright is the engine behind Pylance in VS Code: the editor's errors and the CI's errors are the same.
- import-linter turns our architecture rules into CI failures instead of wishful thinking.
- ty is still in beta (versions 0.0.x in August 2026): too young to be the CI's safeguard.

## Accepted consequences

- pyright needs Node, installed automatically by the `pyright[nodejs]` package: one more download on first run.
- The team has to learn uv.
- A natural candidate for a decision revisited during the year: switching to ty once its version 1.0 is released.
