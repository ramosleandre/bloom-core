"""Chemin critique (O2) : un run complet, de bout en bout, avec les mocks des 4 blocs.

Si un de ces tests casse, le produit est inutilisable ou dangereux.
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


async def test_une_action_externe_refusee_n_est_jamais_executee() -> None:
    ui = MockInterface(pick="reply.send", verdict="reject")
    state, surface, audit = await run(ui, ranker=MockRanker(assess_as=Criticality.SAFE))
    assert state.status is RunStatus.CANCELLED
    assert state.effective is Criticality.EXTERNAL  # le modèle a dit SAFE, le plancher a gagné
    assert ui.approvals and not surface.executed
    assert audit.runs == [state]  # l'audit voit l'état final, une seule fois


async def test_une_action_validee_est_executee() -> None:
    state, surface, _ = await run(MockInterface(pick="reply.send", verdict="approve"))
    assert state.status is RunStatus.DONE
    assert [a.action_id for a in surface.executed] == ["reply.send"]


async def test_fermer_le_menu_annule_sans_rien_executer() -> None:
    state, surface, _ = await run(MockInterface(pick=None))
    assert state.status is RunStatus.CANCELLED and not surface.executed


async def test_un_bloc_qui_plante_donne_un_run_echoue_et_audite() -> None:
    class BrokenSurface(MockSurface):
        async def observe(self, trigger: Trigger) -> Observation:
            raise RuntimeError("permission Accessibilité retirée")

    state, _, audit = await run(MockInterface(), surface=BrokenSurface())
    assert state.status is RunStatus.FAILED
    assert "Accessibilité" in (state.error or "")
    assert audit.runs == [state]


async def test_chaque_etape_est_chronometree() -> None:
    state, _, _ = await run(MockInterface(pick="text.summarize"))  # SAFE : pas de validation
    assert state.status is RunStatus.DONE
    steps = {"observe", "assemble", "rank", "menu_latency", "resolve", "assess", "execute"}
    assert steps <= state.timings_ms.keys()
