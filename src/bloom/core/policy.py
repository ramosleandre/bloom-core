"""Règles de sécurité et de routage du corps. Petites, pures, testées de façon exhaustive.

Si une règle de confiance n'est écrite que dans un document, elle n'existe pas.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Protocol

from bloom.core.contracts import ContextBundle, Criticality, Sensitivity

DEFAULT_GATE = Criticality.DESTRUCTIVE


def effective_criticality(floor: Criticality, contextual: Criticality | None) -> Criticality:
    """Le modèle peut monter le niveau, jamais le descendre sous le plancher déclaré.
    Modèle muet ou indisponible (None) : le plancher seul."""
    return floor if contextual is None else max(floor, contextual)


def gate_reasons(
    effective: Criticality,
    bundle: ContextBundle,
    user_gate: Criticality = DEFAULT_GATE,
) -> list[str]:
    """Liste vide : pas de validation. Sinon, les raisons affichées à l'utilisateur."""
    reasons: list[str] = []
    if effective >= Criticality.EXTERNAL:
        reasons.append("l'action sort de la machine (toujours confirmée en V1)")
    elif effective >= user_gate:
        reasons.append(f"risque {effective.name} au-dessus du seuil choisi ({user_gate.name})")
    if bundle.has_untrusted and effective >= Criticality.REVERSIBLE:
        reasons.append("le contexte contient du contenu tiers non fiable")
    return reasons


class Routable(Protocol):
    @property
    def max_sensitivity(self) -> Sensitivity: ...


def allowed_backends[T: Routable](bundle: ContextBundle, backends: Sequence[T]) -> list[T]:
    """Les backends autorisés à voir ce contexte, dans l'ordre de préférence.
    Un fragment LOCAL_ONLY exclut donc tout backend distant, par construction."""
    return [b for b in backends if b.max_sensitivity >= bundle.max_sensitivity]
