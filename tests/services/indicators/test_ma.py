from unittest.mock import AsyncMock

from src.models import Indicator, Interval, TimeRange
from src.services import get_ema, get_sma, get_vwma, get_wma


class TestMA:
    async def test_get_sma(self, historical_quotes, mock_finance_client, monkeypatch):
        """Test the get_sma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_sma(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.SMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"]["SMA"], 2) == 113.93
        assert round(result["Technical Analysis"]["2025-01-16"]["SMA"], 2) == 138.32
        assert round(result["Technical Analysis"]["2024-09-27"]["SMA"], 2) == 118.57

    async def test_get_ema(self, historical_quotes, mock_finance_client, monkeypatch):
        """Test the get_ema function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_ema(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.EMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"]["EMA"], 2) == 116.76
        assert round(result["Technical Analysis"]["2025-01-16"]["EMA"], 2) == 136.33
        assert round(result["Technical Analysis"]["2024-09-27"]["EMA"], 2) == 119.40

    async def test_get_wma(self, historical_quotes, mock_finance_client, monkeypatch):
        """Test the get_wma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_wma(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.WMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"]["WMA"], 2) == 113.51
        assert round(result["Technical Analysis"]["2025-01-16"]["WMA"], 2) == 140.26
        assert round(result["Technical Analysis"]["2024-09-27"]["WMA"], 2) == 117.11

    async def test_get_vwma(self, historical_quotes, mock_finance_client, monkeypatch):
        """Test the get_vwma function with real data from fixture"""
        mock_get_historical = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_ma.get_historical", mock_get_historical)

        result = await get_vwma(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )

        assert isinstance(result, dict)
        assert result["type"] == Indicator.VWMA
        assert "Technical Analysis" in result
        assert round(result["Technical Analysis"]["2025-03-14"]["VWMA"], 2) == 121.0
        assert round(result["Technical Analysis"]["2025-01-16"]["VWMA"], 2) == 136.77
        assert round(result["Technical Analysis"]["2024-09-27"]["VWMA"], 2) == 114.55
