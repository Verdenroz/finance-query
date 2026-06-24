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


def test_tickers_has_vec_batch_methods():
    expected = {"dividends", "splits", "capital_gains"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"


def test_tickers_has_scalar_batch_methods():
    expected = {"recommendations", "financials", "spark"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"


def test_tickers_has_options():
    assert "options" in {m for m in dir(Tickers) if not m.startswith("_")}


def test_tickers_has_chart_and_range():
    expected = {"chart", "charts_range"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"


def test_tickers_has_cache_helpers():
    """Test that cache-clear helpers exist."""
    expected = {"clear_cache", "clear_quote_cache", "clear_chart_cache"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"


def test_tickers_has_news_and_quote():
    """Test that news (batch) and quote (single-symbol) methods exist on Tickers."""
    expected = {"news", "quote"}
    actual = {m for m in dir(Tickers) if not m.startswith("_")}
    assert expected.issubset(actual), f"missing: {expected - actual}"
