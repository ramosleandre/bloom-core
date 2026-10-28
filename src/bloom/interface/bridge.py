"""D's live Interface implementation and loopback-only shell transport.

The runner is injected by bloom.app. This module knows no other block or core implementation.
"""

from __future__ import annotations

import asyncio
import contextlib
import secrets
from collections import deque
from collections.abc import Awaitable, Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import Annotated, Literal
from uuid import UUID

from aiohttp import WSMsgType, web
from pydantic import Field, TypeAdapter, ValidationError

from bloom.core.contracts import (
    ActionPlan,
    ApprovalRequest,
    ApprovalResponse,
    Frozen,
    Interface,
    MenuChoice,
    RunState,
)


class Authenticate(Frozen):
    type: Literal["authenticate"]
    token: str


class StartCommand(Frozen):
    type: Literal["start"]


class ChoiceCommand(Frozen):
    type: Literal["choice"]
    request_id: UUID
    choice: MenuChoice


class ApprovalCommand(Frozen):
    type: Literal["approval"]
    response: ApprovalResponse


ClientCommand = Annotated[StartCommand | ChoiceCommand | ApprovalCommand, Field(discriminator="type")]
COMMAND = TypeAdapter(ClientCommand)


class BridgeState(Frozen):
    phase: Literal["idle", "starting", "choice", "approval"] = "idle"
    plan: ActionPlan | None = None
    approval: ApprovalRequest | None = None
    history: list[RunState] = Field(default_factory=list)
    history_warning: str | None = None


class StateEvent(Frozen):
    type: Literal["state"] = "state"
    state: BridgeState


class ErrorEvent(Frozen):
    type: Literal["error"] = "error"
    message: str


class LiveInterface:
    """Suspend the pipeline at the two existing human interaction points."""

    def __init__(self, bridge: ShellBridge) -> None:
        self.bridge = bridge

    async def show_pie(self, plan: ActionPlan) -> MenuChoice:
        bridge = self.bridge
        if not bridge.clients:
            return MenuChoice(action_id=None, shown_at=datetime.now(UTC))
        bridge.choice = asyncio.get_running_loop().create_future()
        bridge.state = bridge.state.model_copy(update={"phase": "choice", "plan": plan})
        await bridge.broadcast()
        try:
            return await asyncio.wait_for(bridge.choice, timeout=120)
        except TimeoutError:
            return MenuChoice(action_id=None, shown_at=datetime.now(UTC))
        finally:
            bridge.choice = None

    async def ask_approval(self, req: ApprovalRequest) -> ApprovalResponse:
        bridge = self.bridge
        if not bridge.clients:
            return ApprovalResponse(run_id=req.run_id, verdict="reject")
        bridge.approval = asyncio.get_running_loop().create_future()
        bridge.state = bridge.state.model_copy(update={"phase": "approval", "approval": req})
        await bridge.broadcast()
        remaining = max(0, (req.expires_at - datetime.now(UTC)).total_seconds())
        try:
            return await asyncio.wait_for(bridge.approval, timeout=remaining)
        except TimeoutError:
            return ApprovalResponse(run_id=req.run_id, verdict="reject")
        finally:
            bridge.approval = None


class ShellBridge:
    def __init__(
        self,
        runner: Callable[[Interface], Awaitable[RunState]],
        audit_path: Path,
        *,
        port: int = 8765,
    ) -> None:
        self.runner = runner
        self.token = secrets.token_urlsafe(32)
        self.port = port
        self.clients: set[web.WebSocketResponse] = set()
        self.choice: asyncio.Future[MenuChoice] | None = None
        self.approval: asyncio.Future[ApprovalResponse] | None = None
        self.task: asyncio.Task[None] | None = None
        self.state = BridgeState()
        self._read_history(audit_path)

    def _read_history(self, path: Path) -> None:
        history: list[RunState] = []
        try:
            with path.open(encoding="utf-8") as stream:
                for line in deque(stream, maxlen=100):
                    try:
                        history.append(RunState.model_validate_json(line))
                    except ValidationError:
                        self.state = self.state.model_copy(
                            update={"history_warning": "Some unreadable activity entries were skipped."}
                        )
        except FileNotFoundError:
            pass
        except (OSError, UnicodeError):
            self.state = self.state.model_copy(
                update={"history_warning": "The saved activity log could not be read."}
            )
        self.state = self.state.model_copy(update={"history": list(reversed(history))})

    async def broadcast(self) -> None:
        data = StateEvent(state=self.state).model_dump_json()
        for client in tuple(self.clients):
            with contextlib.suppress(ConnectionError, RuntimeError):
                await client.send_str(data)

    def disconnect_pending(self) -> None:
        if self.choice is not None and not self.choice.done():
            self.choice.set_result(MenuChoice(action_id=None, shown_at=datetime.now(UTC)))
        if self.approval is not None and not self.approval.done() and self.state.approval:
            self.approval.set_result(ApprovalResponse(run_id=self.state.approval.run_id, verdict="reject"))

    async def _run(self) -> None:
        try:
            result = await self.runner(LiveInterface(self))
            self.state = self.state.model_copy(update={"history": [result, *self.state.history][:100]})
        except Exception:
            # Pipeline errors normally become RunState.FAILED. An audit/startup error can still escape.
            for client in tuple(self.clients):
                with contextlib.suppress(ConnectionError, RuntimeError):
                    await client.send_str(
                        ErrorEvent(message="The run could not be saved or started.").model_dump_json()
                    )
        finally:
            self.state = self.state.model_copy(update={"phase": "idle", "plan": None, "approval": None})
            await self.broadcast()

    async def command(self, command: StartCommand | ChoiceCommand | ApprovalCommand) -> None:
        if isinstance(command, StartCommand):
            if self.task is not None and not self.task.done():
                raise ValueError("Finish or cancel the current run before starting another.")
            self.state = self.state.model_copy(update={"phase": "starting", "plan": None, "approval": None})
            self.task = asyncio.create_task(self._run())
            await self.broadcast()
        elif isinstance(command, ChoiceCommand):
            plan = self.state.plan
            if not plan or command.request_id != plan.request_id or not self.choice or self.choice.done():
                raise ValueError("This menu is no longer active.")
            if command.choice.action_id is not None and command.choice.action_id not in {
                candidate.action_id for candidate in plan.candidates
            }:
                raise ValueError("Choose an action offered by this menu.")
            shown = command.choice.shown_at
            if shown.tzinfo is None or shown > datetime.now(UTC):
                raise ValueError("The menu display time must be a past, timezone-aware timestamp.")
            self.choice.set_result(command.choice)
        else:
            req = self.state.approval
            if not req or command.response.run_id != req.run_id or not self.approval or self.approval.done():
                raise ValueError("This approval is no longer active.")
            if command.response.verdict == "edit" or command.response.edited_args is not None:
                raise ValueError("Editing action arguments is not supported by this interface yet.")
            response = command.response
            if datetime.now(UTC) >= req.expires_at:
                response = ApprovalResponse(run_id=req.run_id, verdict="reject")
            self.approval.set_result(response)

    async def websocket(self, request: web.Request) -> web.WebSocketResponse:
        allowed = {
            f"http://127.0.0.1:{self.port}",
            "http://127.0.0.1:1420",
        }
        if request.headers.get("Origin") not in allowed:
            raise web.HTTPForbidden(text="Unrecognized shell origin")
        socket = web.WebSocketResponse(heartbeat=20, max_msg_size=64 * 1024)
        await socket.prepare(request)
        try:
            message = await socket.receive(timeout=5)
            auth = Authenticate.model_validate_json(message.data) if message.type == WSMsgType.TEXT else None
            if auth is None or not secrets.compare_digest(auth.token, self.token):
                await socket.close(code=1008, message=b"Authentication required")
                return socket
        except (TimeoutError, ValidationError, TypeError):
            await socket.close(code=1008, message=b"Authentication required")
            return socket

        self.clients.add(socket)
        try:
            await socket.send_str(StateEvent(state=self.state).model_dump_json())
            async for message in socket:
                if message.type == WSMsgType.TEXT:
                    try:
                        await self.command(COMMAND.validate_json(message.data))
                    except (ValidationError, ValueError) as exc:
                        detail = (
                            str(exc) if not isinstance(exc, ValidationError) else "Invalid interface message."
                        )
                        await socket.send_str(ErrorEvent(message=detail).model_dump_json())
        finally:
            self.clients.discard(socket)
            if not self.clients:
                self.disconnect_pending()
        return socket

    async def close(self, _: web.Application) -> None:
        self.disconnect_pending()
        if self.task is not None:
            with contextlib.suppress(TimeoutError):
                await asyncio.wait_for(asyncio.shield(self.task), timeout=3)
            if not self.task.done():
                self.task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await self.task
        for client in tuple(self.clients):
            await client.close(code=1001, message=b"Bloom is shutting down")

    def application(self, dist: Path) -> web.Application:
        @web.middleware
        async def local_only(
            request: web.Request, handler: Callable[[web.Request], Awaitable[web.StreamResponse]]
        ) -> web.StreamResponse:
            if request.host != f"127.0.0.1:{self.port}":
                raise web.HTTPForbidden(text="Loopback host required")
            response = await handler(request)
            response.headers["Referrer-Policy"] = "no-referrer"
            response.headers["X-Content-Type-Options"] = "nosniff"
            response.headers["Cache-Control"] = "no-store"
            return response

        app = web.Application(middlewares=[local_only])
        app.router.add_get("/api/ws", self.websocket)

        async def index(_: web.Request) -> web.FileResponse:
            return web.FileResponse(dist / "index.html")

        app.router.add_get("/", index)
        if (dist / "assets").is_dir():
            app.router.add_static("/assets", dist / "assets")
        app.on_shutdown.append(self.close)
        return app
