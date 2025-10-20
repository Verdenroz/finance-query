import pytest
import pandas as pd
from unittest.mock import AsyncMock, patch, MagicMock
from datetime import datetime
from fastapi import HTTPException

from src.services.analysis.fetchers.analysis_fetcher import (
    fetch_analyst_price_targets,
    fetch_earnings_estimate,
    fetch_revenue_estimate,
    fetch_earnings_history,
    fetch_eps_trend,
    fetch_eps_revisions,
    fetch_growth_estimates,
)
from src.models.analysis import (
    AnalystPriceTargets,
    EarningsEstimate,
    RevenueEstimate,
    EarningsHistory,
    EPSTrend,
    EPSRevisions,
    GrowthEstimates,
)


class TestFetchAnalystPriceTargets:
    """Test fetch_analyst_price_targets fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of analyst price targets"""
        # Mock Ticker instance
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        # Mock analyst_price_targets property
        mock_ticker.analyst_price_targets = {
            "current": 150.0,
            "low": 140.0,
            "high": 160.0,
            "mean": 150.0,
            "median": 150.0
        }

        result = await fetch_analyst_price_targets("AAPL")

        assert isinstance(result, AnalystPriceTargets)
        assert result.current == 150.0
        assert result.low == 140.0
        assert result.high == 160.0
        assert result.mean == 150.0
        assert result.median == 150.0
        mock_ticker_class.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_empty_data(self, mock_ticker_class):
        """Test handling of empty data"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets = {}

        result = await fetch_analyst_price_targets("EMPTY")

        assert isinstance(result, AnalystPriceTargets)
        assert result.current is None
        assert result.low is None
        assert result.high is None
        assert result.mean is None
        assert result.median is None

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_none_data(self, mock_ticker_class):
        """Test handling of None data"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets = None

        with pytest.raises(HTTPException) as exc_info:
            await fetch_analyst_price_targets("INVALID")

        assert exc_info.value.status_code == 404
        assert "Analyst price targets not found" in exc_info.value.detail

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_partial_data(self, mock_ticker_class):
        """Test handling of partial data"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets = {
            "current": 150.0,
            "low": None,
            "high": 160.0,
            "mean": None,
            "median": 150.0
        }

        result = await fetch_analyst_price_targets("PARTIAL")

        assert isinstance(result, AnalystPriceTargets)
        assert result.current == 150.0
        assert result.low is None
        assert result.high == 160.0
        assert result.mean is None
        assert result.median == 150.0


class TestFetchEarningsEstimate:
    """Test fetch_earnings_estimate fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of earnings estimates"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        # Mock DataFrame
        mock_df = pd.DataFrame({
            '0q': {'numberOfAnalysts': 25, 'avg': 2.50, 'low': 2.00, 'high': 3.00, 'yearAgoEps': 2.20, 'growth': 0.136},
            '+1q': {'numberOfAnalysts': 20, 'avg': 2.75, 'low': 2.25, 'high': 3.25, 'yearAgoEps': 2.50, 'growth': 0.100}
        })
        mock_ticker.earnings_estimate = mock_df

        result = await fetch_earnings_estimate("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], EarningsEstimate)
        assert result[0].period == '0q'
        assert result[0].number_of_analysts == 25
        assert result[0].avg == 2.50
        assert result[1].period == '+1q'
        assert result[1].avg == 2.75

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_empty_dataframe(self, mock_ticker_class):
        """Test handling of empty DataFrame"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.earnings_estimate = pd.DataFrame()

        result = await fetch_earnings_estimate("EMPTY")

        assert isinstance(result, list)
        assert len(result) == 0

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_dataframe_with_nan_values(self, mock_ticker_class):
        """Test handling of DataFrame with NaN values"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        mock_df = pd.DataFrame({
            '0q': {'numberOfAnalysts': 25, 'avg': 2.50, 'low': None, 'high': 3.00, 'yearAgoEps': None, 'growth': 0.136}
        })
        mock_ticker.earnings_estimate = mock_df

        result = await fetch_earnings_estimate("AAPL")

        assert isinstance(result, list)
        assert len(result) == 1
        assert result[0].period == '0q'
        assert result[0].number_of_analysts == 25
        assert result[0].avg == 2.50
        assert result[0].low is None
        assert result[0].high == 3.00
        assert result[0].year_ago_eps is None
        assert result[0].growth == 0.136


class TestFetchRevenueEstimate:
    """Test fetch_revenue_estimate fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of revenue estimates"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        mock_df = pd.DataFrame({
            '0y': {'numberOfAnalysts': 20, 'avg': 1000000000.0, 'low': 950000000.0, 'high': 1050000000.0, 'yearAgoRevenue': 900000000.0, 'growth': 0.111},
            '+1y': {'numberOfAnalysts': 15, 'avg': 1100000000.0, 'low': 1000000000.0, 'high': 1200000000.0, 'yearAgoRevenue': 1000000000.0, 'growth': 0.100}
        })
        mock_ticker.revenue_estimate = mock_df

        result = await fetch_revenue_estimate("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], RevenueEstimate)
        assert result[0].period == '0y'
        assert result[0].avg == 1000000000.0
        assert result[1].period == '+1y'
        assert result[1].avg == 1100000000.0


class TestFetchEarningsHistory:
    """Test fetch_earnings_history fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of earnings history"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        # Mock DataFrame with datetime index
        dates = [datetime(2024, 3, 31), datetime(2023, 12, 31)]
        mock_df = pd.DataFrame({
            'epsEstimate': [2.50, 2.20],
            'epsActual': [2.75, 2.30],
            'epsDifference': [0.25, 0.10],
            'surprisePercent': [10.0, 4.55]
        }, index=dates)
        mock_ticker.earnings_history = mock_df

        result = await fetch_earnings_history("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], EarningsHistory)
        assert result[0].quarter == dates[0]
        assert result[0].eps_estimate == 2.50
        assert result[0].eps_actual == 2.75
        assert result[1].quarter == dates[1]
        assert result[1].eps_estimate == 2.20

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_empty_dataframe(self, mock_ticker_class):
        """Test handling of empty DataFrame"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.earnings_history = pd.DataFrame()

        result = await fetch_earnings_history("EMPTY")

        assert isinstance(result, list)
        assert len(result) == 0


class TestFetchEPSTrend:
    """Test fetch_eps_trend fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of EPS trend"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        mock_df = pd.DataFrame({
            '0q': {'current': 2.50, '7daysAgo': 2.45, '30daysAgo': 2.40, '60daysAgo': 2.35, '90daysAgo': 2.30},
            '+1q': {'current': 2.75, '7daysAgo': 2.70, '30daysAgo': 2.65, '60daysAgo': 2.60, '90daysAgo': 2.55}
        })
        mock_ticker.eps_trend = mock_df

        result = await fetch_eps_trend("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], EPSTrend)
        assert result[0].period == '0q'
        assert result[0].current == 2.50
        assert result[0].seven_days_ago == 2.45
        assert result[1].period == '+1q'
        assert result[1].current == 2.75


class TestFetchEPSRevisions:
    """Test fetch_eps_revisions fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of EPS revisions"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        mock_df = pd.DataFrame({
            '0q': {'upLast7days': 5, 'upLast30days': 12, 'downLast7days': 2, 'downLast30days': 8},
            '+1q': {'upLast7days': 3, 'upLast30days': 10, 'downLast7days': 1, 'downLast30days': 6}
        })
        mock_ticker.eps_revisions = mock_df

        result = await fetch_eps_revisions("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], EPSRevisions)
        assert result[0].period == '0q'
        assert result[0].up_last_7_days == 5
        assert result[0].up_last_30_days == 12
        assert result[1].period == '+1q'
        assert result[1].up_last_7_days == 3


class TestFetchGrowthEstimates:
    """Test fetch_growth_estimates fetcher"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_success(self, mock_ticker_class):
        """Test successful fetching of growth estimates"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        mock_df = pd.DataFrame({
            '+5y': {'stockTrend': 0.15, 'industryTrend': 0.12, 'sectorTrend': 0.10, 'indexTrend': 0.08},
            '-5y': {'stockTrend': -0.05, 'industryTrend': -0.03, 'sectorTrend': -0.02, 'indexTrend': -0.01}
        })
        mock_ticker.growth_estimates = mock_df

        result = await fetch_growth_estimates("AAPL")

        assert isinstance(result, list)
        assert len(result) == 2
        assert isinstance(result[0], GrowthEstimates)
        assert result[0].period == '+5y'
        assert result[0].stock_trend == 0.15
        assert result[0].industry_trend == 0.12
        assert result[1].period == '-5y'
        assert result[1].stock_trend == -0.05


class TestFetcherErrorHandling:
    """Test error handling in fetchers"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_ticker_initialization_error(self, mock_ticker_class):
        """Test handling of Ticker initialization errors"""
        mock_ticker_class.side_effect = Exception("Ticker initialization failed")

        with pytest.raises(Exception) as exc_info:
            await fetch_analyst_price_targets("ERROR")

        assert str(exc_info.value) == "Ticker initialization failed"

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_property_access_error(self, mock_ticker_class):
        """Test handling of property access errors"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets.side_effect = Exception("Property access failed")

        with pytest.raises(Exception) as exc_info:
            await fetch_analyst_price_targets("ERROR")

        assert str(exc_info.value) == "Property access failed"

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_dataframe_conversion_error(self, mock_ticker_class):
        """Test handling of DataFrame conversion errors"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.earnings_estimate = "invalid_data"

        with pytest.raises(Exception):
            await fetch_earnings_estimate("ERROR")


class TestFetcherDataTypes:
    """Test data type handling in fetchers"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_numeric_data_types(self, mock_ticker_class):
        """Test handling of different numeric data types"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets = {
            "current": 150,  # int
            "low": 140.5,    # float
            "high": 160,     # int
            "mean": 150.0,   # float
            "median": 150    # int
        }

        result = await fetch_analyst_price_targets("AAPL")

        assert isinstance(result, AnalystPriceTargets)
        assert result.current == 150.0  # Should be converted to float
        assert result.low == 140.5
        assert result.high == 160.0
        assert result.mean == 150.0
        assert result.median == 150.0

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_string_numeric_conversion(self, mock_ticker_class):
        """Test handling of string numeric values"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        mock_ticker.analyst_price_targets = {
            "current": "150.0",  # string
            "low": "140.5",      # string
            "high": "160",       # string
            "mean": "150.0",     # string
            "median": "150"      # string
        }

        result = await fetch_analyst_price_targets("AAPL")

        assert isinstance(result, AnalystPriceTargets)
        assert result.current == 150.0
        assert result.low == 140.5
        assert result.high == 160.0
        assert result.mean == 150.0
        assert result.median == 150.0


class TestFetcherIntegration:
    """Test fetcher integration scenarios"""

    @pytest.mark.asyncio
    @patch("src.services.analysis.fetchers.analysis_fetcher.Ticker")
    async def test_multiple_fetchers_same_ticker(self, mock_ticker_class):
        """Test calling multiple fetchers with the same ticker"""
        mock_ticker = MagicMock()
        mock_ticker_class.return_value = mock_ticker
        
        # Setup mock data for different properties
        mock_ticker.analyst_price_targets = {"current": 150.0}
        mock_ticker.earnings_estimate = pd.DataFrame({'0q': {'avg': 2.50}})
        mock_ticker.revenue_estimate = pd.DataFrame({'0y': {'avg': 1000000000.0}})

        price_result = await fetch_analyst_price_targets("AAPL")
        earnings_result = await fetch_earnings_estimate("AAPL")
        revenue_result = await fetch_revenue_estimate("AAPL")

        assert isinstance(price_result, AnalystPriceTargets)
        assert isinstance(earnings_result, list)
        assert isinstance(revenue_result, list)
        assert price_result.current == 150.0
        assert earnings_result[0].avg == 2.50
        assert revenue_result[0].avg == 1000000000.0

    @pytest.mark.asyncio
    async def test_fetcher_functions_exist(self):
        """Test that all fetcher functions exist and are callable"""
        fetchers = [
            fetch_analyst_price_targets,
            fetch_earnings_estimate,
            fetch_revenue_estimate,
            fetch_earnings_history,
            fetch_eps_trend,
            fetch_eps_revisions,
            fetch_growth_estimates,
        ]

        for fetcher_func in fetchers:
            assert callable(fetcher_func)
            assert fetcher_func.__name__.startswith("fetch_")

