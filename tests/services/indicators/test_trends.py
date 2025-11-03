from unittest.mock import AsyncMock

from src.models import Indicator, Interval, TimeRange
from src.services import get_adx, get_aroon, get_bbands, get_ichimoku, get_macd, get_obv, get_super_trend


class TestTrends:
    async def test_get_adx(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_adx(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.ADX
        assert round(result["Technical Analysis"]["2025-03-14"]["ADX"], 2) == 25.36

    async def test_get_macd(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_macd(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.MACD
        assert round(result["Technical Analysis"]["2025-03-14"]["MACD"], 2) == -4.67

    async def test_get_aroon(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_aroon(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.AROON
        assert round(result["Technical Analysis"]["2025-03-14"]["Aroon Up"], 2) == 28.0

    async def test_get_bbands(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_bbands(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.BBANDS
        assert round(result["Technical Analysis"]["2025-03-14"]["Upper Band"], 2) == 145.44

    async def test_get_obv(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_obv(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.OBV
        assert round(result["Technical Analysis"]["2025-03-14"]["OBV"], 2) == 4908424676.0

    async def test_get_supertrend(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_super_trend(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.SUPER_TREND
        assert result["Technical Analysis"]["2025-03-14"]["Trend"] == "DOWN"

    async def test_get_ichimoku(self, historical_quotes, mock_finance_client, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_ichimoku(
            finance_client=mock_finance_client,
            symbol="NVDA",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
        )
        assert isinstance(result, dict)
        assert result["type"] == Indicator.ICHIMOKU
        assert round(result["Technical Analysis"]["2025-03-14"]["Conversion Line"], 2) == 113.32
