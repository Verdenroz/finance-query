from unittest.mock import AsyncMock

from fastapi import HTTPException

from src.models import TimeRange, Interval, HistoricalData
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


def test_all_invalid_combinations(test_client, mock_yahoo_auth, monkeypatch):
    """Test all invalid combinations of interval and time range"""
    # Define all invalid combinations to test
    invalid_combinations = [
        # 1-minute interval only works with 1d and 5d ranges
        {"interval": Interval.ONE_MINUTE, "range": TimeRange.ONE_MONTH,
         "error": "If interval is 1m, range must be 1d, 5d"},
        {"interval": Interval.ONE_MINUTE, "range": TimeRange.THREE_MONTHS,
         "error": "If interval is 1m, range must be 1d, 5d"},
        {"interval": Interval.ONE_MINUTE, "range": TimeRange.YEAR,
         "error": "If interval is 1m, range must be 1d, 5d"},

        # 5-minute interval only works with 1d, 5d, and 1mo ranges
        {"interval": Interval.FIVE_MINUTES, "range": TimeRange.THREE_MONTHS,
         "error": "If interval is 5m, range must be 1d, 5d, 1mo"},
        {"interval": Interval.FIVE_MINUTES, "range": TimeRange.YEAR,
         "error": "If interval is 5m, range must be 1d, 5d, 1mo"},

        # 15-minute interval only works with 1d, 5d, and 1mo ranges
        {"interval": Interval.FIFTEEN_MINUTES, "range": TimeRange.THREE_MONTHS,
         "error": "If interval is 15m, range must be 1d, 5d, 1mo"},
        {"interval": Interval.FIFTEEN_MINUTES, "range": TimeRange.YEAR,
         "error": "If interval is 15m, range must be 1d, 5d, 1mo"},

        # 30-minute interval only works with 1d, 5d, and 1mo ranges
        {"interval": Interval.THIRTY_MINUTES, "range": TimeRange.THREE_MONTHS,
         "error": "If interval is 30m, range must be 1d, 5d, 1mo"},
        {"interval": Interval.THIRTY_MINUTES, "range": TimeRange.YEAR,
         "error": "If interval is 30m, range must be 1d, 5d, 1mo"},

        # 1-hour interval has specific allowed ranges
        {"interval": Interval.ONE_HOUR, "range": TimeRange.FIVE_YEARS,
         "error": "If interval is 1h, range must be 1d, 5d, 1mo, 3mo, 6mo, ytd, 1y"},

        # Max range only works with monthly interval
        {"interval": Interval.DAILY, "range": TimeRange.MAX,
         "error": "If range is max, interval must be 1mo"},
        {"interval": Interval.WEEKLY, "range": TimeRange.MAX,
         "error": "If range is max, interval must be 1mo"}
    ]

    for combo in invalid_combinations:
        # Set up the specific error for this combination
        error_message = combo["error"]
        mock_get_historical = AsyncMock(side_effect=HTTPException(400, error_message))
        monkeypatch.setattr("src.routes.historical_prices.get_historical", mock_get_historical)

        # Make the request with this invalid combination
        response = test_client.get(
            f"{VERSION}/historical?symbol=AAPL&range={combo['range'].value}&interval={combo['interval'].value}"
        )

        # Assertions
        assert response.status_code == 400, f"Failed for {combo['interval'].value} and {combo['range'].value}"
        assert response.json()["detail"] == error_message

        # Verify mock was called
        mock_get_historical.assert_awaited_once()
        mock_get_historical.reset_mock()


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
