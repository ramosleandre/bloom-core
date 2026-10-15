"""Racine de composition : le seul module qui branche les blocs entre eux.

Aujourd'hui il branche les mocks de B, C et D et la décision par règles. Chaque owner remplacera
son mock par son vrai bloc ici, et nulle part ailleurs. `make run` lance main().
"""

from __future__ import annotations

import asyncio
import os
from datetime import UTC, datetime
from pathlib import Path

from bloom.context.mock import MockContext
from bloom.core.audit import JsonlAudit
from bloom.core.contracts import RunState, Trigger
from bloom.core.pipeline import handle_trigger
from bloom.decision.adapters import BACKENDS
from bloom.decision.ranker import DecisionRanker
from bloom.interface.mock import MockInterface
from bloom.surface.mock import MockSurface


def audit_path() -> Path:
    return Path(os.environ.get("BLOOM_AUDIT_PATH", ".bloom/audit.jsonl"))


def build_ranker() -> DecisionRanker:
    names = [n.strip() for n in os.environ.get("BLOOM_DECISION_BACKENDS", "rules").split(",")]
    return DecisionRanker([BACKENDS[n]() for n in names if n])


async def demo() -> RunState:
    return await handle_trigger(
        Trigger(kind="hotkey", at=datetime.now(UTC)),
        surface=MockSurface(),
        context=MockContext(),
        ranker=build_ranker(),
        interface=MockInterface(),
        audit=JsonlAudit(audit_path()),
    )


def main() -> None:
    run = asyncio.run(demo())
    print("Bloom : run de démonstration (blocs B, C et D simulés, décision par règles)")
    if run.plan:
        menu = " | ".join(f"{c.label} {c.prob:.2f}" for c in run.plan.candidates)
        print(f"  menu        {menu}   (décidé par {run.plan.decided_by})")
    if run.chosen:
        print(f"  choix       {run.chosen.preview}")
    if run.effective is not None:
        print(f"  risque      {run.effective.name}")
    for reason in run.gate_reasons:
        print(f"  validation  demandée : {reason}")
    if run.approval:
        print(f"  réponse     {run.approval.verdict} (simulée)")
    detail = f" : {run.result.detail}" if run.result else ""
    print(f"  statut      {run.status.value}{detail}")
    print("  chronos     " + " · ".join(f"{k} {v} ms" for k, v in run.timings_ms.items()))
    print(f"  audit       {audit_path()}")
