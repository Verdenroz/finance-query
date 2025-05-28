from unittest.mock import AsyncMock

import orjson
import pytest
from fastapi import HTTPException

from src.models import HistoricalData, Interval, TimeRange
from src.services import get_historical
from tests.conftest import VERSION


class TestHistorical:
    # Sample mock response for historical data
    MOCK_HISTORICAL_DATA = {
        "2025-01-01": HistoricalData(open=150.0, high=155.0, low=149.0, close=153.5, adj_close=153.5, volume=10000000),
        "2025-01-02": HistoricalData(open=153.5, high=158.0, low=152.0, close=157.0, adj_close=157.0, volume=12000000),
    }

    @pytest.fixture
    def mock_api_response(self):
        """
        Fixture that provides a function to get a mock API response for different tickers.
        """
        MOCK_HISTORICAL_API_RESPONSES = {
            "AAPL": {
                "chart": {
                    "result": [
                        {
                            "timestamp": [1672531200, 1672617600],
                            "indicators": {
                                "quote": [
                                    {
                                        "open": [150.0, 153.5],
                                        "high": [155.0, 158.0],
                                        "low": [149.0, 152.0],
                                        "close": [153.5, 157.0],
                                        "volume": [10000000, 12000000],
                                    }
                                ],
                                "adjclose": [{"adjclose": [153.5, 157.0]}],
                            },
                        }
                    ],
                    "error": None,
                }
            },
            "GOOGL": {
                "chart": {
                    "result": [
                        {
                            "timestamp": [1672531200, 1672617600],
                            "indicators": {
                                "quote": [
                                    {
                                        "open": [2800.0, 2850.0],
                                        "high": [2850.0, 2900.0],
                                        "low": [2750.0, 2800.0],
                                        "close": [2850.0, 2900.0],
                                        "volume": [1500000, 1600000],
                                    }
                                ],
                                "adjclose": [{"adjclose": [2850.0, 2900.0]}],
                            },
                        }
                    ],
                    "error": None,
                }
            },
        }

        def get_mock_response(ticker):
            response_content = orjson.dumps(MOCK_HISTORICAL_API_RESPONSES[ticker]).decode("utf-8")
            return response_content

        return get_mock_response

    def test_get_historical_success(self, test_client, mock_finance_client, monkeypatch):
        """Test successful historical data retrieval"""
        mock_get_historical = AsyncMock(return_value=self.MOCK_HISTORICAL_DATA)
        monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

        symbol = "AAPL"
        time_range = "1mo"
        interval = "1d"
        response = test_client.get(f"{VERSION}/historical?symbol={symbol}&range={time_range}&interval={interval}")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 2
        assert "2025-01-01" in data
        assert data["2025-01-01"]["open"] == 150.0
        assert data["2025-01-02"]["close"] == 157.0

        mock_get_historical.assert_awaited_once()
        args = mock_get_historical.await_args[0]
        assert args[1:] == (symbol, TimeRange.ONE_MONTH, Interval.DAILY, False)  # Verify all args except the client

    def test_get_historical_with_epoch(self, test_client, mock_finance_client, monkeypatch):
        """Test historical data retrieval with epoch timestamps"""
        epoch_data = {"1672531200": HistoricalData(open=150.0, high=155.0, low=149.0, close=153.5, adj_close=153.5, volume=10000000)}

        mock_get_historical = AsyncMock(return_value=epoch_data)
        monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

        response = test_client.get(f"{VERSION}/historical?symbol=AAPL&range=1mo&interval=1d&epoch=true")
        assert response.status_code == 200
        assert "1672531200" in response.json()

        mock_get_historical.assert_awaited_once()
        args = mock_get_historical.await_args[0]
        assert args[1:] == ("AAPL", TimeRange.ONE_MONTH, Interval.DAILY, True)  # Verify all args except the client

    def test_get_historical_symbol_not_found(self, test_client, mock_finance_client, monkeypatch):
        """Test when symbol is not found"""
        mock_get_historical = AsyncMock(side_effect=HTTPException(status_code=404, detail="Symbol not found"))
        monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

        response = test_client.get(f"{VERSION}/historical?symbol=NONEXISTENT&range=1mo&interval=1d")

        assert response.status_code == 404
        assert response.json()["detail"] == "Symbol not found"

    def test_get_historical_missing_params(self, test_client, mock_finance_client):
        """Test missing required parameters"""
        response = test_client.get(f"{VERSION}/historical?symbol=AAPL")

        assert response.status_code == 422
        error_data = response.json()
        assert "errors" in error_data or "detail" in error_data

    def test_get_historical_invalid_interval(self, test_client, mock_finance_client):
        """Test with invalid interval value"""
        response = test_client.get(f"{VERSION}/historical?symbol=AAPL&range=1mo&interval=invalid")

        assert response.status_code == 422
        error_data = response.json()
        if "errors" in error_data:
            assert "interval" in error_data["errors"]
        else:
            details = error_data["detail"]
            assert any("interval" in str(item.get("loc", [])) for item in details)

    @pytest.mark.parametrize(
        "interval,time_range,expected_error",
        [
            (Interval.ONE_MINUTE, TimeRange.ONE_MONTH, "If interval is 1m, range must be 1d, 5d"),
            (Interval.ONE_MINUTE, TimeRange.THREE_MONTHS, "If interval is 1m, range must be 1d, 5d"),
            (Interval.ONE_MINUTE, TimeRange.YEAR, "If interval is 1m, range must be 1d, 5d"),
            (Interval.FIVE_MINUTES, TimeRange.THREE_MONTHS, "If interval is 5m, range must be 1d, 5d, 1mo"),
            (Interval.FIVE_MINUTES, TimeRange.YEAR, "If interval is 5m, range must be 1d, 5d, 1mo"),
            (Interval.FIFTEEN_MINUTES, TimeRange.THREE_MONTHS, "If interval is 15m, range must be 1d, 5d, 1mo"),
            (Interval.FIFTEEN_MINUTES, TimeRange.YEAR, "If interval is 15m, range must be 1d, 5d, 1mo"),
            (Interval.THIRTY_MINUTES, TimeRange.THREE_MONTHS, "If interval is 30m, range must be 1d, 5d, 1mo"),
            (Interval.THIRTY_MINUTES, TimeRange.YEAR, "If interval is 30m, range must be 1d, 5d, 1mo"),
            (
                Interval.ONE_HOUR,
                TimeRange.FIVE_YEARS,
                "If interval is 1h, range must be 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y",
            ),
            (Interval.DAILY, TimeRange.MAX, "If range is max, interval must be 1mo"),
            (Interval.WEEKLY, TimeRange.MAX, "If range is max, interval must be 1mo"),
        ],
    )
    async def test_all_invalid_combinations(self, bypass_cache, interval, time_range, expected_error):
        """Test all invalid combinations of interval and time range"""
        with pytest.raises(HTTPException) as exc_info:
            await get_historical(symbol="AAPL", time_range=time_range, interval=interval, finance_client=AsyncMock())

        assert exc_info.value.status_code == 400
        assert exc_info.value.detail == expected_error
