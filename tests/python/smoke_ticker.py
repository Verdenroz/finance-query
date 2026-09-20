#!/usr/bin/env python3
"""Smoke test for the generated `finance_query` Python bindings.

Run with `python3 tests/python/smoke_ticker.py`. With no arguments it is the
orchestrator: it builds the debug wheel, installs it into a scratch venv per
interpreter with `uv`, and re-invokes itself as `--worker` inside each venv
against live network data. `--worker` runs the actual Ticker exercise using
whatever `finance_query` is importable in the current interpreter.
"""

from __future__ import annotations

import argparse
import asyncio
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
WHEELS_DIR = REPO_ROOT / "bindings" / "python" / "target" / "wheels"
INTERPRETERS = ("python3", "python3.14t")
SYMBOL = "AAPL"
BOGUS_SYMBOL = "NOTAREALTICKERXYZ"


async def exercise(symbol: str) -> None:
    import finance_query as fq

    ticker = await fq.Ticker.new(symbol)
    assert ticker.symbol() == symbol, ticker.symbol()

    chart = await ticker.chart(fq.Interval.OneDay, fq.TimeRange.OneMonth)
    candles = chart.candles
    assert len(candles) > 0, "no candles for a live symbol"
    close = candles.close
    assert len(close) == len(candles)
    assert all(isinstance(v, float) for v in close.tolist())

    await ticker.dividends(fq.TimeRange.OneYear)
    await ticker.splits(fq.TimeRange.Max)
    await ticker.news()
    await ticker.recommendations(5)
    await ticker.options(None)
    await ticker.financials(fq.StatementType.Income, fq.Frequency.Annual)
    await ticker.company_profile()
    await ticker.short_interest()

    facts = await ticker.edgar_company_facts()
    taxonomy_name, taxonomy = next(iter(facts.facts.items()))
    assert isinstance(taxonomy_name, str)
    assert isinstance(taxonomy, fq.FactsByTaxonomy)

    blocking_chart = ticker.chart_blocking(fq.Interval.OneDay, fq.TimeRange.OneMonth)
    assert len(blocking_chart.candles) > 0

    # `Ticker.new` never touches the network, so a bogus symbol only fails on
    # the first call that fetches data for it.
    bogus = await fq.Ticker.new(BOGUS_SYMBOL)
    try:
        await bogus.chart(fq.Interval.OneDay, fq.TimeRange.OneMonth)
    except fq.FinanceError as exc:
        assert type(exc) is fq.SymbolNotFound, type(exc)
    else:
        raise AssertionError("a nonexistent symbol did not raise")


def worker() -> None:
    # EDGAR requires a contact email in its User-Agent; the SEC does not
    # verify it, so any well-formed placeholder satisfies edgar_company_facts.
    os.environ.setdefault("EDGAR_EMAIL", "smoke-test@example.com")
    asyncio.run(exercise(SYMBOL))
    print(f"worker ok: {sys.executable}")


def build_wheels() -> None:
    subprocess.run(
        [
            "cargo",
            "soothfast",
            "bind",
            "build",
            "-p",
            "finance-query",
            "--only",
            "python",
            "--debug",
        ],
        cwd=REPO_ROOT,
        check=True,
    )


def run_in_venv(interpreter: str, scratch: Path) -> None:
    resolved = shutil.which(interpreter)
    if resolved is None:
        print(f"skip {interpreter}: not on PATH")
        return
    venv = scratch / interpreter
    subprocess.run(["uv", "venv", "--python", resolved, str(venv)], check=True)
    python = venv / "bin" / "python"
    subprocess.run(
        [
            "uv",
            "pip",
            "install",
            "--python",
            str(python),
            "--find-links",
            str(WHEELS_DIR),
            "--no-index",
            "finance-query",
        ],
        check=True,
    )
    subprocess.run(
        [str(python), "-c", "import finance_query; help(finance_query.Ticker)"],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    subprocess.run([str(python), str(Path(__file__).resolve()), "--worker"], check=True)


def orchestrate() -> None:
    build_wheels()
    with tempfile.TemporaryDirectory(prefix="finance-query-smoke-") as scratch:
        for interpreter in INTERPRETERS:
            run_in_venv(interpreter, Path(scratch))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--worker",
        action="store_true",
        help="run the exercise in the current interpreter",
    )
    args = parser.parse_args()
    if args.worker:
        worker()
    else:
        orchestrate()


if __name__ == "__main__":
    main()
