"""Desktop process lifecycle, independently of a native renderer or GTK installation."""

from __future__ import annotations

import asyncio
import sys
from pathlib import Path
from typing import Any

import pytest

from bloom import app
from bloom.interface.bridge import ShellBridge


@pytest.mark.parametrize("exit_code", [0, 7])
async def test_desktop_lifecycle_uses_vite_and_cleans_up_on_exit(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, exit_code: int
) -> None:
    bridge = ShellBridge(app.demo, tmp_path / "audit.jsonl", port=0)
    original_exec = asyncio.create_subprocess_exec
    original_close = bridge.close
    closed = asyncio.Event()
    observed: dict[str, Any] = {}

    async def close(application: Any) -> None:
        await original_close(application)
        closed.set()

    async def launch(*args: str, **kwargs: Any) -> asyncio.subprocess.Process:
        observed.update(args=args, **kwargs)
        # Exercise an actual child exit, without pretending to test a Tauri renderer.
        return await original_exec(
            sys.executable,
            "-c",
            f"raise SystemExit({exit_code})",
            start_new_session=sys.platform != "win32",
        )

    monkeypatch.setattr(app.shutil, "which", lambda _: "/local/npm")
    monkeypatch.setattr(app.asyncio, "create_subprocess_exec", launch)
    monkeypatch.setattr(bridge, "close", close)
    if exit_code:
        with pytest.raises(RuntimeError, match="Tauri exited with status 7"):
            await app._serve_ui(bridge, tmp_path, desktop=True, open_browser=False)
    else:
        await app._serve_ui(bridge, tmp_path, desktop=True, open_browser=False)
    assert observed["args"] == ("/local/npm", "run", "tauri", "dev")
    assert observed["env"]["BLOOM_UI_URL"] == f"http://127.0.0.1:1420/#token={bridge.token}"
    assert observed["cwd"] == tmp_path
    assert closed.is_set()
