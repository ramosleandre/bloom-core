"""Backend de référence : des règles écrites à la main, sans modèle.

C'est la baseline du prototype de décision (O1) : un modèle System 1 n'a d'intérêt que s'il
fait mieux que ces règles. Il ne répond qu'à la question de classement ; pour le reste il n'a
pas d'avis (choice=None), et la policy applique alors le plancher seul.

Ne pas régler ces règles sur le jeu d'évaluation : sinon le score ne mesure plus rien.
"""

from __future__ import annotations

import time
from dataclasses import dataclass

from bloom.core.contracts import ContextBundle, Decision, Question, Sensitivity

# App active -> actions probables, de la plus à la moins probable.
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
    max_sensitivity: Sensitivity = Sensitivity.LOCAL_ONLY  # tourne en local, peut tout voir

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
