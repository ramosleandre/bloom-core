"""Reference backend: hand-written rules, no model.

This is the baseline of the decision prototype (O1): a System 1 model is only worth it if it
beats these rules. It only answers the ranking question; for the rest it has no opinion
(choice=None), and the policy then applies the floor alone.

Do not tune these rules on the evaluation set: otherwise the score no longer measures anything.
"""

from __future__ import annotations

import time
from dataclasses import dataclass

from bloom.core.contracts import ContextBundle, Decision, Question, Sensitivity

# Active app -> likely actions, from most to least likely.
APP_HINTS: dict[str, list[str]] = {
    "com.apple.mail": ["reply.draft", "text.summarize", "calendar.add_event"],
    "com.google.Chrome": ["text.summarize", "text.translate", "notes.save"],
    "com.apple.Safari": ["text.summarize", "text.translate", "notes.save"],
    "com.microsoft.VSCode": ["code.explain", "text.summarize", "notes.save"],
    "com.apple.finder": ["file.trash", "notes.save"],
}


@dataclass(frozen=True)
class RulesDecision:
    model_id: str = "rules-v0"
    max_sensitivity: Sensitivity = Sensitivity.LOCAL_ONLY  # runs locally, can see everything

    async def decide(self, bundle: ContextBundle, question: Question) -> Decision:
        start = time.perf_counter()
        if question.id != "rank" or not question.choices:
            return Decision(question_id=question.id, confidence=0.0, model_id=self.model_id, latency_ms=0)

        app = next((f.content for f in bundle.fragments if f.source == "device.app"), "")
        hints = APP_HINTS.get(app, [])
        scores = {c: 1.0 + (len(hints) - hints.index(c) if c in hints else 0.0) for c in question.choices}
        total = sum(scores.values())
        probs = {c: s / total for c, s in scores.items()}
        best = max(probs, key=lambda c: probs[c])
        return Decision(
            question_id=question.id,
            choice=best,
            probs=probs,
            confidence=probs[best],
            model_id=self.model_id,
            latency_ms=round((time.perf_counter() - start) * 1000),
        )
