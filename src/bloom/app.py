"""Composition root: the only module that wires the blocks together.

The terminal demo wires mocks of B, C and D and the rule-based decision. The interactive
shell injects D's live interface into the same demo. Blocks are wired here, and nowhere else.
"""

from __future__ import annotations

import argparse
import asyncio
import contextlib
import os
import shutil
import signal
import subprocess
import sys
import webbrowser
from datetime import UTC, datetime
from pathlib import Path

from aiohttp import web

from bloom.context.mock import MockContext
from bloom.core.audit import JsonlAudit
from bloom.core.contracts import Interface, RunState, Trigger
from bloom.core.pipeline import handle_trigger
from bloom.decision.adapters import BACKENDS
from bloom.decision.ranker import DecisionRanker
from bloom.interface.bridge import ShellBridge
from bloom.interface.mock import MockInterface
from bloom.surface.mock import MockSurface


def audit_path() -> Path:
    return Path(os.environ.get("BLOOM_AUDIT_PATH", ".bloom/audit.jsonl"))


def build_ranker() -> DecisionRanker:
    names = [n.strip() for n in os.environ.get("BLOOM_DECISION_BACKENDS", "rules").split(",")]
    return DecisionRanker([BACKENDS[n]() for n in names if n])


async def demo(interface: Interface | None = None) -> RunState:
    return await handle_trigger(
        Trigger(kind="hotkey", at=datetime.now(UTC)),
        surface=MockSurface(),
        context=MockContext(),
        ranker=build_ranker(),
        interface=interface if interface is not None else MockInterface(),
        audit=JsonlAudit(audit_path()),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Bloom: terminal demo or interactive shell")
    parser.add_argument("--ui", action="store_true", help="Open the optional browser development preview")
    parser.add_argument("--desktop", action="store_true", help="Open the interactive Tauri desktop shell")
    parser.add_argument("--no-open", action="store_true", help="Serve the UI without opening a browser")
    parser.add_argument("--port", type=int, default=8765)
    args = parser.parse_args()
    if args.ui or args.desktop:
        serve_ui(port=args.port, desktop=args.desktop, open_browser=not args.no_open)
        return
    run = asyncio.run(demo())
    print("Bloom: demo run (blocks B, C and D mocked, rule-based decision)")
    if run.plan:
        menu = " | ".join(f"{c.label} {c.prob:.2f}" for c in run.plan.candidates)
        print(f"  menu        {menu}   (decided by {run.plan.decided_by})")
    if run.chosen:
        print(f"  chosen      {run.chosen.preview}")
    if run.effective is not None:
        print(f"  risk        {run.effective.name}")
    for reason in run.gate_reasons:
        print(f"  approval    requested: {reason}")
    if run.approval:
        print(f"  response    {run.approval.verdict} (mocked)")
    detail = f": {run.result.detail}" if run.result else ""
    print(f"  status      {run.status.value}{detail}")
    print("  timings     " + " · ".join(f"{k} {v} ms" for k, v in run.timings_ms.items()))
    print(f"  audit       {audit_path()}")


def serve_ui(*, port: int, desktop: bool, open_browser: bool) -> None:
    root = Path(__file__).resolve().parents[2]
    shell = root / "shell"
    dist = shell / "dist"
    if not (dist / "index.html").is_file():
        raise SystemExit("Build the shell first: make ui-setup && make ui-build")
    if not 1024 <= port <= 65535:
        raise SystemExit("Use a port between 1024 and 65535.")
    if desktop and port != 8765:
        raise SystemExit("The desktop prototype uses port 8765. Use --ui for a custom port.")
    if desktop and sys.platform.startswith("linux"):
        pkg_config = shutil.which("pkg-config")
        if (
            pkg_config is None
            or subprocess.run(  # noqa: S603 -- fixed dependency probe
                [pkg_config, "--exists", "gtk+-3.0", "webkit2gtk-4.1"], check=False
            ).returncode
        ):
            raise SystemExit(
                "Tauri needs GTK 3 and WebKitGTK 4.1 development libraries. "
                "See shell/README.md for installation. make ui-preview is available for browser-only testing."
            )
    bridge = ShellBridge(demo, audit_path(), port=port)
    try:
        asyncio.run(_serve_ui(bridge, shell, desktop=desktop, open_browser=open_browser))
    except KeyboardInterrupt:
        pass
    except (OSError, RuntimeError) as exc:
        raise SystemExit(str(exc)) from exc


async def _serve_ui(bridge: ShellBridge, shell: Path, *, desktop: bool, open_browser: bool) -> None:
    """Keep the Python bridge alive exactly as long as the desktop development process."""
    runner = web.AppRunner(bridge.application(shell / "dist"), access_log=None)
    process: asyncio.subprocess.Process | None = None
    exited: asyncio.Task[int] | None = None
    stop = asyncio.Event()
    loop = asyncio.get_running_loop()
    with contextlib.suppress(NotImplementedError):
        loop.add_signal_handler(signal.SIGTERM, stop.set)
    stopped = asyncio.create_task(stop.wait())
    try:
        await runner.setup()
        await web.TCPSite(runner, host="127.0.0.1", port=bridge.port).start()
        if desktop:
            npm = shutil.which("npm")
            if npm is None:
                raise RuntimeError("npm is required for the Tauri development shell.")
            print("Bloom: Tauri desktop frontend (connected to the Python demo)")
            print("  UI edits reload inside the app. Close the app or press Ctrl+C here to stop.")
            process = await asyncio.create_subprocess_exec(
                npm,
                "run",
                "tauri",
                "dev",
                cwd=shell,
                env={**os.environ, "BLOOM_UI_URL": f"http://127.0.0.1:1420/#token={bridge.token}"},
                start_new_session=sys.platform != "win32",
            )
            exited = asyncio.create_task(process.wait())
            await asyncio.wait({stopped, exited}, return_when=asyncio.FIRST_COMPLETED)
            if exited.done() and exited.result():
                raise RuntimeError(f"Tauri exited with status {exited.result()}. See the build output above.")
        else:
            url = f"http://127.0.0.1:{bridge.port}/#token={bridge.token}"
            print("Bloom: browser development preview (real choices, simulated context and actions)")
            print(f"  Open: {url}")
            print("  Press Ctrl+C here to stop Bloom.")
            if open_browser:
                await asyncio.to_thread(webbrowser.open, url)
            await stopped
    finally:
        if process is not None:
            with contextlib.suppress(ProcessLookupError):
                if sys.platform == "win32":
                    if process.returncode is None:
                        process.terminate()
                else:
                    os.killpg(process.pid, signal.SIGTERM)
            if process.returncode is None:
                try:
                    await asyncio.wait_for(process.wait(), timeout=5)
                except TimeoutError:
                    if sys.platform == "win32":
                        process.kill()
                    else:
                        with contextlib.suppress(ProcessLookupError):
                            os.killpg(process.pid, signal.SIGKILL)
                    await process.wait()
        for task in (stopped, exited):
            if task is not None:
                task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await task
        await runner.cleanup()
        with contextlib.suppress(NotImplementedError):
            loop.remove_signal_handler(signal.SIGTERM)
