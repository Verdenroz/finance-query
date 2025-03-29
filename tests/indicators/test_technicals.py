from unittest.mock import AsyncMock, patch

import numpy as np
import pandas as pd

import src.routes.technicals
from src.models import TechnicalIndicator, TimeRange, Interval, Indicator
from src.services import get_technical_indicators
from tests.conftest import VERSION

# Sample mock response for technical indicator data
MOCK_SMA_RESPONSE = TechnicalIndicator(
    type=Indicator.SMA,
    indicators={
        "2023-01-01": {"SMA": 150.0}
    }
)

# Mock technical indicators summary response
MOCK_INDICATORS_SUMMARY = {
    "SMA(14)": {"SMA": 150.0},
    "RSI(14)": {"RSI": 65.7},
    "BBANDS(20,2)": {
        "Upper Band": 165.5,
        "Middle Band": 155.0,
        "Lower Band": 144.5
    }
}


def test_technical_indicator_success(test_client, mock_yahoo_auth, monkeypatch):
    """Test successful technical indicator retrieval"""
    # Mock the technical indicator function
    mock_get_indicator = AsyncMock(return_value=MOCK_SMA_RESPONSE)

    # Directly replace the function in IndicatorFunctions
    monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.SMA, mock_get_indicator)

    # Make the request
    symbol = "AAPL"
    function = "SMA"
    time_range = "1y"
    interval = "1d"
    response = test_client.get(
        f"{VERSION}/indicator?function={function}&symbol={symbol}&range={time_range}&interval={interval}"
    )

    # Assertions
    assert response.status_code == 200

    # Verify mock was called with correct parameters
    mock_get_indicator.assert_awaited_once_with(
        symbol=symbol,
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
        epoch=False
    )


def test_technical_indicator_with_optional_params(test_client, mock_yahoo_auth, monkeypatch):
    """Test technical indicator with all optional parameters"""
    # Mock the technical indicator function (MACD has multiple parameters)
    mock_get_macd = AsyncMock(return_value=TechnicalIndicator(
        type=Indicator.MACD,
        indicators=MOCK_INDICATORS_SUMMARY,
    ))
    monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.MACD, mock_get_macd)

    # Make the request with all optional MACD parameters
    response = test_client.get(
        f"{VERSION}/indicator?function=MACD&symbol=AAPL&range=1y&interval=1d&fastPeriod=4&slowPeriod=30&signalPeriod=1&epoch=true"
    )

    # Assertions
    assert response.status_code == 200

    # Verify mock was called with correct parameters
    mock_get_macd.assert_awaited_once_with(
        symbol="AAPL",
        time_range=TimeRange.YEAR,
        interval=Interval.DAILY,
        epoch=True,
        fast_period=4,
        slow_period=30,
        signal_period=1
    )


def test_technical_indicator_invalid_parameter(test_client, mock_yahoo_auth, monkeypatch):
    """Test invalid parameter for technical indicator"""
    # Mock the function to raise TypeError
    error_message = "got an unexpected keyword argument 'invalid_param'"
    mock_get_indicator = AsyncMock(side_effect=TypeError(error_message))
    monkeypatch.setattr("src.routes.technicals.get_sma", mock_get_indicator)

    # Make the request
    response = test_client.get(f"{VERSION}/indicator?function=SMA&symbol=AAPL&smooth=3")

    # Assertions
    assert response.status_code == 400
    assert "Invalid parameter" in response.json()["detail"]


def test_technical_indicator_general_exception(test_client, mock_yahoo_auth, monkeypatch):
    """Test when service raises a general exception"""
    # Mock the function to raise an exception
    mock_get_indicator = AsyncMock(side_effect=Exception("Unexpected error"))
    monkeypatch.setattr("src.routes.technicals.get_sma", mock_get_indicator)

    # Make the request
    response = test_client.get(f"{VERSION}/indicator?function=SMA&symbol=AAPL")

    # Assertions
    assert response.status_code == 500
    assert "Invalid JSON response from Yahoo Finance API" in response.json()["detail"]


def test_technical_indicators_summary_success(test_client, mock_yahoo_auth, monkeypatch):
    """Test successful technical indicators summary retrieval"""
    # Mock the technical indicators summary function
    mock_get_indicators = AsyncMock(return_value=MOCK_INDICATORS_SUMMARY)
    monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

    # Make the request
    symbol = "AAPL"
    functions = "SMA,RSI,BBANDS"
    response = test_client.get(f"{VERSION}/indicators?symbol={symbol}&functions={functions}")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert "SMA(14)" in data
    assert "RSI(14)" in data
    assert "BBANDS(20,2)" in data
    assert data["SMA(14)"]["SMA"] == 150.0

    # Verify mock was called with correct parameters
    mock_get_indicators.assert_awaited_once_with(
        symbol,
        Interval.DAILY,
        [Indicator.SMA, Indicator.RSI, Indicator.BBANDS]
    )


def test_technical_indicators_summary_no_functions(test_client, mock_yahoo_auth, monkeypatch):
    """Test technical indicators summary without specifying functions"""
    # Mock the technical indicators summary function
    mock_get_indicators = AsyncMock(return_value=MOCK_INDICATORS_SUMMARY)
    monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

    # Make the request without functions parameter
    response = test_client.get(f"{VERSION}/indicators?symbol=AAPL")

    # Assertions
    assert response.status_code == 200

    # Verify mock was called with None for indicator_list
    mock_get_indicators.assert_awaited_once_with("AAPL", Interval.DAILY, None)


def test_technical_indicators_summary_invalid_function(test_client, mock_yahoo_auth, monkeypatch):
    """Test technical indicators summary with invalid function"""
    # Mock the function to raise KeyError
    mock_get_indicators = AsyncMock(side_effect=KeyError("INVALID"))
    monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

    # Make the request with invalid function
    response = test_client.get(f"{VERSION}/indicators?symbol=AAPL&functions=INVALID")

    # Assertions
    assert response.status_code == 400
    assert "Invalid indicator" in response.json()["detail"]


def test_technical_indicators_summary_exception(test_client, mock_yahoo_auth, monkeypatch):
    """Test technical indicators summary with general exception"""
    # Mock the function to raise an exception
    mock_get_indicators = AsyncMock(side_effect=Exception("Failed to fetch data"))
    monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

    # Make the request
    response = test_client.get(f"{VERSION}/indicators?symbol=AAPL")

    # Assertions
    assert response.status_code == 500
    assert "Failed to retrieve technical analysis" in response.json()["detail"]


def test_technical_indicator_invalid_input_validation(test_client, mock_yahoo_auth):
    """Test input validation for technical indicator"""
    # Make the request with invalid input
    response = test_client.get(f"{VERSION}/indicator?symbol=AAPL")

    # Should return a 422 Unprocessable Entity
    assert response.status_code == 422

    # Check that error contains function is required
    error_data = response.json()
    if "errors" in error_data:
        assert "function" in error_data["errors"]
    else:
        assert any("function" in str(item.get("loc", [])) for item in error_data["detail"])


def test_technical_indicator_with_all_indicator_types(test_client, mock_yahoo_auth, monkeypatch):
    """Test all indicator types to ensure routing works for each"""

    # Define a simple mock response for any indicator
    mock_response = TechnicalIndicator(
        type=Indicator.SMA,
        indicators={
            "2023-01-01": {"SMA": 150.0}
        }
    )

    # Test each indicator in the IndicatorFunctions mapping
    for indicator in Indicator:
        # Create a mock for the specific indicator function
        mock_indicator = AsyncMock(return_value=mock_response)
        # Get the function name from the enum and set it on the technicals module
        monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, indicator, mock_indicator)


        # Make the request
        response = test_client.get(f"{VERSION}/indicator?function={indicator.value}&symbol=AAPL&range=1y&interval=1d")

        # Assert success
        assert response.status_code == 200, f"Failed for indicator {indicator.value}"

        # Reset the mock
        mock_indicator.reset_mock()


async def test_get_technical_indicators_comprehensive(historical_quotes):
    """Test the actual implementation of get_technical_indicators with all indicators"""
    # Mock the get_historical function
    with patch('src.services.indicators.get_technical_indicators.get_historical',
               AsyncMock(return_value=historical_quotes)):

        # Test with default parameters (all indicators)
        result = await get_technical_indicators('AAPL', Interval.DAILY)

        # Verify all indicator types are in the result
        for indicator in Indicator:
            if indicator == Indicator.SMA:
                assert 'SMA(10)' in result
                assert 'SMA(20)' in result
                assert 'SMA(50)' in result
                assert 'SMA(100)' in result
                assert 'SMA(200)' in result
            elif indicator == Indicator.EMA:
                assert 'EMA(10)' in result
                assert 'EMA(20)' in result
                assert 'EMA(50)' in result
                assert 'EMA(100)' in result
                assert 'EMA(200)' in result
            elif indicator == Indicator.WMA:
                assert 'WMA(10)' in result
                assert 'WMA(20)' in result
                assert 'WMA(50)' in result
                assert 'WMA(100)' in result
                assert 'WMA(200)' in result
            elif indicator == Indicator.VWMA:
                assert 'VWMA(20)' in result
            elif indicator == Indicator.RSI:
                assert 'RSI(14)' in result
            elif indicator == Indicator.SRSI:
                assert 'SRSI(3,3,14,14)' in result
            elif indicator == Indicator.STOCH:
                assert 'STOCH %K(14,3,3)' in result
            elif indicator == Indicator.CCI:
                assert 'CCI(20)' in result
            elif indicator == Indicator.MACD:
                assert 'MACD(12,26)' in result
            elif indicator == Indicator.ADX:
                assert 'ADX(14)' in result
            elif indicator == Indicator.AROON:
                assert 'Aroon(25)' in result
            elif indicator == Indicator.BBANDS:
                assert 'BBANDS(20,2)' in result
            elif indicator == Indicator.SUPER_TREND:
                assert 'Super Trend' in result
            elif indicator == Indicator.ICHIMOKU:
                assert 'Ichimoku Cloud' in result

        # Test correct time range selection for different intervals
        for interval in [Interval.ONE_MINUTE, Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES,
                         Interval.THIRTY_MINUTES, Interval.ONE_HOUR, Interval.DAILY]:

            with patch('src.services.indicators.get_technical_indicators.get_historical',
                       AsyncMock(return_value=historical_quotes)) as mock_get_historical:

                # Test with a subset of indicators to check specific calculations
                subset_indicators = [Indicator.SMA, Indicator.RSI, Indicator.MACD]
                await get_technical_indicators('AAPL', interval, subset_indicators)

                # Verify correct time range was used based on interval
                if interval in [Interval.ONE_MINUTE]:
                    expected_time_range = TimeRange.FIVE_DAYS
                elif interval in [Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES,
                                  Interval.THIRTY_MINUTES, Interval.ONE_HOUR]:
                    expected_time_range = TimeRange.ONE_MONTH
                else:
                    expected_time_range = TimeRange.FIVE_YEARS

                mock_get_historical.assert_called_once_with(
                    'AAPL', time_range=expected_time_range, interval=interval)

        # Test empty indicators list
        with patch('src.services.indicators.get_technical_indicators.get_historical',
                   AsyncMock(return_value=historical_quotes)):
            result = await get_technical_indicators('AAPL', Interval.DAILY, [])
            # Should default to all indicators
            assert len(result) > 10  # Basic sanity check that we got multiple indicators