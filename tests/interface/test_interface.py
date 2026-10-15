from __future__ import annotations

from uuid import uuid4

from bloom.core.contracts import ActionPlan, Candidate
from bloom.interface.mock import MockInterface


async def test_pick_top_takes_the_first_candidate() -> None:
    plan = ActionPlan(
        request_id=uuid4(),
        candidates=[
            Candidate(action_id="a", label="A", prob=0.7),
            Candidate(action_id="b", label="B", prob=0.3),
        ],
        decided_by="test",
        latency_ms=0,
    )
    choice = await MockInterface().show_pie(plan)
    assert choice.action_id == "a"


async def test_closing_the_menu_returns_none() -> None:
    plan = ActionPlan(request_id=uuid4(), candidates=[], decided_by="test", latency_ms=0)
    assert (await MockInterface(pick=None).show_pie(plan)).action_id is None
