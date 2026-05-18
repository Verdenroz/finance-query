"""Tests for Chart.to_dataframe() returning a polars DataFrame.

These require network access — they're skipped when Yahoo Finance is unreachable.
"""

import pytest

try:
    import polars as pl
    HAS_POLARS = True
except ImportError:
    HAS_POLARS = False

from finance_query import Ticker, Interval, TimeRange


@pytest.mark.asyncio
@pytest.mark.network
@pytest.mark.skipif(not HAS_POLARS, reason="polars not installed")
async def test_chart_to_dataframe_returns_polars_dataframe():
    ticker = await Ticker.new("AAPL")
    chart = await ticker.chart(Interval.OneDay, TimeRange.OneMonth)
    df = chart.to_dataframe()
    assert isinstance(df, pl.DataFrame)
    expected_cols = {"open", "high", "low", "close", "volume"}
    actual_cols = set(df.columns)
    assert expected_cols.issubset(actual_cols), \
        f"missing: {expected_cols - actual_cols}; got: {actual_cols}"
    assert len(df) == len(chart.candles)
