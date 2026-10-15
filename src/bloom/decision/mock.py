"""Faux bloc A, pour tester le corps sans modèle : ordre du catalogue, avis de risque imposé."""

from __future__ import annotations

from dataclasses import dataclass

from bloom.core.contracts import (
    ActionDescriptor,
    ActionPlan,
    Candidate,
    ContextBundle,
    Criticality,
    ResolvedAction,
)


@dataclass
class MockRanker:
    assess_as: Criticality | None = None  # ex. SAFE pour vérifier que le plancher gagne

    async def rank(self, bundle: ContextBundle, actions: list[ActionDescriptor]) -> ActionPlan:
        prob = 1 / max(len(actions), 1)
        return ActionPlan(
            request_id=bundle.request_id,
            candidates=[Candidate(action_id=a.id, label=a.label, prob=prob) for a in actions[:3]],
            decided_by="mock",
            latency_ms=0,
        )

    async def assess(self, bundle: ContextBundle, action: ResolvedAction) -> Criticality | None:
        return self.assess_as
