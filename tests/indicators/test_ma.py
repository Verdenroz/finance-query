from unittest.mock import AsyncMock

from src.models import Indicator, Interval, TimeRange
from src.services import get_ema, get_sma, get_vwma, get_wma


class TestMA:
    async def test_get_sma(self, historical_quotes, monkeypatch):
        """Test the get_sma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_sma(
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.SMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 113.93
        assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 138.32
        assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 118.57

    async def test_get_ema(self, historical_quotes, monkeypatch):
        """Test the get_ema function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_ema(
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.EMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 116.76
        assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 136.33
        assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 119.40

    async def test_get_wma(self, historical_quotes, monkeypatch):
        """Test the get_wma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_wma(
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.WMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 113.51
        assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 140.26
        assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 117.11

    async def test_get_vwma(self, historical_quotes, monkeypatch):
        """Test the get_vwma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_vwma(
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.VWMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 121.0
        assert round(result["Technical Analysis"]["2025-01-16"].value, 2) == 136.77
        assert round(result["Technical Analysis"]["2024-09-27"].value, 2) == 114.55
