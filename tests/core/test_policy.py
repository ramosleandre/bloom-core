"""Safety rules: tested on every combination, not on three examples."""

from __future__ import annotations

import itertools
from dataclasses import dataclass
from uuid import uuid4

import pytest

from bloom.core.contracts import ContextBundle, Fragment, Sensitivity, Trust
from bloom.core.contracts import Criticality as C
from bloom.core.policy import allowed_backends, effective_criticality, gate_reasons


def bundle(*frags: tuple[Sensitivity, Trust]) -> ContextBundle:
    return ContextBundle(
        request_id=uuid4(),
        budget_tokens=1000,
        fragments=[
            Fragment(content="x", source="t", sensitivity=s, trust=t, cost_tokens=1) for s, t in frags
        ],
    )


@pytest.mark.parametrize(("floor", "contextual"), list(itertools.product(C, [None, *C])))
def test_a_model_never_lowers_below_the_floor(floor: C, contextual: C | None) -> None:
    assert effective_criticality(floor, contextual) >= floor


@pytest.mark.parametrize("user_gate", list(C))
def test_an_external_action_always_needs_approval(user_gate: C) -> None:
    assert gate_reasons(C.EXTERNAL, bundle(), user_gate)


def test_third_party_content_tightens_approval() -> None:
    clean = bundle((Sensitivity.PUBLIC, Trust.USER))
    dirty = bundle((Sensitivity.PUBLIC, Trust.UNTRUSTED))
    assert not gate_reasons(C.REVERSIBLE, clean)
    assert gate_reasons(C.REVERSIBLE, dirty)


def test_a_read_never_needs_approval() -> None:
    assert not gate_reasons(C.SAFE, bundle((Sensitivity.PERSONAL, Trust.UNTRUSTED)))


@dataclass(frozen=True)
class Backend:
    name: str
    max_sensitivity: Sensitivity


def test_a_local_only_context_is_never_routed_to_a_remote_backend() -> None:
    remote = Backend("hosted", Sensitivity.PERSONAL)
    local = Backend("local", Sensitivity.LOCAL_ONLY)
    secret = bundle((Sensitivity.LOCAL_ONLY, Trust.USER))
    assert allowed_backends(secret, [remote, local]) == [local]
    assert allowed_backends(secret, [remote]) == []
