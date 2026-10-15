from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
from uuid import uuid4

import pytest

from bloom.context.mock import MockContext
from bloom.core.contracts import (
    ContextBundle,
    Decision,
    Observation,
    Question,
    ResolvedAction,
    Sensitivity,
)
from bloom.decision.adapters.rules import RulesDecision
from bloom.decision.ranker import DecisionRanker, DecisionUnavailable
from bloom.surface.mock import EXAMPLE_CATALOG


async def mail_bundle(sensitivity: Sensitivity = Sensitivity.PERSONAL) -> ContextBundle:
    obs = Observation(trigger_id=uuid4(), app_bundle_id="com.apple.mail", captured_at=datetime.now(UTC))
    return await MockContext(sensitivity=sensitivity).assemble(obs, 4000)


@dataclass(frozen=True)
class FailingBackend:
    model_id: str = "broken"
    max_sensitivity: Sensitivity = Sensitivity.LOCAL_ONLY

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision:
        raise TimeoutError("the backend does not answer")


@dataclass(frozen=True)
class RemoteBackend:
    model_id: str = "hosted"
    max_sensitivity: Sensitivity = Sensitivity.PERSONAL

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision:
        return Decision(question_id=question.id, confidence=1.0, model_id=self.model_id, latency_ms=0)


async def test_rules_suggest_replying_in_mail() -> None:
    plan = await DecisionRanker([RulesDecision()]).rank(await mail_bundle(), EXAMPLE_CATALOG)
    assert plan.candidates[0].action_id == "reply.draft"
    assert len(plan.candidates) == 3
    assert plan.decided_by == "rules-v0"


async def test_falls_back_to_the_next_backend_if_the_preferred_one_fails() -> None:
    plan = await DecisionRanker([FailingBackend(), RulesDecision()]).rank(
        await mail_bundle(), EXAMPLE_CATALOG
    )
    assert plan.decided_by == "rules-v0"


async def test_a_local_only_context_never_reaches_a_remote_backend() -> None:
    secret = await mail_bundle(Sensitivity.LOCAL_ONLY)
    with pytest.raises(DecisionUnavailable):
        await DecisionRanker([RemoteBackend()]).rank(secret, EXAMPLE_CATALOG)


async def test_without_a_confident_opinion_assess_lets_the_floor_decide() -> None:
    action = ResolvedAction(action_id="reply.send", args={}, preview="Send")
    assert await DecisionRanker([RulesDecision()]).assess(await mail_bundle(), action) is None
