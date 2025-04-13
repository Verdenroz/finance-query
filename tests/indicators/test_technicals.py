# tests/test_technicals.py
from unittest.mock import AsyncMock

import src.routes.technicals
from src.models import TechnicalIndicator, TimeRange, Interval, Indicator
from tests.conftest import VERSION


class TestTechnicals:
    def test_technical_indicator_success(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test successful technical indicator retrieval"""
        mock_get_indicator = AsyncMock(return_value=TechnicalIndicator(
            type=Indicator.SMA,
            indicators={"2023-01-01": {"SMA": 150.0}}
        ))
        monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.SMA, mock_get_indicator)

        response = test_client.get(
            f"{VERSION}/indicator?function=SMA&symbol=AAPL&range=1y&interval=1d"
        )
        assert response.status_code == 200
        mock_get_indicator.assert_awaited_once_with(
            symbol="AAPL",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
            epoch=False
        )

    def test_technical_indicator_with_optional_params(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicator with all optional parameters"""
        mock_get_macd = AsyncMock(return_value=TechnicalIndicator(
            type=Indicator.MACD,
            indicators={"SMA(14)": {"SMA": 150.0}, "RSI(14)": {"RSI": 65.7}}
        ))
        monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.MACD, mock_get_macd)

        response = test_client.get(
            f"{VERSION}/indicator?function=MACD&symbol=AAPL&range=1y&interval=1d"
            "&fastPeriod=4&slowPeriod=30&signalPeriod=1&epoch=true"
        )
        assert response.status_code == 200
        mock_get_macd.assert_awaited_once_with(
            symbol="AAPL",
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
            epoch=True,
            fast_period=4,
            slow_period=30,
            signal_period=1
        )

    def test_technical_indicator_invalid_parameter(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test invalid parameter for technical indicator"""
        mock_get_indicator = AsyncMock(side_effect=TypeError("got an unexpected keyword argument"))
        monkeypatch.setattr("src.routes.technicals.get_sma", mock_get_indicator)

        response = test_client.get(f"{VERSION}/indicator?function=SMA&symbol=AAPL&smooth=3")
        assert response.status_code == 400
        assert "Invalid parameter" in response.json()["detail"]

    def test_technical_indicator_general_exception(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test when service raises a general exception"""
        mock_get_indicator = AsyncMock(side_effect=Exception("Unexpected error"))
        monkeypatch.setattr("src.routes.technicals.get_sma", mock_get_indicator)

        response = test_client.get(f"{VERSION}/indicator?function=SMA&symbol=AAPL")
        assert response.status_code == 500
        assert "Invalid JSON response from Yahoo Finance API" in response.json()["detail"]

    def test_technical_indicators_summary_success(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test successful technical indicators summary retrieval"""
        mock_get_indicators = AsyncMock(return_value={
            "SMA(14)": {"SMA": 150.0},
            "RSI(14)": {"RSI": 65.7},
            "BBANDS(20,2)": {"Upper Band": 165.5, "Middle Band": 155.0, "Lower Band": 144.5}
        })
        monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

        response = test_client.get(f"{VERSION}/indicators?symbol=AAPL&functions=SMA,RSI,BBANDS")
        data = response.json()
        assert response.status_code == 200
        assert "SMA(14)" in data and data["SMA(14)"]["SMA"] == 150.0
        mock_get_indicators.assert_awaited_once_with("AAPL", Interval.DAILY, [Indicator.SMA, Indicator.RSI, Indicator.BBANDS])

    def test_technical_indicators_summary_no_functions(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicators summary without specifying functions"""
        mock_get_indicators = AsyncMock(return_value={})
        monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

        response = test_client.get(f"{VERSION}/indicators?symbol=AAPL")
        assert response.status_code == 200
        mock_get_indicators.assert_awaited_once_with("AAPL", Interval.DAILY, None)

    def test_technical_indicators_summary_invalid_function(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicators summary with invalid function"""
        mock_get_indicators = AsyncMock(side_effect=KeyError("INVALID"))
        monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

        response = test_client.get(f"{VERSION}/indicators?symbol=AAPL&functions=INVALID")
        assert response.status_code == 400
        assert "Invalid indicator" in response.json()["detail"]

    def test_technical_indicators_summary_exception(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicators summary with general exception"""
        mock_get_indicators = AsyncMock(side_effect=Exception("Failed to fetch data"))
        monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

        response = test_client.get(f"{VERSION}/indicators?symbol=AAPL")
        assert response.status_code == 500
        assert "Failed to retrieve technical analysis" in response.json()["detail"]

    def test_technical_indicator_invalid_input_validation(self, test_client, mock_yahoo_auth):
        """Test input validation for technical indicator"""
        response = test_client.get(f"{VERSION}/indicator?symbol=AAPL")
        assert response.status_code == 422
        error_data = response.json()
        if "errors" in error_data:
            assert "function" in error_data["errors"]
        else:
            assert any("function" in str(item.get("loc", [])) for item in error_data["detail"])
