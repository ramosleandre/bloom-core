"""Critical path (O2): a full run, end to end, with the mocks of the 4 blocks.

If one of these tests breaks, the product is unusable or dangerous.
"""

from __future__ import annotations

from datetime import UTC, datetime

from bloom.context.mock import MockContext
from bloom.core.contracts import Criticality, Observation, Ranker, RunState, RunStatus, Trigger
from bloom.core.pipeline import handle_trigger
from bloom.decision.mock import MockRanker
from bloom.interface.mock import MockInterface
from bloom.surface.mock import MockSurface


class MemoryAudit:
    def __init__(self) -> None:
        self.runs: list[RunState] = []

    def record(self, run: RunState) -> None:
        self.runs.append(run)


async def run(
    interface: MockInterface,
    surface: MockSurface | None = None,
    ranker: Ranker | None = None,
) -> tuple[RunState, MockSurface, MemoryAudit]:
    surface = surface or MockSurface()
    audit = MemoryAudit()
    state = await handle_trigger(
        Trigger(kind="hotkey", at=datetime.now(UTC)),
        surface=surface,
        context=MockContext(),
        ranker=ranker or MockRanker(),
        interface=interface,
        audit=audit,
    )
    return state, surface, audit


async def test_a_rejected_external_action_is_never_executed() -> None:
    ui = MockInterface(pick="reply.send", verdict="reject")
    state, surface, audit = await run(ui, ranker=MockRanker(assess_as=Criticality.SAFE))
    assert state.status is RunStatus.CANCELLED
    assert state.effective is Criticality.EXTERNAL  # the model said SAFE, the floor won
    assert ui.approvals and not surface.executed
    assert audit.runs == [state]  # the audit sees the final state, exactly once


async def test_an_approved_action_is_executed() -> None:
    state, surface, _ = await run(MockInterface(pick="reply.send", verdict="approve"))
    assert state.status is RunStatus.DONE
    assert [a.action_id for a in surface.executed] == ["reply.send"]


async def test_closing_the_menu_cancels_without_executing_anything() -> None:
    state, surface, _ = await run(MockInterface(pick=None))
    assert state.status is RunStatus.CANCELLED and not surface.executed


async def test_a_crashing_block_gives_a_failed_and_audited_run() -> None:
    class BrokenSurface(MockSurface):
        async def observe(self, trigger: Trigger) -> Observation:
            raise RuntimeError("Accessibility permission revoked")

    state, _, audit = await run(MockInterface(), surface=BrokenSurface())
    assert state.status is RunStatus.FAILED
    assert "Accessibility" in (state.error or "")
    assert audit.runs == [state]


async def test_every_step_is_timed() -> None:
    state, _, _ = await run(MockInterface(pick="text.summarize"))  # SAFE: no approval
    assert state.status is RunStatus.DONE
    steps = {"observe", "assemble", "rank", "menu_latency", "resolve", "assess", "execute"}
    assert steps <= state.timings_ms.keys()
