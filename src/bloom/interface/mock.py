"""Faux bloc D : choisit et valide à la place de l'utilisateur.

pick="top" prend le premier candidat, pick=None ferme le menu, sinon l'id d'action donné.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Literal

from bloom.core.contracts import ActionPlan, ApprovalRequest, ApprovalResponse, MenuChoice


@dataclass
class MockInterface:
    pick: str | None = "top"
    verdict: Literal["approve", "reject", "edit"] = "approve"
    shown: list[ActionPlan] = field(default_factory=lambda: [])
    approvals: list[ApprovalRequest] = field(default_factory=lambda: [])

    async def show_pie(self, plan: ActionPlan) -> MenuChoice:
        self.shown.append(plan)
        if self.pick == "top":
            choice = plan.candidates[0].action_id if plan.candidates else None
        else:
            choice = self.pick
        return MenuChoice(action_id=choice, shown_at=datetime.now(UTC))

    async def ask_approval(self, req: ApprovalRequest) -> ApprovalResponse:
        self.approvals.append(req)
        return ApprovalResponse(run_id=req.run_id, verdict=self.verdict)
