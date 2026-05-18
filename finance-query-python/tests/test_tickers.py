"""Tests for PyTickers (batch ticker surface)."""

import pytest
from finance_query import Tickers, BatchResult, Interval, TimeRange


def test_tickers_class_and_batchresult_importable():
    assert Tickers is not None
    assert BatchResult is not None
    # Check expected methods exist
    expected = {"new", "symbols", "quotes", "charts"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"


@pytest.mark.asyncio
@pytest.mark.network
async def test_tickers_new_returns_tickers_with_symbols():
    tickers = await Tickers.new(["AAPL", "MSFT", "NVDA"])
    syms = tickers.symbols()
    assert set(syms) == {"AAPL", "MSFT", "NVDA"}
    assert len(tickers) == 3


@pytest.mark.asyncio
@pytest.mark.network
async def test_tickers_quotes_returns_batch_result():
    tickers = await Tickers.new(["AAPL", "MSFT"])
    result = await tickers.quotes()
    assert isinstance(result.data, dict)
    assert isinstance(result.errors, dict)
    # At least one symbol should resolve (no errors) — or all errored (also valid).
    assert len(result.data) + len(result.errors) == 2


@pytest.mark.asyncio
@pytest.mark.network
async def test_tickers_charts_returns_batch_result():
    tickers = await Tickers.new(["AAPL", "MSFT"])
    result = await tickers.charts(Interval.OneDay, TimeRange.OneMonth)
    assert isinstance(result.data, dict)
    assert isinstance(result.errors, dict)
    assert len(result.data) + len(result.errors) == 2
