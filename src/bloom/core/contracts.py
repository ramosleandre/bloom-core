"""Contrats du corps : les seuls types qui traversent une frontière de bloc.

Lire d'abord docs/architecture.md (section « Les contrats »). Règles :
- des types et des façades (Protocol) uniquement, aucune logique métier
  (la logique de sécurité vit dans policy.py) ;
- frozen + extra="forbid" : un champ inconnu est une erreur, pas un oubli silencieux ;
- modifier ce fichier = PR avec le label `contrat`, relue par les owners des blocs concernés.
  Ajouter un champ optionnel est sans risque ; renommer ou retirer casse les autres blocs.

Les types TypeScript du shell (bloc D) seront générés depuis ces modèles, jamais écrits à la main.
"""

from __future__ import annotations

from datetime import datetime
from enum import IntEnum, StrEnum
from typing import Any, Literal, Protocol
from uuid import UUID, uuid4

from pydantic import BaseModel, ConfigDict, Field


class Frozen(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")


# ── Déclencheur et observation : produits par D (raccourci) puis par C (écran) ──


class Trigger(Frozen):
    id: UUID = Field(default_factory=uuid4)
    kind: Literal["hotkey"]  # V1 : le raccourci clavier uniquement
    at: datetime  # horodaté par le shell à l'appui : début de la latence mesurée par PERF


class Observation(Frozen):
    """Ce que l'appareil perçoit au moment du raccourci. Brut, pas encore filtré."""

    trigger_id: UUID
    app_bundle_id: str  # ex. "com.apple.mail"
    window_title: str | None = None
    selected_text: str | None = None
    url: str | None = None
    captured_at: datetime


# ── Contexte : produit par B ──


class Sensitivity(IntEnum):
    PUBLIC = 0
    PERSONAL = 1  # peut aller vers un backend distant que la policy autorise
    LOCAL_ONLY = 2  # ne quitte jamais la machine


class Trust(StrEnum):
    USER = "user"  # saisi ou choisi par l'utilisateur, préférences
    SYSTEM = "system"  # produit par Bloom ou par l'OS (nom de l'app active)
    UNTRUSTED = "untrusted"  # contenu tiers (mail reçu, page web) : vecteur d'injection


class Fragment(Frozen):
    content: str
    # Convention : "device.app", "device.window", "device.selection", "device.url",
    # "stm", "ltm:<id>", "prefs"
    source: str
    sensitivity: Sensitivity
    trust: Trust
    cost_tokens: int


class ContextBundle(Frozen):
    request_id: UUID
    fragments: list[Fragment]
    budget_tokens: int
    dropped: list[str] = Field(default_factory=list)  # sources coupées par le budget

    @property
    def max_sensitivity(self) -> Sensitivity:
        return max((f.sensitivity for f in self.fragments), default=Sensitivity.PUBLIC)

    @property
    def has_untrusted(self) -> bool:
        return any(f.trust is Trust.UNTRUSTED for f in self.fragments)


# ── Décision : questions posées aux modèles par A ──


class Question(Frozen):
    """Calquée sur l'interface que les modèles « System 1 » standardisent (choice / score / bool)."""

    id: str
    kind: Literal["choice", "score", "bool"]
    prompt: str
    choices: list[str] = Field(default_factory=list)  # requis si kind == "choice"


class Decision(Frozen):
    question_id: str
    choice: str | None = None  # None = le backend n'a pas d'avis
    score: float | None = None
    probs: dict[str, float] = Field(default_factory=dict)
    confidence: float
    model_id: str
    latency_ms: int


# ── Actions : décrites, préparées et exécutées par C ──


class Criticality(IntEnum):
    """Ordonnée par « dégâts si on se trompe ». Le plancher est déclaré par l'action ;
    un modèle peut le monter, jamais le descendre (policy.effective_criticality)."""

    SAFE = 0  # lecture, résumé, aucun effet de bord
    REVERSIBLE = 1  # écrit en local, annulable (brouillon, note, événement)
    DESTRUCTIVE = 2  # supprime ou écrase en local
    EXTERNAL = 3  # sort de la machine (envoyer, publier, payer) : irréversible


class ActionDescriptor(Frozen):
    """Face publique d'une action : ce que voient le modèle de décision et le menu."""

    id: str  # ex. "reply.draft"
    label: str  # libellé affiché dans le pie menu
    floor: Criticality
    needs_generation: bool = False  # ses arguments exigent un appel au modèle génératif


class ResolvedAction(Frozen):
    action_id: str
    args: dict[str, Any]
    preview: str  # exactement ce qui va se passer, lisible par un humain
    generated_by: str | None = None  # model_id si un LLM a écrit une partie des arguments


class ActionResult(Frozen):
    ok: bool
    detail: str
    undo_token: str | None = None


class Candidate(Frozen):
    action_id: str
    label: str
    prob: float


class ActionPlan(Frozen):
    """Ce qui sort du classement et part vers le pie menu."""

    request_id: UUID
    candidates: list[Candidate]  # triés par probabilité décroissante
    decided_by: str  # model_id du backend qui a répondu
    latency_ms: int


# ── Interface : échangés avec D ──


class MenuChoice(Frozen):
    action_id: str | None  # None = l'utilisateur a fermé le menu
    shown_at: datetime  # menu réellement visible : fin de la latence mesurée par PERF


class ApprovalRequest(Frozen):
    run_id: UUID
    action_id: str
    effective: Criticality
    preview: str
    reasons: list[str]  # pourquoi on demande : affiché tel quel à l'utilisateur
    expires_at: datetime


class ApprovalResponse(Frozen):
    run_id: UUID
    verdict: Literal["approve", "reject", "edit"]
    edited_args: dict[str, Any] | None = None


# ── Run : tenu par le corps, écrit dans l'audit, lu par le lab ──


class RunStatus(StrEnum):
    OBSERVING = "observing"
    AWAITING_CHOICE = "awaiting_choice"
    AWAITING_APPROVAL = "awaiting_approval"
    EXECUTING = "executing"
    DONE = "done"
    CANCELLED = "cancelled"
    FAILED = "failed"


class RunState(Frozen):
    """Sérialisable dès le premier jour : c'est ce qui permettra de suspendre et reprendre un run."""

    run_id: UUID = Field(default_factory=uuid4)
    trigger: Trigger
    status: RunStatus = RunStatus.OBSERVING
    plan: ActionPlan | None = None
    chosen: ResolvedAction | None = None
    effective: Criticality | None = None
    gate_reasons: list[str] = Field(default_factory=list)
    approval: ApprovalResponse | None = None
    result: ActionResult | None = None
    error: str | None = None
    timings_ms: dict[str, int] = Field(default_factory=dict)  # par étape : outillage de PERF


# ── Façades des blocs (ports). Implémentées par les blocs, appelées par le corps. ──


class DecisionPort(Protocol):
    """A : un backend de décision (Jev, Laya, règles...). Un seul fichier par backend,
    dans bloom/decision/adapters/ ; aucun autre module n'importe un SDK de modèle."""

    @property
    def model_id(self) -> str: ...

    @property
    def max_sensitivity(self) -> Sensitivity: ...  # donnée la plus sensible qu'il a le droit de voir

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision: ...


class Generator(Protocol):
    """A : modèle génératif (System 2). Injecté dans C par app.py pour les actions qui écrivent."""

    @property
    def model_id(self) -> str: ...

    @property
    def max_sensitivity(self) -> Sensitivity: ...

    async def generate(self, bundle: ContextBundle, instruction: str) -> str: ...


class Ranker(Protocol):
    """A : façade appelée par le corps."""

    async def rank(self, bundle: ContextBundle, actions: list[ActionDescriptor]) -> ActionPlan: ...

    async def assess(self, bundle: ContextBundle, action: ResolvedAction) -> Criticality | None: ...


class ContextAssembler(Protocol):
    """B : façade appelée par le corps."""

    async def assemble(self, obs: Observation, budget_tokens: int) -> ContextBundle: ...


class Surface(Protocol):
    """C : façade appelée par le corps."""

    async def observe(self, trigger: Trigger) -> Observation: ...

    def available(self, obs: Observation) -> list[ActionDescriptor]: ...

    async def resolve(self, action_id: str, bundle: ContextBundle) -> ResolvedAction: ...

    async def execute(self, action: ResolvedAction) -> ActionResult: ...


class Interface(Protocol):
    """D : façade appelée par le corps (via le pont IPC à partir de la v0.2)."""

    async def show_pie(self, plan: ActionPlan) -> MenuChoice: ...

    async def ask_approval(self, req: ApprovalRequest) -> ApprovalResponse: ...


class Audit(Protocol):
    """Corps : journal des runs terminés."""

    def record(self, run: RunState) -> None: ...
