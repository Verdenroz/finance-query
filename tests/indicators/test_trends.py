from unittest.mock import AsyncMock

from src.models import TimeRange, Interval, Indicator
from src.services import get_adx, get_macd, get_aroon, get_ichimoku, get_super_trend, get_obv, get_bbands


async def test_get_adx(historical_quotes, monkeypatch):
    """Test the get_adx function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_adx(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.ADX
    assert "Technical Analysis" in result

    # Expected ADX values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 25.36
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == 19.36
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 14.91


async def test_get_macd(historical_quotes, monkeypatch):
    """Test the get_macd function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_macd(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.MACD
    assert "Technical Analysis" in result

    # Expected MACD values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == -4.67
    assert round(result["Technical Analysis"]["2025-03-14"].signal, 2) == -4.64
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == 3.67
    assert round(result["Technical Analysis"]["2024-11-01"].signal, 2) == 4.75
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 1.31
    assert round(result["Technical Analysis"]["2024-09-27"].signal, 2) == 0.26


async def test_get_aroon(historical_quotes, monkeypatch):
    """Test the get_aroon function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_aroon(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.AROON
    assert "Technical Analysis" in result

    # Expected Aroon values
    assert round(result["Technical Analysis"]["2025-03-14"].aroon_up, 2) == 28.0
    assert round(result["Technical Analysis"]["2025-03-14"].aroon_down, 2) == 88.0
    assert round(result["Technical Analysis"]["2024-11-01"].aroon_up, 2) == 68.0
    assert round(result["Technical Analysis"]["2024-11-01"].aroon_down, 2) == 12.0
    assert round(result["Technical Analysis"]["2024-09-27"].aroon_up, 2) == 8.0
    assert round(result["Technical Analysis"]["2024-09-27"].aroon_down, 2) == 40.0


async def test_get_bbands(historical_quotes, monkeypatch):
    """Test the get_bbands function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_bbands(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.BBANDS
    assert "Technical Analysis" in result

    # Expected BBands values
    assert round(result["Technical Analysis"]["2025-03-14"].upper_band, 2) == 145.44
    assert round(result["Technical Analysis"]["2025-03-14"].middle_band, 2) == 123.23
    assert round(result["Technical Analysis"]["2025-03-14"].lower_band, 2) == 101.02

    assert round(result["Technical Analysis"]["2024-11-01"].upper_band, 2) == 145.62
    assert round(result["Technical Analysis"]["2024-11-01"].middle_band, 2) == 137.06
    assert round(result["Technical Analysis"]["2024-11-01"].lower_band, 2) == 128.5

    assert round(result["Technical Analysis"]["2024-09-27"].upper_band, 2) == 127.59
    assert round(result["Technical Analysis"]["2024-09-27"].middle_band, 2) == 114.95
    assert round(result["Technical Analysis"]["2024-09-27"].lower_band, 2) == 102.31


async def test_get_obv(historical_quotes, monkeypatch):
    """Test the get_obv function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_obv(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.OBV
    assert "Technical Analysis" in result

    # Expected OBV values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 4908424676.0
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == 5021676700.0
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 3423303200.0


async def test_get_supertrend(historical_quotes, monkeypatch):
    """Test the get_super_trend function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_super_trend(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.SUPER_TREND
    assert "Technical Analysis" in result

    # Expected SuperTrend values
    assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 130.35
    assert result["Technical Analysis"]["2025-03-14"].trend == "DOWN"
    assert round(result["Technical Analysis"]["2024-11-01"].value, 2) == 129.16
    assert result["Technical Analysis"]["2024-11-01"].trend == "UP"
    assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 124.9
    assert result["Technical Analysis"]["2024-09-27"].trend == "DOWN"


async def test_get_ichimoku(historical_quotes, monkeypatch):
    """Test the get_ichimoku function with real data from fixture"""
    # Mock the get_historical function
    mock_get_historical = AsyncMock(return_value=historical_quotes)
    monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock_get_historical)

    # Use real calculation functions
    result = await get_ichimoku(
        symbol="NVDA",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
    )

    # Verify structure
    assert isinstance(result, dict)
    assert result["type"] == Indicator.ICHIMOKU
    assert "Technical Analysis" in result

    # Expected Ichimoku values
    assert round(result["Technical Analysis"]["2025-03-14"].tenkan_sen, 2) == 113.32
    assert round(result["Technical Analysis"]["2025-03-14"].kijun_sen, 2) == 124.1
    assert round(result["Technical Analysis"]["2025-03-14"].senkou_span_a, 2) == 127.04
    assert round(result["Technical Analysis"]["2025-03-14"].senkou_span_b, 2) == 133.07
    assert round(result["Technical Analysis"]["2025-03-14"].chikou_span, 2) == 121.67

    assert round(result["Technical Analysis"]["2024-11-01"].tenkan_sen, 2) == 138.26
    assert round(result["Technical Analysis"]["2024-11-01"].kijun_sen, 2) == 129.78
    assert round(result["Technical Analysis"]["2024-11-01"].senkou_span_a, 2) == 118.27
    assert round(result["Technical Analysis"]["2024-11-01"].senkou_span_b, 2) == 110.97
    assert round(result["Technical Analysis"]["2024-11-01"].chikou_span, 2) == 138.81

    assert round(result["Technical Analysis"]["2024-09-27"].tenkan_sen, 2) == 120.44
    assert round(result["Technical Analysis"]["2024-09-27"].kijun_sen, 2) == 116.1
    assert round(result["Technical Analysis"]["2024-09-27"].senkou_span_a, 2) == 114.61
    assert round(result["Technical Analysis"]["2024-09-27"].senkou_span_b, 2) == 115.72
    assert round(result["Technical Analysis"]["2024-09-27"].chikou_span, 2) == 135.4
