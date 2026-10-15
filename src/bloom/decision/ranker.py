"""Façade du bloc A : classe les actions (rank) et évalue leur risque (assess) via des DecisionPort.

Les backends sont essayés dans l'ordre de préférence, parmi ceux que la policy autorise pour
la sensibilité du contexte. Un backend qui échoue ou dépasse le délai cède sa place au suivant.
"""

from __future__ import annotations

import asyncio
import time
from collections.abc import Sequence

from bloom.core.contracts import (
    ActionDescriptor,
    ActionPlan,
    Candidate,
    ContextBundle,
    Criticality,
    Decision,
    DecisionPort,
    Question,
    ResolvedAction,
)
from bloom.core.policy import allowed_backends


class DecisionUnavailable(RuntimeError):
    """Aucun backend autorisé n'a pu répondre."""


class DecisionRanker:
    def __init__(
        self,
        backends: Sequence[DecisionPort],
        *,
        top_k: int = 3,
        timeout_s: float = 0.5,
        min_confidence: float = 0.6,
    ) -> None:
        self.backends = list(backends)
        self.top_k = top_k
        self.timeout_s = timeout_s
        self.min_confidence = min_confidence

    async def _ask(self, bundle: ContextBundle, question: Question) -> Decision:
        failures: list[str] = []
        for backend in allowed_backends(bundle, self.backends):
            try:
                return await asyncio.wait_for(backend.decide(bundle, question), self.timeout_s)
            except Exception as exc:  # repli : on passe au backend suivant
                failures.append(f"{backend.model_id}: {exc!r}")
        raise DecisionUnavailable("; ".join(failures) or "aucun backend autorisé pour ce contexte")

    async def rank(self, bundle: ContextBundle, actions: list[ActionDescriptor]) -> ActionPlan:
        start = time.perf_counter()
        labels = {a.id: a.label for a in actions}
        question = Question(
            id="rank",
            kind="choice",
            prompt="Quelle action l'utilisateur veut-il lancer maintenant ?",
            choices=list(labels),
        )
        decision = await self._ask(bundle, question)
        ordered = sorted(labels, key=lambda a: decision.probs.get(a, 0.0), reverse=True)
        return ActionPlan(
            request_id=bundle.request_id,
            candidates=[
                Candidate(action_id=a, label=labels[a], prob=decision.probs.get(a, 0.0))
                for a in ordered[: self.top_k]
            ],
            decided_by=decision.model_id,
            latency_ms=round((time.perf_counter() - start) * 1000),
        )

    async def assess(self, bundle: ContextBundle, action: ResolvedAction) -> Criticality | None:
        question = Question(
            id="assess",
            kind="choice",
            prompt=f"Niveau de risque de cette action : {action.preview}",
            choices=[c.name for c in Criticality],
        )
        try:
            decision = await self._ask(bundle, question)
        except DecisionUnavailable:
            return None  # pas d'avis : la policy appliquera le plancher seul
        if decision.choice is None or decision.choice not in Criticality.__members__:
            return None
        if decision.confidence < self.min_confidence:
            return None
        return Criticality[decision.choice]
