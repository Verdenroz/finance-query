from unittest.mock import AsyncMock, patch

import pytest
from fastapi import HTTPException
from orjson import orjson

from src.models import TimeRange, Interval, HistoricalData
from src.services import get_historical
from tests.conftest import VERSION

# Sample mock response for historical data
MOCK_HISTORICAL_DATA = {
    "2025-01-01": HistoricalData(
        open=150.0,
        high=155.0,
        low=149.0,
        close=153.5,
        adj_close=153.5,
        volume=10000000
    ),
    "2025-01-02": HistoricalData(
        open=153.5,
        high=158.0,
        low=152.0,
        close=157.0,
        adj_close=157.0,
        volume=12000000
    )
}


@pytest.fixture
def mock_api_response():
    """
    Fixture that provides a function to get a mock API response for different tickers.
    """
    # Sample mock responses for different tickers
    MOCK_HISTORICAL_API_RESPONSES = {
        "AAPL": {
            "chart": {
                "result": [{
                    "timestamp": [1672531200, 1672617600],
                    "indicators": {
                        "quote": [{
                            "open": [150.0, 153.5],
                            "high": [155.0, 158.0],
                            "low": [149.0, 152.0],
                            "close": [153.5, 157.0],
                            "volume": [10000000, 12000000]
                        }],
                        "adjclose": [{
                            "adjclose": [153.5, 157.0]
                        }]
                    }
                }],
                "error": None
            }
        },
        "GOOGL": {
            "chart": {
                "result": [{
                    "timestamp": [1672531200, 1672617600],
                    "indicators": {
                        "quote": [{
                            "open": [2800.0, 2850.0],
                            "high": [2850.0, 2900.0],
                            "low": [2750.0, 2800.0],
                            "close": [2850.0, 2900.0],
                            "volume": [1500000, 1600000]
                        }],
                        "adjclose": [{
                            "adjclose": [2850.0, 2900.0]
                        }]
                    }
                }],
                "error": None
            }
        }
    }

    def get_mock_response(ticker):
        response_content = orjson.dumps(MOCK_HISTORICAL_API_RESPONSES[ticker]).decode('utf-8')
        return response_content

    return get_mock_response


# Test cases
def test_get_historical_success(test_client, mock_yahoo_auth, monkeypatch):
    """Test successful historical data retrieval"""
    # Mock the historical service function
    mock_get_historical = AsyncMock(return_value=MOCK_HISTORICAL_DATA)
    monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

    # Make the request
    symbol = "AAPL"
    time_range = "1mo"
    interval = "1d"
    response = test_client.get(f"{VERSION}/historical?symbol={symbol}&range={time_range}&interval={interval}")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == 2  # Two data points in our mock
    assert "2025-01-01" in data
    assert data["2025-01-01"]["open"] == 150.0
    assert data["2025-01-02"]["close"] == 157.0

    # Verify mock was called with correct parameters
    mock_get_historical.assert_awaited_once_with(
        symbol, TimeRange.ONE_MONTH, Interval.DAILY, False
    )


def test_get_historical_with_epoch(test_client, mock_yahoo_auth, monkeypatch):
    """Test historical data retrieval with epoch timestamps"""
    epoch_data = {
        "1672531200": HistoricalData(
            open=150.0, high=155.0, low=149.0, close=153.5, adj_close=153.5, volume=10000000
        )
    }

    mock_get_historical = AsyncMock(return_value=epoch_data)
    monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

    # Make the request with epoch=true
    response = test_client.get(f"{VERSION}/historical?symbol=AAPL&range=1mo&interval=1d&epoch=true")

    # Assertions
    assert response.status_code == 200
    assert "1672531200" in response.json()

    # Verify mock was called with epoch=True
    mock_get_historical.assert_awaited_once_with(
        "AAPL", TimeRange.ONE_MONTH, Interval.DAILY, True
    )


def test_get_historical_symbol_not_found(test_client, mock_yahoo_auth, monkeypatch):
    """Test when symbol is not found"""
    # Mock the service to raise a 404 error
    from fastapi import HTTPException
    mock_get_historical = AsyncMock(side_effect=HTTPException(status_code=404, detail="Symbol not found"))
    monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

    # Make the request with a non-existent symbol
    response = test_client.get(f"{VERSION}/historical?symbol=NONEXISTENT&range=1mo&interval=1d")

    # Assertions
    assert response.status_code == 404
    assert response.json()["detail"] == "Symbol not found"


def test_get_historical_missing_params(test_client, mock_yahoo_auth):
    """Test missing required parameters"""
    # Make the request without required parameters
    response = test_client.get(f"{VERSION}/historical?symbol=AAPL")

    # Should return a 422 Unprocessable Entity
    assert response.status_code == 422

    # Validate error response structure
    error_data = response.json()
    assert "errors" in error_data or "detail" in error_data


def test_get_historical_invalid_interval(test_client, mock_yahoo_auth):
    """Test with invalid interval value"""
    # Make the request with an invalid interval
    response = test_client.get(f"{VERSION}/historical?symbol=AAPL&range=1mo&interval=invalid")

    # Should return a 422 Unprocessable Entity
    assert response.status_code == 422

    # Check that the error is about the interval parameter
    error_data = response.json()
    if "errors" in error_data:
        assert "interval" in error_data["errors"]
    else:
        error_details = error_data["detail"]
        assert any("interval" in str(item.get("loc", [])) for item in error_details)


@pytest.mark.parametrize("interval,time_range,expected_error", [
    # 1-minute interval only works with 1d and 5d ranges
    (Interval.ONE_MINUTE, TimeRange.ONE_MONTH, "If interval is 1m, range must be 1d, 5d"),
    (Interval.ONE_MINUTE, TimeRange.THREE_MONTHS, "If interval is 1m, range must be 1d, 5d"),
    (Interval.ONE_MINUTE, TimeRange.YEAR, "If interval is 1m, range must be 1d, 5d"),

    # 5-minute interval only works with 1d, 5d, and 1mo ranges
    (Interval.FIVE_MINUTES, TimeRange.THREE_MONTHS, "If interval is 5m, range must be 1d, 5d, 1mo"),
    (Interval.FIVE_MINUTES, TimeRange.YEAR, "If interval is 5m, range must be 1d, 5d, 1mo"),

    # 15-minute interval only works with 1d, 5d, and 1mo ranges
    (Interval.FIFTEEN_MINUTES, TimeRange.THREE_MONTHS, "If interval is 15m, range must be 1d, 5d, 1mo"),
    (Interval.FIFTEEN_MINUTES, TimeRange.YEAR, "If interval is 15m, range must be 1d, 5d, 1mo"),

    # 30-minute interval only works with 1d, 5d, and 1mo ranges
    (Interval.THIRTY_MINUTES, TimeRange.THREE_MONTHS, "If interval is 30m, range must be 1d, 5d, 1mo"),
    (Interval.THIRTY_MINUTES, TimeRange.YEAR, "If interval is 30m, range must be 1d, 5d, 1mo"),

    # 1-hour interval has specific allowed ranges
    (Interval.ONE_HOUR, TimeRange.FIVE_YEARS, "If interval is 1h, range must be 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y"),

    # Max range only works with monthly interval
    (Interval.DAILY, TimeRange.MAX, "If range is max, interval must be 1mo"),
    (Interval.WEEKLY, TimeRange.MAX, "If range is max, interval must be 1mo")
])
async def test_all_invalid_combinations(bypass_cache, interval, time_range, expected_error):
    """Test all invalid combinations of interval and time range"""
    with pytest.raises(HTTPException) as exc_info:
        await get_historical("AAPL", time_range, interval)

    # Assertions
    assert exc_info.value.status_code == 400, f"Failed for {interval.value} and {time_range.value}"
    assert exc_info.value.detail == expected_error


@pytest.mark.parametrize("symbol, expected_open, expected_close", [
    ("AAPL", 150.0, 157.0),
    ("GOOGL", 2800.0, 2900.0)
])
async def test_get_historical_api_success(bypass_cache, mock_api_response, symbol, expected_open, expected_close):
    """Test successful historical data retrieval with mocked API response"""
    time_range = TimeRange.ONE_MONTH
    interval = Interval.DAILY

    # Mock the fetch function to return the mock API response
    with patch('src.services.historical.get_historical.fetch', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.return_value = mock_api_response(symbol)

        # Call the get_historical function
        result = await get_historical(symbol, time_range, interval)

        # Verify the results
        assert len(result) == 2  # Two data points in our mock
        assert "2023-01-01" in result
        assert result["2023-01-01"].open == expected_open
        assert result["2023-01-02"].close == expected_close

        # Verify the fetch function was called with the correct URL
        expected_url = f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?interval={interval.value}&range={time_range.value}&includePrePost=false"
        mock_fetch.assert_called_once_with(url=expected_url)


@pytest.mark.parametrize("test_case", [
    {
        "response": {},
        "expected_status": 500,
        "expected_detail": "Invalid response structure from Yahoo Finance API"
    },
    {
        "response": {"chart": {"result": None}},
        "expected_status": 404,
        "expected_detail": "No data returned for symbol"
    },
    {
        "response": {"chart": {"result": []}},
        "expected_status": 404,
        "expected_detail": "No data returned for symbol"
    },
    {
        "response": {
            "chart": {
                "error": {
                    "code": "Not Found",
                    "description": "Symbol AAPL not found"
                }
            }
        },
        "expected_status": 404,
        "expected_detail": "Symbol AAPL not found"
    },
    {
        "response": {
            "chart": {
                "error": {
                    "code": "Internal Server Error",
                    "description": "Yahoo API unavailable"
                }
            }
        },
        "expected_status": 500,
        "expected_detail": "Failed to retrieve historical data: Yahoo API unavailable"
    }
])
async def test_get_historical_yahoo_errors(bypass_cache, test_case):
    """Test handling of various Yahoo Finance API error responses"""
    symbol = "AAPL"
    time_range = TimeRange.ONE_MONTH
    interval = Interval.DAILY

    # Mock fetch to return the error response
    with patch('src.services.historical.get_historical.fetch', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.return_value = orjson.dumps(test_case["response"]).decode('utf-8')

        with pytest.raises(HTTPException) as exc_info:
            await get_historical(symbol, time_range, interval)

        assert exc_info.value.status_code == test_case["expected_status"]
        assert exc_info.value.detail == test_case["expected_detail"]


async def test_get_historical_json_decode_error(bypass_cache):
    """Test handling of JSON decode error from API response"""
    symbol = "AAPL"
    time_range = TimeRange.ONE_MONTH
    interval = Interval.DAILY

    # Mock fetch to return invalid JSON
    with patch('src.services.historical.get_historical.fetch', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.return_value = "invalid json response"

        with pytest.raises(HTTPException) as exc_info:
            await get_historical(symbol, time_range, interval)

        assert exc_info.value.status_code == 500
        assert exc_info.value.detail == "Invalid JSON response from Yahoo Finance API"
