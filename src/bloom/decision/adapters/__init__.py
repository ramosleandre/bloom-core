"""One file per decision backend. This is the ONLY place where a model SDK may be imported.

Adding a backend: one file here (e.g. laya.py), one entry in BACKENDS, then `make bench`.
"""

from __future__ import annotations

from collections.abc import Callable

from bloom.core.contracts import DecisionPort
from bloom.decision.adapters.rules import RulesDecision

BACKENDS: dict[str, Callable[[], DecisionPort]] = {
    "rules": RulesDecision,
}
