from unittest.mock import AsyncMock

from src.models import TimeRange, Interval, Indicator
from src.services import get_rsi, get_srsi, get_stoch, get_cci


async def test_get_rsi(historical_quotes, monkeypatch):
    """Test the get_rsi function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_oscillators.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_rsi(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.RSI
    assert "Technical Analysis" in result

    # Expected RSI values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 49.32
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == 52.84
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 54.91


async def test_get_srsi(historical_quotes, monkeypatch):
    """Test the get_srsi function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_oscillators.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_srsi(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.SRSI
    assert "Technical Analysis" in result

    # Expected SRSI values
    assert round(result["Technical Analysis"]["2025-03-14"].k, 2) ==  74.02
    assert round(result["Technical Analysis"]["2025-03-14"].d, 2) == 46.41
    assert round(result["Technical Analysis"]["2024-11-01"].k, 2) ==  8.04
    assert round(result["Technical Analysis"]["2024-11-01"].d, 2) == 16.7
    assert round(result["Technical Analysis"]["2024-09-27"].k, 2) == 92.14
    assert round(result["Technical Analysis"]["2024-09-27"].d, 2) == 95.39


async def test_get_stoch(historical_quotes, monkeypatch):
    """Test the get_stoch function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_oscillators.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_stoch(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.STOCH
    assert "Technical Analysis" in result

    # Expected STOCH values
    assert round(result["Technical Analysis"]["2025-03-14"].k, 2) == 39.25
    assert round(result["Technical Analysis"]["2025-03-14"].d, 2) == 26.16
    assert round(result["Technical Analysis"]["2024-11-01"].k, 2) == 45.24
    assert round(result["Technical Analysis"]["2024-11-01"].d, 2) == 59.03
    assert round(result["Technical Analysis"]["2024-09-27"].k, 2) == 83.77
    assert round(result["Technical Analysis"]["2024-09-27"].d, 2) == 88.05


async def test_get_cci(historical_quotes, monkeypatch):
    """Test the get_cci function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_oscillators.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_cci(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.CCI
    assert "Technical Analysis" in result

    # Expected CCI values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == -20.62
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == -24.36
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 88.88
