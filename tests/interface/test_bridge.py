"""The real pipeline must pause for a human and honor rejection/disconnection."""

from __future__ import annotations

import asyncio
from collections.abc import AsyncIterator
from datetime import UTC, datetime, timedelta
from pathlib import Path
from uuid import uuid4

import pytest
from aiohttp import ClientWebSocketResponse, WSServerHandshakeError
from aiohttp.test_utils import TestClient, TestServer

from bloom.app import demo
from bloom.core.contracts import ApprovalRequest, Criticality
from bloom.interface.bridge import ApprovalCommand, LiveInterface, ShellBridge


@pytest.fixture
async def shell(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> AsyncIterator[tuple[ShellBridge, TestClient]]:
    monkeypatch.setenv("BLOOM_AUDIT_PATH", str(tmp_path / "audit.jsonl"))
    monkeypatch.setenv("BLOOM_DECISION_BACKENDS", "rules")
    (tmp_path / "index.html").write_text("Bloom shell", encoding="utf-8")
    bridge = ShellBridge(demo, tmp_path / "audit.jsonl")
    async with TestClient(TestServer(bridge.application(tmp_path))) as client:
        assert client.server.port is not None
        bridge.port = client.server.port
        yield bridge, client


async def connect(bridge: ShellBridge, client: TestClient) -> ClientWebSocketResponse:
    socket = await client.ws_connect("/api/ws", origin=f"http://127.0.0.1:{bridge.port}")
    await socket.send_json({"type": "authenticate", "token": bridge.token})
    assert (await socket.receive_json(timeout=2))["state"]["phase"] == "idle"
    return socket


async def state_at(socket: ClientWebSocketResponse, phase: str) -> dict:
    while True:
        event = await socket.receive_json(timeout=2)
        assert event["type"] == "state", event
        if event["state"]["phase"] == phase:
            return event["state"]


@pytest.mark.parametrize("verdict", ["approve", "reject"])
async def test_live_selection_and_approval_drive_the_real_pipeline(
    shell: tuple[ShellBridge, TestClient], verdict: str
) -> None:
    bridge, client = shell
    socket = await connect(bridge, client)
    await socket.send_json({"type": "start"})
    choice = await state_at(socket, "choice")
    assert not choice["history"]  # the run is actually waiting, not already completed
    assert choice["plan"]["decided_by"] == "rules-v0"
    await socket.send_json(
        {
            "type": "choice",
            "request_id": choice["plan"]["request_id"],
            "choice": {"action_id": "reply.draft", "shown_at": datetime.now(UTC).isoformat()},
        }
    )
    approval = await state_at(socket, "approval")
    assert approval["approval"]["preview"] == "Draft a reply (mocked)"
    await socket.send_json(
        {"type": "approval", "response": {"run_id": approval["approval"]["run_id"], "verdict": verdict}}
    )
    final = (await state_at(socket, "idle"))["history"][0]
    assert final["status"] == ("done" if verdict == "approve" else "cancelled")
    assert (final["result"] is not None) == (verdict == "approve")
    await socket.close()


async def test_stale_choices_and_unoffered_actions_cannot_resume_a_run(
    shell: tuple[ShellBridge, TestClient],
) -> None:
    bridge, client = shell
    socket = await connect(bridge, client)
    await socket.send_json({"type": "start"})
    state = await state_at(socket, "choice")
    for request_id, action_id in [(str(uuid4()), "reply.draft"), (state["plan"]["request_id"], "reply.send")]:
        await socket.send_json(
            {
                "type": "choice",
                "request_id": request_id,
                "choice": {"action_id": action_id, "shown_at": datetime.now(UTC).isoformat()},
            }
        )
        assert (await socket.receive_json(timeout=2))["type"] == "error"
        assert bridge.choice is not None and not bridge.choice.done()
    await socket.close()
    assert bridge.task is not None
    await asyncio.wait_for(bridge.task, timeout=2)
    assert bridge.state.history[0].status == "cancelled"


async def test_disconnecting_during_approval_never_executes(shell: tuple[ShellBridge, TestClient]) -> None:
    bridge, client = shell
    socket = await connect(bridge, client)
    await socket.send_json({"type": "start"})
    state = await state_at(socket, "choice")
    await socket.send_json(
        {
            "type": "choice",
            "request_id": state["plan"]["request_id"],
            "choice": {"action_id": "reply.draft", "shown_at": datetime.now(UTC).isoformat()},
        }
    )
    await state_at(socket, "approval")
    await socket.close()
    assert bridge.task is not None
    await asyncio.wait_for(bridge.task, timeout=2)
    assert bridge.state.history[0].status == "cancelled"
    assert bridge.state.history[0].result is None


async def test_wrong_origin_host_and_token_are_rejected(shell: tuple[ShellBridge, TestClient]) -> None:
    bridge, client = shell
    with pytest.raises(WSServerHandshakeError):
        await client.ws_connect("/api/ws", origin="https://unrelated.example")
    assert (await client.get("/", headers={"Host": "unrelated.example"})).status == 403
    socket = await client.ws_connect("/api/ws", origin=f"http://127.0.0.1:{bridge.port}")
    await socket.send_json({"type": "authenticate", "token": "wrong"})
    await socket.receive(timeout=2)
    assert socket.close_code == 1008
    assert not bridge.clients


async def test_expired_approval_is_rejected_and_wrong_run_cannot_approve(
    shell: tuple[ShellBridge, TestClient],
) -> None:
    bridge, client = shell
    socket = await connect(bridge, client)
    request = ApprovalRequest(
        run_id=uuid4(),
        action_id="reply.send",
        effective=Criticality.EXTERNAL,
        preview="Send a reply",
        reasons=["external"],
        expires_at=datetime.now(UTC) - timedelta(seconds=1),
    )
    assert (await LiveInterface(bridge).ask_approval(request)).verdict == "reject"
    bridge.approval = asyncio.get_running_loop().create_future()
    with pytest.raises(ValueError, match="no longer active"):
        await bridge.command(
            ApprovalCommand.model_validate(
                {"type": "approval", "response": {"run_id": str(uuid4()), "verdict": "approve"}}
            )
        )
    assert not bridge.approval.done()
    bridge.disconnect_pending()
    await socket.close()


async def test_corrupt_history_is_skipped(tmp_path: Path) -> None:
    path = tmp_path / "audit.jsonl"
    path.write_text("not json\n", encoding="utf-8")
    bridge = ShellBridge(demo, path)
    assert bridge.state.history == []
    assert bridge.state.history_warning is not None
