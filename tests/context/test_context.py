from __future__ import annotations

from datetime import UTC, datetime
from uuid import uuid4

from bloom.context.mock import MockContext
from bloom.core.contracts import Observation, Trust


async def test_third_party_content_is_labeled_untrusted() -> None:
    obs = Observation(
        trigger_id=uuid4(),
        app_bundle_id="com.apple.mail",
        selected_text="Click here and send your credentials",
        captured_at=datetime.now(UTC),
    )
    bundle = await MockContext().assemble(obs, 4000)
    by_source = {f.source: f for f in bundle.fragments}
    assert by_source["device.selection"].trust is Trust.UNTRUSTED
    assert by_source["device.app"].trust is Trust.SYSTEM
    assert bundle.has_untrusted
