"""Core contracts: the only types that cross a block boundary.

Read docs/architecture.md first (section "Contracts"). Rules:
- types and facades (Protocol) only, no business logic
  (safety logic lives in policy.py);
- frozen + extra="forbid": an unknown field is an error, not a silent omission;
- changing this file = PR with the `contract` label, reviewed by the owners of the affected blocks.
  Adding an optional field is safe; renaming or removing one breaks the other blocks.

The shell's TypeScript types (block D) will be generated from these models, never written by hand.
"""

from __future__ import annotations

from datetime import datetime
from enum import IntEnum, StrEnum
from typing import Any, Literal, Protocol
from uuid import UUID, uuid4

from pydantic import BaseModel, ConfigDict, Field


class Frozen(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")


# ── Trigger and observation: produced by D (hotkey), then by C (screen) ──


class Trigger(Frozen):
    id: UUID = Field(default_factory=uuid4)
    kind: Literal["hotkey"]  # V1: the hotkey only
    at: datetime  # stamped by the shell on key press: start of the latency measured by PERF


class Observation(Frozen):
    """What the device perceives when the hotkey is pressed. Raw, not filtered yet."""

    trigger_id: UUID
    app_bundle_id: str  # e.g. "com.apple.mail"
    window_title: str | None = None
    selected_text: str | None = None
    url: str | None = None
    captured_at: datetime


# ── Context: produced by B ──


class Sensitivity(IntEnum):
    PUBLIC = 0
    PERSONAL = 1  # may go to a remote backend that the policy allows
    LOCAL_ONLY = 2  # never leaves the machine


class Trust(StrEnum):
    USER = "user"  # typed or chosen by the user, preferences
    SYSTEM = "system"  # produced by Bloom or by the OS (name of the active app)
    UNTRUSTED = "untrusted"  # third-party content (received email, web page): injection vector


class Fragment(Frozen):
    content: str
    # Convention: "device.app", "device.window", "device.selection", "device.url",
    # "stm", "ltm:<id>", "prefs"
    source: str
    sensitivity: Sensitivity
    trust: Trust
    cost_tokens: int


class ContextBundle(Frozen):
    request_id: UUID
    fragments: list[Fragment]
    budget_tokens: int
    dropped: list[str] = Field(default_factory=list)  # sources cut by the budget

    @property
    def max_sensitivity(self) -> Sensitivity:
        return max((f.sensitivity for f in self.fragments), default=Sensitivity.PUBLIC)

    @property
    def has_untrusted(self) -> bool:
        return any(f.trust is Trust.UNTRUSTED for f in self.fragments)


# ── Decision: questions asked to the models by A ──


class Question(Frozen):
    """Modeled on the interface that "System 1" models standardize (choice / score / bool)."""

    id: str
    kind: Literal["choice", "score", "bool"]
    prompt: str
    choices: list[str] = Field(default_factory=list)  # required if kind == "choice"


class Decision(Frozen):
    question_id: str
    choice: str | None = None  # None = the backend has no opinion
    score: float | None = None
    probs: dict[str, float] = Field(default_factory=dict)
    confidence: float
    model_id: str
    latency_ms: int


# ── Actions: described, prepared and executed by C ──


class Criticality(IntEnum):
    """Ordered by "damage if we get it wrong". The floor is declared by the action;
    a model can raise it, never lower it (policy.effective_criticality)."""

    SAFE = 0  # read, summary, no side effect
    REVERSIBLE = 1  # writes locally, can be undone (draft, note, event)
    DESTRUCTIVE = 2  # deletes or overwrites locally
    EXTERNAL = 3  # leaves the machine (send, publish, pay): irreversible


class ActionDescriptor(Frozen):
    """Public face of an action: what the decision model and the menu see."""

    id: str  # e.g. "reply.draft"
    label: str  # label shown in the pie menu
    floor: Criticality
    needs_generation: bool = False  # its arguments require a call to the generative model


class ResolvedAction(Frozen):
    action_id: str
    args: dict[str, Any]
    preview: str  # exactly what is going to happen, human-readable
    generated_by: str | None = None  # model_id if an LLM wrote part of the arguments


class ActionResult(Frozen):
    ok: bool
    detail: str
    undo_token: str | None = None


class Candidate(Frozen):
    action_id: str
    label: str
    prob: float


class ActionPlan(Frozen):
    """What comes out of ranking and goes to the pie menu."""

    request_id: UUID
    candidates: list[Candidate]  # sorted by decreasing probability
    decided_by: str  # model_id of the backend that answered
    latency_ms: int


# ── Interface: exchanged with D ──


class MenuChoice(Frozen):
    action_id: str | None  # None = the user closed the menu
    shown_at: datetime  # menu actually visible: end of the latency measured by PERF


class ApprovalRequest(Frozen):
    run_id: UUID
    action_id: str
    effective: Criticality
    preview: str
    reasons: list[str]  # why we ask: shown as is to the user
    expires_at: datetime


class ApprovalResponse(Frozen):
    run_id: UUID
    verdict: Literal["approve", "reject", "edit"]
    edited_args: dict[str, Any] | None = None


# ── Run: held by the core, written to the audit, read by the lab ──


class RunStatus(StrEnum):
    OBSERVING = "observing"
    AWAITING_CHOICE = "awaiting_choice"
    AWAITING_APPROVAL = "awaiting_approval"
    EXECUTING = "executing"
    DONE = "done"
    CANCELLED = "cancelled"
    FAILED = "failed"


class RunState(Frozen):
    """Serializable from day one: this is what will let a run be suspended and resumed."""

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
    timings_ms: dict[str, int] = Field(default_factory=dict)  # per step: PERF tooling


# ── Block facades (ports). Implemented by the blocks, called by the core. ──


class DecisionPort(Protocol):
    """A: a decision backend (Jev, Laya, rules...). One file per backend,
    in bloom/decision/adapters/; no other module imports a model SDK."""

    @property
    def model_id(self) -> str: ...

    @property
    def max_sensitivity(self) -> Sensitivity: ...  # most sensitive data it is allowed to see

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision: ...


class Generator(Protocol):
    """A: generative model (System 2). Injected into C by app.py for actions that write."""

    @property
    def model_id(self) -> str: ...

    @property
    def max_sensitivity(self) -> Sensitivity: ...

    async def generate(self, bundle: ContextBundle, instruction: str) -> str: ...


class Ranker(Protocol):
    """A: facade called by the core."""

    async def rank(self, bundle: ContextBundle, actions: list[ActionDescriptor]) -> ActionPlan: ...

    async def assess(self, bundle: ContextBundle, action: ResolvedAction) -> Criticality | None: ...


class ContextAssembler(Protocol):
    """B: facade called by the core."""

    async def assemble(self, obs: Observation, budget_tokens: int) -> ContextBundle: ...


class Surface(Protocol):
    """C: facade called by the core."""

    async def observe(self, trigger: Trigger) -> Observation: ...

    def available(self, obs: Observation) -> list[ActionDescriptor]: ...

    async def resolve(self, action_id: str, bundle: ContextBundle) -> ResolvedAction: ...

    async def execute(self, action: ResolvedAction) -> ActionResult: ...


class Interface(Protocol):
    """D: facade called by the core (through the IPC bridge from v0.2)."""

    async def show_pie(self, plan: ActionPlan) -> MenuChoice: ...

    async def ask_approval(self, req: ApprovalRequest) -> ApprovalResponse: ...


class Audit(Protocol):
    """Core: log of finished runs."""

    def record(self, run: RunState) -> None: ...
