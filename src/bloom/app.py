"""Composition root: the only module that wires the blocks together.

Today it wires the mocks of B, C and D and the rule-based decision. Each owner will replace
their mock with their real block here, and nowhere else. `make run` runs main().
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
    print("Bloom: demo run (blocks B, C and D mocked, rule-based decision)")
    if run.plan:
        menu = " | ".join(f"{c.label} {c.prob:.2f}" for c in run.plan.candidates)
        print(f"  menu        {menu}   (decided by {run.plan.decided_by})")
    if run.chosen:
        print(f"  chosen      {run.chosen.preview}")
    if run.effective is not None:
        print(f"  risk        {run.effective.name}")
    for reason in run.gate_reasons:
        print(f"  approval    requested: {reason}")
    if run.approval:
        print(f"  response    {run.approval.verdict} (mocked)")
    detail = f": {run.result.detail}" if run.result else ""
    print(f"  status      {run.status.value}{detail}")
    print("  timings     " + " · ".join(f"{k} {v} ms" for k, v in run.timings_ms.items()))
    print(f"  audit       {audit_path()}")
