"""Faux bloc C : une scène fixe (un mail ouvert dans Mail), le catalogue d'exemple, exécution simulée.

EXAMPLE_CATALOG est un exemple de catalogue V1 (8 actions, chacune avec son plancher). Le vrai
catalogue sera fixé avec le cas d'usage de référence (roadmap v0.0).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime

from bloom.core.contracts import (
    ActionDescriptor,
    ActionResult,
    ContextBundle,
    Criticality,
    Observation,
    ResolvedAction,
    Trigger,
)

EXAMPLE_CATALOG = [
    ActionDescriptor(
        id="reply.draft", label="Rédiger une réponse", floor=Criticality.REVERSIBLE, needs_generation=True
    ),
    ActionDescriptor(
        id="reply.send", label="Envoyer la réponse", floor=Criticality.EXTERNAL, needs_generation=True
    ),
    ActionDescriptor(id="text.summarize", label="Résumer", floor=Criticality.SAFE, needs_generation=True),
    ActionDescriptor(id="text.translate", label="Traduire", floor=Criticality.SAFE, needs_generation=True),
    ActionDescriptor(id="calendar.add_event", label="Ajouter au calendrier", floor=Criticality.REVERSIBLE),
    ActionDescriptor(id="notes.save", label="Garder dans mes notes", floor=Criticality.REVERSIBLE),
    ActionDescriptor(
        id="code.explain", label="Expliquer le code", floor=Criticality.SAFE, needs_generation=True
    ),
    ActionDescriptor(id="file.trash", label="Mettre à la corbeille", floor=Criticality.DESTRUCTIVE),
]


@dataclass
class MockSurface:
    executed: list[ResolvedAction] = field(default_factory=lambda: [])

    async def observe(self, trigger: Trigger) -> Observation:
        return Observation(
            trigger_id=trigger.id,
            app_bundle_id="com.apple.mail",
            window_title="Re: soutenance Bloom",
            selected_text="Peux-tu confirmer ta présence jeudi à 14 h ?",
            captured_at=datetime.now(UTC),
        )

    def available(self, obs: Observation) -> list[ActionDescriptor]:
        return list(EXAMPLE_CATALOG)

    async def resolve(self, action_id: str, bundle: ContextBundle) -> ResolvedAction:
        label = next(a.label for a in EXAMPLE_CATALOG if a.id == action_id)
        return ResolvedAction(action_id=action_id, args={}, preview=f"{label} (simulé)")

    async def execute(self, action: ResolvedAction) -> ActionResult:
        self.executed.append(action)
        return ActionResult(ok=True, detail=f"{action.action_id} exécutée (simulé)")
