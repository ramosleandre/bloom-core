"""Fake block C: a fixed scene (an email open in Mail), the example catalog, mocked execution.

EXAMPLE_CATALOG is an example V1 catalog (8 actions, each with its floor). The real catalog
will be set together with the reference use case (roadmap v0.0).
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
        id="reply.draft", label="Draft a reply", floor=Criticality.REVERSIBLE, needs_generation=True
    ),
    ActionDescriptor(
        id="reply.send", label="Send the reply", floor=Criticality.EXTERNAL, needs_generation=True
    ),
    ActionDescriptor(id="text.summarize", label="Summarize", floor=Criticality.SAFE, needs_generation=True),
    ActionDescriptor(id="text.translate", label="Translate", floor=Criticality.SAFE, needs_generation=True),
    ActionDescriptor(id="calendar.add_event", label="Add to calendar", floor=Criticality.REVERSIBLE),
    ActionDescriptor(id="notes.save", label="Save to my notes", floor=Criticality.REVERSIBLE),
    ActionDescriptor(
        id="code.explain", label="Explain the code", floor=Criticality.SAFE, needs_generation=True
    ),
    ActionDescriptor(id="file.trash", label="Move to Trash", floor=Criticality.DESTRUCTIVE),
]


@dataclass
class MockSurface:
    executed: list[ResolvedAction] = field(default_factory=lambda: [])

    async def observe(self, trigger: Trigger) -> Observation:
        return Observation(
            trigger_id=trigger.id,
            app_bundle_id="com.apple.mail",
            window_title="Re: Bloom defense",
            selected_text="Can you confirm you'll attend on Thursday at 2 pm?",
            captured_at=datetime.now(UTC),
        )

    def available(self, obs: Observation) -> list[ActionDescriptor]:
        return list(EXAMPLE_CATALOG)

    async def resolve(self, action_id: str, bundle: ContextBundle) -> ResolvedAction:
        label = next(a.label for a in EXAMPLE_CATALOG if a.id == action_id)
        return ResolvedAction(action_id=action_id, args={}, preview=f"{label} (mocked)")

    async def execute(self, action: ResolvedAction) -> ActionResult:
        self.executed.append(action)
        return ActionResult(ok=True, detail=f"{action.action_id} executed (mocked)")
