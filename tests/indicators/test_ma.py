from unittest.mock import AsyncMock

import pytest

from src.models import TimeRange, Interval, Indicator
from src.services import get_sma, get_ema, get_wma, get_vwma


async def test_get_sma(historical_quotes, monkeypatch):
    """Test the get_sma function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_sma(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.SMA
    assert "Technical Analysis" in result

    # Expected SMA value
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 113.93
    assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 138.32
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 118.57


async def test_get_ema(historical_quotes, monkeypatch):
    """Test the get_ema function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_ema(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.EMA
    assert "Technical Analysis" in result

    # Expected EMA value
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 116.76
    assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 136.33
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 119.40


async def test_get_wma(historical_quotes, monkeypatch):
    """Test the get_wma function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_wma(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.WMA
    assert "Technical Analysis" in result

    # Expected WMA value
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 113.51
    assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 140.26
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 117.11


async def test_get_vwma(historical_quotes, monkeypatch):
    """Test the get_vwma function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_vwma(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.VWMA
    assert "Technical Analysis" in result

    # Expected VWMA value
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 121.0
    assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 136.77
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 114.55


