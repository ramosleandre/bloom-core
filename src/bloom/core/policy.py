"""Core safety and routing rules. Small, pure, exhaustively tested.

A trust rule that only exists in a document does not exist.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Protocol

from bloom.core.contracts import ContextBundle, Criticality, Sensitivity

DEFAULT_GATE = Criticality.DESTRUCTIVE


def effective_criticality(floor: Criticality, contextual: Criticality | None) -> Criticality:
    """The model can raise the level, never lower it below the declared floor.
    Silent or unavailable model (None): the floor alone."""
    return floor if contextual is None else max(floor, contextual)


def gate_reasons(
    effective: Criticality,
    bundle: ContextBundle,
    user_gate: Criticality = DEFAULT_GATE,
) -> list[str]:
    """Empty list: no approval needed. Otherwise, the reasons shown to the user."""
    reasons: list[str] = []
    if effective >= Criticality.EXTERNAL:
        reasons.append("the action leaves the machine (always confirmed in V1)")
    elif effective >= user_gate:
        reasons.append(f"risk {effective.name} above the chosen threshold ({user_gate.name})")
    if bundle.has_untrusted and effective >= Criticality.REVERSIBLE:
        reasons.append("the context contains untrusted third-party content")
    return reasons


class Routable(Protocol):
    @property
    def max_sensitivity(self) -> Sensitivity: ...


def allowed_backends[T: Routable](bundle: ContextBundle, backends: Sequence[T]) -> list[T]:
    """The backends allowed to see this context, in order of preference.
    A LOCAL_ONLY fragment therefore excludes every remote backend, by construction."""
    return [b for b in backends if b.max_sensitivity >= bundle.max_sensitivity]
