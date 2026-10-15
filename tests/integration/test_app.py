"""`make run` doit toujours marcher : c'est la première chose qu'une nouvelle personne lance."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from bloom.app import demo
from bloom.core.contracts import RunStatus


async def test_la_demo_tourne_et_ecrit_l_audit(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    audit = tmp_path / "audit.jsonl"
    monkeypatch.setenv("BLOOM_AUDIT_PATH", str(audit))
    run = await demo()
    assert run.status is RunStatus.DONE
    lines = audit.read_text(encoding="utf-8").splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0])["run_id"] == str(run.run_id)
