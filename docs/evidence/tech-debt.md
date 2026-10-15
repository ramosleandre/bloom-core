# Technical debt and quality (O2)

> What we know we must fix, why not now, and roughly how much it would cost. The dated quality reports (baseline in December, final in May) are produced by `make quality` in `lab/results/quality/`.

## Debt

| Date | Block | What | Why not now | Estimated cost | Status |
|---|---|---|---|---|---|
| 2026-10-15 | core | when the user edits an action at approval time (`edit`), the new arguments are not revalidated | no action has an argument schema yet | 0.5 day | open |

## Measured quality

| Date | ruff warnings | pyright errors | Test coverage | Report |
|---|---|---|---|---|

## Rules deliberately disabled

| Rule | Where | Why |
|---|---|---|
| S101 (use of `assert`) | `tests/` | `assert` is the normal way to write a pytest test |
