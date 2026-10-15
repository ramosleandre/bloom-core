"""Decision prototype (O1): compares decision backends on the same set, with the same protocol.

    make bench
    make bench backends=rules,laya data=lab/datasets/situations.jsonl out=lab/results/2027-01-20_decision

Protocol: each situation becomes an Observation, then a ContextBundle (through block B's assembler,
mocked until the real one exists), then each backend ranks the available actions through
block A's real DecisionRanker. Metrics: top1, top3, p50 and p95 latency.
The "random" row gives the score of a random ranking: a backend that does not beat it is useless.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import math
import platform
import statistics
import subprocess
import time
from datetime import UTC, datetime
from pathlib import Path
from uuid import uuid4

from pydantic import BaseModel

from bloom.context.mock import MockContext
from bloom.core.contracts import ActionDescriptor, Criticality, Observation
from bloom.decision.adapters import BACKENDS
from bloom.decision.ranker import DecisionRanker

TOP_K = 3


class Situation(BaseModel):
    id: str
    app_bundle_id: str
    window_title: str | None = None
    selected_text: str | None = None
    url: str | None = None
    available: list[str]
    expected: list[str]
    profile: str = "default"
    annotator: str = ""
    notes: str = ""


class Score(BaseModel):
    backend: str
    n: int
    top1: float
    top3: float
    p50_ms: float | None
    p95_ms: float | None
    ranked: dict[str, list[str]]


def load(path: Path) -> list[Situation]:
    lines = path.read_text(encoding="utf-8").splitlines()
    return [Situation.model_validate_json(line) for line in lines if line.strip()]


def chance(s: Situation, k: int) -> float:
    """Probability that a random ranking puts an expected action in the top k."""
    n = len(s.available)
    e = len(set(s.expected) & set(s.available))
    k = min(k, n)
    return 1 - math.comb(n - e, k) / math.comb(n, k)


def percentile(values: list[float], q: int) -> float | None:
    if not values:
        return None
    if len(values) == 1:
        return values[0]
    return statistics.quantiles(values, n=100, method="inclusive")[q - 1]


async def score_backend(name: str, situations: list[Situation], warmup: int) -> Score:
    ranker = DecisionRanker([BACKENDS[name]()], top_k=TOP_K)
    assembler = MockContext()  # Replace with block B's real assembler as soon as it exists.
    ranked: dict[str, list[str]] = {}
    latencies: list[float] = []
    hits1 = hits3 = 0
    for i, s in enumerate(situations):
        obs = Observation(
            trigger_id=uuid4(),
            app_bundle_id=s.app_bundle_id,
            window_title=s.window_title,
            selected_text=s.selected_text,
            url=s.url,
            captured_at=datetime.now(UTC),
        )
        bundle = await assembler.assemble(obs, 4000)
        actions = [ActionDescriptor(id=a, label=a, floor=Criticality.SAFE) for a in s.available]
        if i == 0:
            for _ in range(warmup):  # warmup, not measured
                await ranker.rank(bundle, actions)
        start = time.perf_counter()
        plan = await ranker.rank(bundle, actions)
        latencies.append((time.perf_counter() - start) * 1000)
        order = [c.action_id for c in plan.candidates]
        ranked[s.id] = order
        hits1 += bool(order) and order[0] in s.expected
        hits3 += any(a in s.expected for a in order[:3])
    n = len(situations)
    return Score(
        backend=name,
        n=n,
        top1=hits1 / n,
        top3=hits3 / n,
        p50_ms=percentile(latencies, 50),
        p95_ms=percentile(latencies, 95),
        ranked=ranked,
    )


def git_commit() -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],  # noqa: S607
            capture_output=True,
            text=True,
            check=True,
        )
        return out.stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "?"


def fmt(ms: float | None) -> str:
    return "-" if ms is None else f"{ms:.1f}"


async def score_all(names: list[str], situations: list[Situation], warmup: int) -> list[Score]:
    return [await score_backend(name, situations, warmup) for name in names]


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--backends", default="rules", help="comma-separated names")
    parser.add_argument("--data", type=Path, default=Path("lab/datasets/situations.sample.jsonl"))
    parser.add_argument("--out", type=Path, help="output folder (default: lab/results/local/<timestamp>)")
    parser.add_argument("--warmup", type=int, default=1)
    args = parser.parse_args()

    situations = load(args.data)
    names = [b.strip() for b in args.backends.split(",") if b.strip()]
    scores = asyncio.run(score_all(names, situations, args.warmup))
    chance1 = statistics.fmean(chance(s, 1) for s in situations)
    chance3 = statistics.fmean(chance(s, TOP_K) for s in situations)

    print(f"Decision bench: {len(situations)} situations ({args.data})")
    print(f"{'backend':<16}{'top1':>7}{'top3':>7}{'p50 ms':>9}{'p95 ms':>9}")
    print(f"{'random':<16}{chance1:>7.2f}{chance3:>7.2f}{'-':>9}{'-':>9}")
    for sc in scores:
        print(f"{sc.backend:<16}{sc.top1:>7.2f}{sc.top3:>7.2f}{fmt(sc.p50_ms):>9}{fmt(sc.p95_ms):>9}")

    stamp = datetime.now(UTC).strftime("%Y-%m-%d_%H%M%S")
    out: Path = args.out or Path("lab/results/local") / f"{stamp}_decision"
    out.mkdir(parents=True, exist_ok=True)
    report = {
        "date": datetime.now(UTC).isoformat(),
        "commit": git_commit(),
        "machine": platform.platform(),
        "python": platform.python_version(),
        "dataset": str(args.data),
        "n_situations": len(situations),
        "warmup": args.warmup,
        "chance": {"top1": chance1, "top3": chance3},
        "scores": [sc.model_dump() for sc in scores],
    }
    (out / "results.json").write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Results: {out / 'results.json'}")


if __name__ == "__main__":
    main()
