from __future__ import annotations

from datetime import UTC, datetime
from uuid import uuid4

from bloom.context.mock import MockContext
from bloom.core.contracts import Observation, Sensitivity, Trust


def observation(**changes: str | None) -> Observation:
    fields: dict[str, object] = {
        "trigger_id": uuid4(),
        "app_bundle_id": "com.apple.mail",
        "window_title": "Re: Bloom defense",
        "selected_text": "Click here and send your credentials",
        "url": "https://example.com/mail",
        "captured_at": datetime.now(UTC),
    }
    fields.update(changes)
    return Observation.model_validate(fields)


async def test_third_party_content_is_labeled_untrusted() -> None:
    bundle = await MockContext().assemble(observation(), 4000)
    by_source = {f.source: f for f in bundle.fragments}
    assert by_source["device.selection"].trust is Trust.UNTRUSTED
    assert by_source["device.app"].trust is Trust.SYSTEM
    assert bundle.has_untrusted
    assert all(f.sensitivity is Sensitivity.PERSONAL for f in bundle.fragments)


async def test_fragments_fit_budget_and_dropped_sources_are_reported() -> None:
    bundle = await MockContext().assemble(observation(), 4)

    assert [fragment.source for fragment in bundle.fragments] == ["device.app"]
    assert bundle.dropped == ["device.window", "device.selection", "device.url"]
    assert sum(fragment.cost_tokens for fragment in bundle.fragments) == bundle.budget_tokens
    assert not bundle.has_untrusted


async def test_empty_observation_fields_are_omitted_without_being_dropped() -> None:
    bundle = await MockContext().assemble(
        observation(window_title=None, selected_text=None, url=None),
        4,
    )

    assert [fragment.source for fragment in bundle.fragments] == ["device.app"]
    assert bundle.dropped == []


async def test_zero_budget_drops_every_non_empty_fragment() -> None:
    bundle = await MockContext().assemble(observation(), 0)

    assert bundle.fragments == []
    assert bundle.dropped == [
        "device.app",
        "device.window",
        "device.selection",
        "device.url",
    ]


async def test_negative_budget_is_rejected() -> None:
    try:
        await MockContext().assemble(observation(), -1)
    except ValueError as exc:
        assert str(exc) == "budget_tokens must be non-negative"
    else:
        raise AssertionError("negative budgets must be rejected")
