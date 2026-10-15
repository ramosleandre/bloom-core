"""The run, end to end. One function, a hard-coded order, no workflow engine.

Each step matches a step of the "The run" diagram in docs/architecture.md.
The day a second use case requires a branch this function cannot express,
we will extract an abstraction. Not before.
"""

from __future__ import annotations

import time
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from typing import Any

from bloom.core.contracts import (
    ApprovalRequest,
    Audit,
    ContextAssembler,
    Criticality,
    Interface,
    Ranker,
    RunState,
    RunStatus,
    Surface,
    Trigger,
)
from bloom.core.policy import DEFAULT_GATE, effective_criticality, gate_reasons


@dataclass
class _Progress:
    """Current state of the run: kept even if a step crashes, so that the audit sees everything."""

    state: RunState
    timings: dict[str, int] = field(default_factory=lambda: {})

    def update(self, **changes: Any) -> None:
        self.state = self.state.model_copy(update=changes)

    @contextmanager
    def timed(self, step: str) -> Generator[None]:
        start = time.perf_counter()
        try:
            yield
        finally:
            self.timings[step] = round((time.perf_counter() - start) * 1000)


async def handle_trigger(
    trigger: Trigger,
    *,
    surface: Surface,
    context: ContextAssembler,
    ranker: Ranker,
    interface: Interface,
    audit: Audit,
    budget_tokens: int = 4000,
    user_gate: Criticality = DEFAULT_GATE,
    approval_ttl: timedelta = timedelta(minutes=2),
) -> RunState:
    """Never raises: a crashing block gives a FAILED run, audited with its partial state."""
    progress = _Progress(RunState(trigger=trigger))
    try:
        await _run(progress, surface, context, ranker, interface, budget_tokens, user_gate, approval_ttl)
    except Exception as exc:
        progress.update(status=RunStatus.FAILED, error=repr(exc))
    final = progress.state.model_copy(update={"timings_ms": progress.timings})
    audit.record(final)
    return final


async def _run(
    p: _Progress,
    surface: Surface,
    context: ContextAssembler,
    ranker: Ranker,
    interface: Interface,
    budget_tokens: int,
    user_gate: Criticality,
    approval_ttl: timedelta,
) -> None:
    trigger = p.state.trigger

    # 2. Bloom looks at the screen (C)
    with p.timed("observe"):
        obs = await surface.observe(trigger)

    # 3. Bloom gathers the useful context (B)
    with p.timed("assemble"):
        bundle = await context.assemble(obs, budget_tokens)

    # 4. The brain ranks the possible actions (C provides the catalog, A ranks)
    actions = {a.id: a for a in surface.available(obs)}
    with p.timed("rank"):
        plan = await ranker.rank(bundle, list(actions.values()))
    p.update(status=RunStatus.AWAITING_CHOICE, plan=plan)

    # 5. The pie menu opens, the user chooses (D)
    menu = await interface.show_pie(plan)
    p.timings["menu_latency"] = round((menu.shown_at - trigger.at).total_seconds() * 1000)
    if menu.action_id is None or menu.action_id not in actions:
        p.update(status=RunStatus.CANCELLED)
        return

    # 6. Bloom prepares the action: the arguments are filled in BEFORE approval,
    #    so that the human approves the actual content (the written email), not an intent.
    with p.timed("resolve"):
        resolved = await surface.resolve(menu.action_id, bundle)
    with p.timed("assess"):
        contextual = await ranker.assess(bundle, resolved)

    # 7. Safety check (core policy)
    effective = effective_criticality(actions[menu.action_id].floor, contextual)
    reasons = gate_reasons(effective, bundle, user_gate)
    p.update(chosen=resolved, effective=effective, gate_reasons=reasons)

    if reasons:
        p.update(status=RunStatus.AWAITING_APPROVAL)
        response = await interface.ask_approval(
            ApprovalRequest(
                run_id=p.state.run_id,
                action_id=menu.action_id,
                effective=effective,
                preview=resolved.preview,
                reasons=reasons,
                expires_at=datetime.now(UTC) + approval_ttl,
            )
        )
        p.update(approval=response)
        if response.verdict == "reject":
            p.update(status=RunStatus.CANCELLED)
            return
        if response.verdict == "edit":
            # Known debt (docs/evidence/tech-debt.md): revalidate the edited arguments.
            resolved = resolved.model_copy(update={"args": response.edited_args or {}})

    # 8. The action runs (C)
    p.update(status=RunStatus.EXECUTING)
    with p.timed("execute"):
        result = await surface.execute(resolved)
    p.update(status=RunStatus.DONE, result=result)
