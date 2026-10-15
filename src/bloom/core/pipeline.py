"""Le run, de bout en bout. Une fonction, un ordre écrit en dur, pas de moteur de workflow.

Chaque étape correspond à une étape du schéma « Le run » de docs/architecture.md.
Le jour où un deuxième cas d'usage impose une branche que cette fonction ne sait pas
exprimer, on extraira une abstraction. Pas avant.
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
    """État courant du run : gardé même si une étape plante, pour que l'audit voie tout."""

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
    """Ne lève jamais : un bloc qui plante donne un run FAILED, audité avec son état partiel."""
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

    # 2. Bloom regarde l'écran (C)
    with p.timed("observe"):
        obs = await surface.observe(trigger)

    # 3. Bloom rassemble le contexte utile (B)
    with p.timed("assemble"):
        bundle = await context.assemble(obs, budget_tokens)

    # 4. Le cerveau classe les actions possibles (C fournit le catalogue, A classe)
    actions = {a.id: a for a in surface.available(obs)}
    with p.timed("rank"):
        plan = await ranker.rank(bundle, list(actions.values()))
    p.update(status=RunStatus.AWAITING_CHOICE, plan=plan)

    # 5. Le menu en cercle s'ouvre, l'utilisateur choisit (D)
    menu = await interface.show_pie(plan)
    p.timings["menu_latency"] = round((menu.shown_at - trigger.at).total_seconds() * 1000)
    if menu.action_id is None or menu.action_id not in actions:
        p.update(status=RunStatus.CANCELLED)
        return

    # 6. Bloom prépare l'action : les arguments sont remplis AVANT la validation,
    #    pour que l'humain valide le contenu réel (le mail écrit), pas une intention.
    with p.timed("resolve"):
        resolved = await surface.resolve(menu.action_id, bundle)
    with p.timed("assess"):
        contextual = await ranker.assess(bundle, resolved)

    # 7. Contrôle de sécurité (policy du corps)
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
            # Dette connue (docs/evidence/tech-debt.md) : revalider les arguments édités.
            resolved = resolved.model_copy(update={"args": response.edited_args or {}})

    # 8. L'action s'exécute (C)
    p.update(status=RunStatus.EXECUTING)
    with p.timed("execute"):
        result = await surface.execute(resolved)
    p.update(status=RunStatus.DONE, result=result)
