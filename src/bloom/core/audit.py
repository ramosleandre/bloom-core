"""Append-only log of finished runs. Local, never committed: it contains personal data."""

from __future__ import annotations

from pathlib import Path

from bloom.core.contracts import RunState


class JsonlAudit:
    def __init__(self, path: Path) -> None:
        self.path = path

    def record(self, run: RunState) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with self.path.open("a", encoding="utf-8") as f:
            f.write(run.model_dump_json() + "\n")
