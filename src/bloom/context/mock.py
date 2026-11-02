"""Fake block B: turns the observation into labeled fragments, with no memory.

Also used by the lab bench (lab/decision/bench.py) until the real assembler exists (v0.2);
once it does, the bench must use it instead of this mock.
"""

from __future__ import annotations

from dataclasses import dataclass
from uuid import uuid4

from bloom.core.contracts import ContextBundle, Fragment, Observation, Sensitivity, Trust


@dataclass
class MockContext:
    sensitivity: Sensitivity = Sensitivity.PERSONAL

    async def assemble(self, obs: Observation, budget_tokens: int) -> ContextBundle:
        if budget_tokens < 0:
            raise ValueError("budget_tokens must be non-negative")

        raw = [
            ("device.app", obs.app_bundle_id, Trust.SYSTEM),
            ("device.window", obs.window_title, Trust.UNTRUSTED),  # email subject = written by a third party
            ("device.selection", obs.selected_text, Trust.UNTRUSTED),
            ("device.url", obs.url, Trust.UNTRUSTED),
        ]
        fragments: list[Fragment] = []
        dropped: list[str] = []
        remaining = budget_tokens
        for source, content, trust in raw:
            if not content:
                continue
            cost_tokens = (len(content) + 3) // 4
            if cost_tokens > remaining:
                dropped.append(source)
                continue
            fragments.append(
                Fragment(
                    content=content,
                    source=source,
                    sensitivity=self.sensitivity,
                    trust=trust,
                    cost_tokens=cost_tokens,
                )
            )
            remaining -= cost_tokens

        return ContextBundle(
            request_id=uuid4(),
            fragments=fragments,
            budget_tokens=budget_tokens,
            dropped=dropped,
        )
