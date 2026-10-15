from __future__ import annotations

from datetime import UTC, datetime
from uuid import uuid4

from bloom.core.contracts import ContextBundle, Criticality, Trigger
from bloom.surface.mock import EXAMPLE_CATALOG, MockSurface


def test_the_catalog_stays_within_v1_scope() -> None:
    assert len(EXAMPLE_CATALOG) <= 8
    assert len({a.id for a in EXAMPLE_CATALOG}) == len(EXAMPLE_CATALOG)
    floors = {a.floor for a in EXAMPLE_CATALOG}
    assert {Criticality.EXTERNAL, Criticality.DESTRUCTIVE} <= floors  # enough to exercise approval


async def test_resolve_gives_a_readable_preview() -> None:
    surface = MockSurface()
    obs = await surface.observe(Trigger(kind="hotkey", at=datetime.now(UTC)))
    bundle = ContextBundle(request_id=uuid4(), fragments=[], budget_tokens=100)
    resolved = await surface.resolve(surface.available(obs)[0].id, bundle)
    assert resolved.preview
