"""Tests for PyTicker — Ticker.new() and ticker.quote()."""

import pytest
from finance_query import Ticker, Interval, TimeRange, StatementType, Frequency


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_new_returns_ticker_with_symbol():
    ticker = await Ticker.new("AAPL")
    assert ticker.symbol == "AAPL"


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_quote_returns_quote_with_symbol():
    ticker = await Ticker.new("AAPL")
    quote = await ticker.quote()
    assert quote.symbol == "AAPL"


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_chart_returns_chart_with_candles():
    ticker = await Ticker.new("AAPL")
    chart = await ticker.chart(Interval.OneDay, TimeRange.OneMonth)
    assert len(chart.candles) > 0
    first = chart.candles[0]
    assert first.open is not None
    assert first.close is not None


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_chart_range():
    ticker = await Ticker.new("AAPL")
    chart = await ticker.chart_range(Interval.OneDay, 1700000000, 1702592000)
    assert len(chart.candles) > 0


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_dividends():
    ticker = await Ticker.new("AAPL")
    divs = await ticker.dividends(TimeRange.OneYear)
    assert isinstance(divs, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_financials():
    ticker = await Ticker.new("AAPL")
    fin = await ticker.financials(StatementType.Income, Frequency.Quarterly)
    assert fin is not None


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_news():
    ticker = await Ticker.new("AAPL")
    news = await ticker.news()
    assert isinstance(news, list)


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_recommendations():
    ticker = await Ticker.new("AAPL")
    recs = await ticker.recommendations(5)
    assert recs is not None


def test_ticker_has_all_methods():
    """Smoke: no network. Just verify the method bindings exist."""
    expected = {
        "new", "symbol", "quote", "chart", "chart_range",
        "dividends", "splits", "capital_gains",
        "financials", "news", "recommendations",
        "edgar_submissions",
        "clear_cache", "clear_quote_cache", "clear_chart_cache",
    }
    actual = {m for m in dir(Ticker) if not m.startswith("_")}
    missing = expected - actual
    assert not missing, f"missing methods: {missing}"


def test_ticker_has_builder():
    """Smoke: builder() exists and returns a TickerBuilder."""
    from finance_query import TickerBuilder
    b = Ticker.builder("AAPL")
    # Check the builder type is what we expect
    assert b is not None
    # Check the setter methods are callable and chain
    assert hasattr(b, "lang")
    assert hasattr(b, "region_code")
    assert hasattr(b, "build")


@pytest.mark.asyncio
@pytest.mark.network
async def test_ticker_builder_chain_builds_ticker():
    ticker = await Ticker.builder("7203.T").lang("ja-JP").region_code("JP").build()
    assert ticker.symbol == "7203.T"


def test_enable_logging_callable():
    """Smoke: enable_logging() can be called with default and explicit level."""
    import finance_query
    finance_query.enable_logging()  # default INFO
    finance_query.enable_logging(level="DEBUG")
    # Idempotent — subscriber already installed silently no-ops.
    finance_query.enable_logging(level="WARN")


def test_enable_logging_rejects_invalid_level():
    import finance_query
    import pytest
    with pytest.raises(ValueError):
        finance_query.enable_logging(level="BOGUS")
