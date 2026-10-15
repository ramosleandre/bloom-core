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
    model_id: str = "en-panne"
    max_sensitivity: Sensitivity = Sensitivity.LOCAL_ONLY

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision:
        raise TimeoutError("le backend ne répond pas")


@dataclass(frozen=True)
class RemoteBackend:
    model_id: str = "heberge"
    max_sensitivity: Sensitivity = Sensitivity.PERSONAL

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision:
        return Decision(question_id=question.id, confidence=1.0, model_id=self.model_id, latency_ms=0)


async def test_les_regles_proposent_de_repondre_dans_mail() -> None:
    plan = await DecisionRanker([RulesDecision()]).rank(await mail_bundle(), EXAMPLE_CATALOG)
    assert plan.candidates[0].action_id == "reply.draft"
    assert len(plan.candidates) == 3
    assert plan.decided_by == "rules-v0"


async def test_repli_sur_le_backend_suivant_si_le_prefere_echoue() -> None:
    plan = await DecisionRanker([FailingBackend(), RulesDecision()]).rank(
        await mail_bundle(), EXAMPLE_CATALOG
    )
    assert plan.decided_by == "rules-v0"


async def test_un_contexte_local_only_n_atteint_jamais_un_backend_distant() -> None:
    secret = await mail_bundle(Sensitivity.LOCAL_ONLY)
    with pytest.raises(DecisionUnavailable):
        await DecisionRanker([RemoteBackend()]).rank(secret, EXAMPLE_CATALOG)


async def test_sans_avis_confiant_assess_laisse_le_plancher_decider() -> None:
    action = ResolvedAction(action_id="reply.send", args={}, preview="Envoyer")
    assert await DecisionRanker([RulesDecision()]).assess(await mail_bundle(), action) is None
