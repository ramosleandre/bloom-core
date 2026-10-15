"""Faux bloc B : transforme l'observation en fragments étiquetés, sans mémoire ni budget réel.

Sert aussi au banc du lab (lab/decision/bench.py) en attendant le vrai assembleur (v0.2) ;
quand il existera, le banc devra l'utiliser à la place de ce mock.
"""

from __future__ import annotations

from dataclasses import dataclass
from uuid import uuid4

from bloom.core.contracts import ContextBundle, Fragment, Observation, Sensitivity, Trust


@dataclass
class MockContext:
    sensitivity: Sensitivity = Sensitivity.PERSONAL

    async def assemble(self, obs: Observation, budget_tokens: int) -> ContextBundle:
        raw = [
            ("device.app", obs.app_bundle_id, Trust.SYSTEM),
            ("device.window", obs.window_title, Trust.UNTRUSTED),  # titre d'un mail = écrit par un tiers
            ("device.selection", obs.selected_text, Trust.UNTRUSTED),
            ("device.url", obs.url, Trust.UNTRUSTED),
        ]
        fragments = [
            Fragment(
                content=content,
                source=source,
                sensitivity=self.sensitivity,
                trust=trust,
                cost_tokens=len(content) // 4 + 1,
            )
            for source, content, trust in raw
            if content
        ]
        return ContextBundle(request_id=uuid4(), fragments=fragments, budget_tokens=budget_tokens)
