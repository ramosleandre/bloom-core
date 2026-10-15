"""Un fichier par backend de décision. C'est le SEUL endroit où un SDK de modèle a le droit d'être importé.

Ajouter un backend : un fichier ici (ex. laya.py), une entrée dans BACKENDS, puis `make bench`.
"""

from __future__ import annotations

from collections.abc import Callable

from bloom.core.contracts import DecisionPort
from bloom.decision.adapters.rules import RulesDecision

BACKENDS: dict[str, Callable[[], DecisionPort]] = {
    "rules": RulesDecision,
}
