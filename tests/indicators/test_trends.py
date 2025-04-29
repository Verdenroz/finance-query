from unittest.mock import AsyncMock

from src.models import TimeRange, Interval, Indicator
from src.services import get_adx, get_macd, get_aroon, get_ichimoku, get_super_trend, get_obv, get_bbands


class TestTrends:
    async def test_get_adx(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_adx("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.ADX
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 25.36

    async def test_get_macd(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_macd("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.MACD
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == -4.67

    async def test_get_aroon(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_aroon("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.AROON
        assert round(result["Technical Analysis"]["2025-03-14"].aroon_up, 2) == 28.0

    async def test_get_bbands(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_bbands("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.BBANDS
        assert round(result["Technical Analysis"]["2025-03-14"].upper_band, 2) == 145.44

    async def test_get_obv(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_obv("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.OBV
        assert round(result["Technical Analysis"]["2025-03-14"].value, 2) == 4908424676.0

    async def test_get_supertrend(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_super_trend("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.SUPER_TREND
        assert result["Technical Analysis"]["2025-03-14"].trend == "DOWN"

    async def test_get_ichimoku(self, historical_quotes, monkeypatch):
        mock = AsyncMock(return_value=historical_quotes)
        monkeypatch.setattr("src.services.indicators.get_trends.get_historical", mock)

        result = await get_ichimoku("NVDA", TimeRange.YEAR, Interval.DAILY)
        assert isinstance(result, dict)
        assert result["type"] == Indicator.ICHIMOKU
        assert round(result["Technical Analysis"]["2025-03-14"].tenkan_sen, 2) == 113.32
