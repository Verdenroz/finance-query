from unittest.mock import AsyncMock, patch

import src.routes.technicals
from src.models import TechnicalIndicator, TimeRange, Interval, Indicator
from src.services import get_technical_indicators
from tests.conftest import VERSION

# Mock response constants remain at module level
MOCK_SMA_RESPONSE = TechnicalIndicator(
    type=Indicator.SMA,
    indicators={
        "2023-01-01": {"SMA": 150.0}
    }
)

MOCK_INDICATORS_SUMMARY = {
    "SMA(14)": {"SMA": 150.0},
    "RSI(14)": {"RSI": 65.7},
    "BBANDS(20,2)": {
        "Upper Band": 165.5,
        "Middle Band": 155.0,
        "Lower Band": 144.5
    }
}


class TestTechnicals:
    def test_technical_indicator_success(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test successful technical indicator retrieval"""
        mock_get_indicator = AsyncMock(return_value=MOCK_SMA_RESPONSE)
        monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.SMA, mock_get_indicator)

        symbol = "AAPL"
        function = "SMA"
        time_range = "1y"
        interval = "1d"
        response = test_client.get(
            f"{VERSION}/indicator?function={function}&symbol={symbol}&range={time_range}&interval={interval}"
        )

        assert response.status_code == 200
        mock_get_indicator.assert_awaited_once_with(
            symbol=symbol,
            time_range=TimeRange.YEAR,
            interval=Interval.DAILY,
            epoch=False
        )

    def test_technical_indicator_with_optional_params(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicator with all optional parameters"""
        mock_get_macd = AsyncMock(return_value=TechnicalIndicator(
            type=Indicator.MACD,
            indicators=MOCK_INDICATORS_SUMMARY,
        ))
        monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, Indicator.MACD, mock_get_macd)

        response = test_client.get(
            f"{VERSION}/indicator?function=MACD&symbol=AAPL&range=1y&interval=1d&fastPeriod=4&slowPeriod=30&signalPeriod=1&epoch=true"
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
        error_message = "got an unexpected keyword argument 'invalid_param'"
        mock_get_indicator = AsyncMock(side_effect=TypeError(error_message))
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
        mock_get_indicators = AsyncMock(return_value=MOCK_INDICATORS_SUMMARY)
        monkeypatch.setattr("src.routes.technicals.get_technical_indicators", mock_get_indicators)

        symbol = "AAPL"
        functions = "SMA,RSI,BBANDS"
        response = test_client.get(f"{VERSION}/indicators?symbol={symbol}&functions={functions}")
        data = response.json()

        assert response.status_code == 200
        assert "SMA(14)" in data
        assert "RSI(14)" in data
        assert "BBANDS(20,2)" in data
        assert data["SMA(14)"]["SMA"] == 150.0

        mock_get_indicators.assert_awaited_once_with(
            symbol,
            Interval.DAILY,
            [Indicator.SMA, Indicator.RSI, Indicator.BBANDS]
        )

    def test_technical_indicators_summary_no_functions(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test technical indicators summary without specifying functions"""
        mock_get_indicators = AsyncMock(return_value=MOCK_INDICATORS_SUMMARY)
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

    def test_technical_indicator_with_all_indicator_types(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test all indicator types to ensure routing works for each"""
        mock_response = TechnicalIndicator(
            type=Indicator.SMA,
            indicators={
                "2023-01-01": {"SMA": 150.0}
            }
        )

        for indicator in Indicator:
            mock_indicator = AsyncMock(return_value=mock_response)
            monkeypatch.setitem(src.routes.technicals.IndicatorFunctions, indicator, mock_indicator)

            response = test_client.get(
                f"{VERSION}/indicator?function={indicator.value}&symbol=AAPL&range=1y&interval=1d")

            assert response.status_code == 200, f"Failed for indicator {indicator.value}"
            mock_indicator.reset_mock()

    async def test_get_technical_indicators_comprehensive(self, historical_quotes):
        """Test the actual implementation of get_technical_indicators with all indicators"""
        with patch('src.services.indicators.get_technical_indicators.get_historical',
                   AsyncMock(return_value=historical_quotes)):
            result = await get_technical_indicators('AAPL', Interval.DAILY)

            self._verify_indicator_presence(result)
            await self._test_interval_time_ranges(historical_quotes)

            with patch('src.services.indicators.get_technical_indicators.get_historical',
                       AsyncMock(return_value=historical_quotes)):
                result = await get_technical_indicators('AAPL', Interval.DAILY, [])
                assert len(result) > 10

    def _verify_indicator_presence(self, result):
        """Helper method to verify presence of all indicators"""
        indicators_config = {
            Indicator.SMA: ['SMA(10)', 'SMA(20)', 'SMA(50)', 'SMA(100)', 'SMA(200)'],
            Indicator.EMA: ['EMA(10)', 'EMA(20)', 'EMA(50)', 'EMA(100)', 'EMA(200)'],
            Indicator.WMA: ['WMA(10)', 'WMA(20)', 'WMA(50)', 'WMA(100)', 'WMA(200)'],
            Indicator.VWMA: ['VWMA(20)'],
            Indicator.RSI: ['RSI(14)'],
            Indicator.SRSI: ['SRSI(3,3,14,14)'],
            Indicator.STOCH: ['STOCH %K(14,3,3)'],
            Indicator.CCI: ['CCI(20)'],
            Indicator.MACD: ['MACD(12,26)'],
            Indicator.ADX: ['ADX(14)'],
            Indicator.AROON: ['Aroon(25)'],
            Indicator.BBANDS: ['BBANDS(20,2)'],
            Indicator.SUPER_TREND: ['Super Trend'],
            Indicator.ICHIMOKU: ['Ichimoku Cloud']
        }

        for indicator, expected_values in indicators_config.items():
            for value in expected_values:
                assert value in result

    async def _test_interval_time_ranges(self, historical_quotes):
        """Helper method to test time ranges for different intervals"""
        intervals = [
            Interval.ONE_MINUTE, Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES,
            Interval.THIRTY_MINUTES, Interval.ONE_HOUR, Interval.DAILY
        ]

        for interval in intervals:
            with patch('src.services.indicators.get_technical_indicators.get_historical',
                       AsyncMock(return_value=historical_quotes)) as mock_get_historical:
                subset_indicators = [Indicator.SMA, Indicator.RSI, Indicator.MACD]
                await get_technical_indicators('AAPL', interval, subset_indicators)

                expected_time_range = TimeRange.FIVE_DAYS if interval == Interval.ONE_MINUTE else (
                    TimeRange.ONE_MONTH if interval in [
                        Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES,
                        Interval.THIRTY_MINUTES, Interval.ONE_HOUR
                    ] else TimeRange.FIVE_YEARS
                )

                mock_get_historical.assert_called_once_with(
                    'AAPL', time_range=expected_time_range, interval=interval
                )