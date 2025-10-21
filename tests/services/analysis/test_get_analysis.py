import pytest
from unittest.mock import MagicMock, patch
import pandas as pd
from datetime import datetime
from fastapi import HTTPException

from src.services.analysis.get_analysis import (
    get_analysis_data,
    _parse_recommendations,
    _parse_upgrades_downgrades,
    _parse_price_targets,
    _parse_earnings_estimate,
    _parse_revenue_estimate,
    _parse_earnings_history,
    _parse_sustainability,
)
from src.models.analysis import AnalysisType


class TestGetAnalysisData:
    """Test suite for the get_analysis_data service."""

    @pytest.mark.asyncio
    async def test_get_analysis_data_recommendations(self):
        """Test get_analysis_data with recommendations type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.recommendations = MagicMock()
            mock_ticker_instance.recommendations.empty = False
            mock_ticker_instance.recommendations.iterrows.return_value = [
                (0, {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0})
            ]
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.RECOMMENDATIONS)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.RECOMMENDATIONS
            assert result.recommendations is not None
            assert len(result.recommendations) == 1
            assert result.recommendations[0].period == "3m"
            assert result.recommendations[0].strong_buy == 5

    @pytest.mark.asyncio
    async def test_get_analysis_data_price_targets(self):
        """Test get_analysis_data with price targets type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.analyst_price_targets = {
                "current": 150.0,
                "mean": 160.0,
                "median": 155.0,
                "low": 140.0,
                "high": 180.0
            }
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.PRICE_TARGETS)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.PRICE_TARGETS
            assert result.price_targets is not None
            assert result.price_targets.current == 150.0
            assert result.price_targets.mean == 160.0

    @pytest.mark.asyncio
    async def test_get_analysis_data_earnings_estimate(self):
        """Test get_analysis_data with earnings estimate type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_df = pd.DataFrame({
                "2024-12-31": {"avg": 6.5, "low": 6.0, "high": 7.0},
                "2025-12-31": {"avg": 7.2, "low": 6.8, "high": 7.6}
            })
            mock_ticker_instance.earnings_estimate = mock_df
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.EARNINGS_ESTIMATE)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.EARNINGS_ESTIMATE
            assert result.earnings_estimate is not None
            assert "estimates" in result.earnings_estimate.model_dump()

    @pytest.mark.asyncio
    async def test_get_analysis_data_sustainability(self):
        """Test get_analysis_data with sustainability type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.sustainability = MagicMock()
            mock_ticker_instance.sustainability.empty = False
            mock_ticker_instance.sustainability.columns = ["environmentScore", "socialScore", "governanceScore"]
            mock_ticker_instance.sustainability.__getitem__.side_effect = lambda x: {
                "environmentScore": 75,
                "socialScore": 80,
                "governanceScore": 85
            }[x]
            mock_ticker_instance.sustainability.iloc = [75, 80, 85]
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.SUSTAINABILITY)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.SUSTAINABILITY
            assert result.sustainability is not None
            assert "scores" in result.sustainability.model_dump()

    @pytest.mark.asyncio
    async def test_get_analysis_data_upgrades_downgrades(self):
        """Test get_analysis_data with upgrades/downgrades type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.upgrades_downgrades = MagicMock()
            mock_ticker_instance.upgrades_downgrades.empty = False
            mock_ticker_instance.upgrades_downgrades.iterrows.return_value = [
                (0, {"firm": "Goldman Sachs", "toGrade": "Buy", "fromGrade": "Hold", "action": "upgrade", "date": "2024-01-15"})
            ]
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.UPGRADES_DOWNGRADES)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.UPGRADES_DOWNGRADES
            assert result.upgrades_downgrades is not None
            assert len(result.upgrades_downgrades) == 1
            assert result.upgrades_downgrades[0].firm == "Goldman Sachs"

    @pytest.mark.asyncio
    async def test_get_analysis_data_earnings_history(self):
        """Test get_analysis_data with earnings history type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.earnings_history = MagicMock()
            mock_ticker_instance.earnings_history.empty = False
            mock_ticker_instance.earnings_history.iterrows.return_value = [
                (0, {"date": "2024-01-15", "eps_actual": 2.18, "eps_estimate": 2.10, "surprise": 0.08, "surprise_percent": 3.8})
            ]
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.EARNINGS_HISTORY)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.EARNINGS_HISTORY
            assert result.earnings_history is not None
            assert len(result.earnings_history) == 1
            assert result.earnings_history[0].eps_actual == 2.18

    @pytest.mark.asyncio
    async def test_get_analysis_data_revenue_estimate(self):
        """Test get_analysis_data with revenue estimate type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            # Mock the yfinance Ticker object
            mock_ticker_instance = MagicMock()
            mock_df = pd.DataFrame({
                "2024-12-31": {"avg": 400000000000, "low": 380000000000, "high": 420000000000}
            })
            mock_ticker_instance.revenue_estimate = mock_df
            mock_ticker.return_value = mock_ticker_instance

            result = await get_analysis_data("AAPL", AnalysisType.REVENUE_ESTIMATE)

            assert result.symbol == "AAPL"
            assert result.analysis_type == AnalysisType.REVENUE_ESTIMATE
            assert result.revenue_estimate is not None
            assert "estimates" in result.revenue_estimate.model_dump()

    @pytest.mark.asyncio
    async def test_get_analysis_data_invalid_type(self):
        """Test get_analysis_data with invalid analysis type"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            mock_ticker_instance = MagicMock()
            mock_ticker.return_value = mock_ticker_instance

            with pytest.raises(HTTPException) as exc_info:
                await get_analysis_data("AAPL", "invalid_type")
            
            assert exc_info.value.status_code == 400
            assert "Invalid analysis type" in exc_info.value.detail

    @pytest.mark.asyncio
    async def test_get_analysis_data_yfinance_error(self):
        """Test get_analysis_data with yfinance error"""
        with patch("src.services.analysis.get_analysis.yf.Ticker") as mock_ticker:
            mock_ticker_instance = MagicMock()
            mock_ticker_instance.recommendations = MagicMock()
            mock_ticker_instance.recommendations.empty = False
            mock_ticker_instance.recommendations.iterrows.side_effect = Exception("Yahoo Finance API error")
            mock_ticker.return_value = mock_ticker_instance

            with pytest.raises(HTTPException):  # Should raise HTTPException
                await get_analysis_data("AAPL", AnalysisType.RECOMMENDATIONS)


class TestParseRecommendations:
    """Test suite for _parse_recommendations function."""

    def test_parse_recommendations_empty_dataframe(self):
        """Test parsing empty recommendations DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_recommendations(empty_df)
        assert result == []

    def test_parse_recommendations_with_data(self):
        """Test parsing recommendations DataFrame with data"""
        df = pd.DataFrame([
            {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0},
            {"period": "1m", "strongBuy": 3, "buy": 8, "hold": 5, "sell": 2, "strongSell": 1}
        ])
        result = _parse_recommendations(df)
        
        assert len(result) == 2
        assert result[0].period == "3m"
        assert result[0].strong_buy == 5
        assert result[0].buy == 10
        assert result[1].period == "1m"
        assert result[1].strong_sell == 1

    def test_parse_recommendations_with_nan_values(self):
        """Test parsing recommendations DataFrame with NaN values"""
        df = pd.DataFrame([
            {"period": "3m", "strongBuy": 5, "buy": pd.NA, "hold": 3, "sell": 1, "strongSell": 0}
        ])
        result = _parse_recommendations(df)
        
        assert len(result) == 1
        assert result[0].period == "3m"
        assert result[0].strong_buy == 5
        assert result[0].buy is None  # NaN should be converted to None


class TestParseUpgradesDowngrades:
    """Test suite for _parse_upgrades_downgrades function."""

    def test_parse_upgrades_downgrades_empty_dataframe(self):
        """Test parsing empty upgrades/downgrades DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_upgrades_downgrades(empty_df)
        assert result == []

    def test_parse_upgrades_downgrades_with_data(self):
        """Test parsing upgrades/downgrades DataFrame with data"""
        df = pd.DataFrame([
            {"firm": "Goldman Sachs", "toGrade": "Buy", "fromGrade": "Hold", "action": "upgrade", "date": "2024-01-15"},
            {"firm": "Morgan Stanley", "toGrade": "Hold", "fromGrade": "Buy", "action": "downgrade", "date": "2024-01-10"}
        ])
        result = _parse_upgrades_downgrades(df)
        
        assert len(result) == 2
        assert result[0].firm == "Goldman Sachs"
        assert result[0].action == "upgrade"
        assert result[1].firm == "Morgan Stanley"
        assert result[1].action == "downgrade"

    def test_parse_upgrades_downgrades_with_nan_values(self):
        """Test parsing upgrades/downgrades DataFrame with NaN values"""
        df = pd.DataFrame([
            {"firm": "Goldman Sachs", "toGrade": pd.NA, "fromGrade": "Hold", "action": "upgrade", "date": pd.NA}
        ])
        result = _parse_upgrades_downgrades(df)
        
        assert len(result) == 1
        assert result[0].firm == "Goldman Sachs"
        assert result[0].to_grade is None
        assert result[0].date is None


class TestParsePriceTargets:
    """Test suite for _parse_price_targets function."""

    def test_parse_price_targets_dict_format(self):
        """Test parsing price targets in dict format"""
        data = {
            "current": 150.0,
            "mean": 160.0,
            "median": 155.0,
            "low": 140.0,
            "high": 180.0
        }
        result = _parse_price_targets(data)
        
        assert result.current == 150.0
        assert result.mean == 160.0
        assert result.median == 155.0
        assert result.low == 140.0
        assert result.high == 180.0

    def test_parse_price_targets_series_format(self):
        """Test parsing price targets in Series format"""
        series = pd.Series({
            "current": 150.0,
            "mean": 160.0,
            "median": 155.0,
            "low": 140.0,
            "high": 180.0
        })
        result = _parse_price_targets(series)
        
        assert result.current == 150.0
        assert result.mean == 160.0
        assert result.median == 155.0

    def test_parse_price_targets_none_data(self):
        """Test parsing price targets with None data"""
        result = _parse_price_targets(None)
        
        assert result.current is None
        assert result.mean is None
        assert result.median is None

    def test_parse_price_targets_empty_dataframe(self):
        """Test parsing price targets with empty DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_price_targets(empty_df)
        
        assert result.current is None
        assert result.mean is None
        assert result.median is None

    def test_parse_price_targets_with_nan_values(self):
        """Test parsing price targets with NaN values"""
        data = {
            "current": 150.0,
            "mean": pd.NA,
            "median": 155.0,
            "low": 140.0,
            "high": 180.0
        }
        result = _parse_price_targets(data)
        
        assert result.current == 150.0
        assert result.mean is None  # NaN should be converted to None
        assert result.median == 155.0


class TestParseEarningsEstimate:
    """Test suite for _parse_earnings_estimate function."""

    def test_parse_earnings_estimate_empty_dataframe(self):
        """Test parsing empty earnings estimate DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_earnings_estimate(empty_df)
        
        assert result.estimates == {}

    def test_parse_earnings_estimate_with_data(self):
        """Test parsing earnings estimate DataFrame with data"""
        df = pd.DataFrame({
            "2024-12-31": [6.5, 6.0, 7.0],
            "2025-12-31": [7.2, 6.8, 7.6]
        }, index=["avg", "low", "high"])
        
        result = _parse_earnings_estimate(df)
        
        assert "2024-12-31" in result.estimates
        assert "2025-12-31" in result.estimates


class TestParseRevenueEstimate:
    """Test suite for _parse_revenue_estimate function."""

    def test_parse_revenue_estimate_empty_dataframe(self):
        """Test parsing empty revenue estimate DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_revenue_estimate(empty_df)
        
        assert result.estimates == {}

    def test_parse_revenue_estimate_with_data(self):
        """Test parsing revenue estimate DataFrame with data"""
        df = pd.DataFrame({
            "2024-12-31": [400000000000, 380000000000, 420000000000],
            "2025-12-31": [420000000000, 400000000000, 440000000000]
        }, index=["avg", "low", "high"])
        
        result = _parse_revenue_estimate(df)
        
        assert "2024-12-31" in result.estimates
        assert "2025-12-31" in result.estimates


class TestParseEarningsHistory:
    """Test suite for _parse_earnings_history function."""

    def test_parse_earnings_history_empty_dataframe(self):
        """Test parsing empty earnings history DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_earnings_history(empty_df)
        assert result == []

    def test_parse_earnings_history_with_data(self):
        """Test parsing earnings history DataFrame with data"""
        df = pd.DataFrame([
            {"date": "2024-01-15", "eps_actual": 2.18, "eps_estimate": 2.10, "surprise": 0.08, "surprise_percent": 3.8},
            {"date": "2023-10-15", "eps_actual": 1.46, "eps_estimate": 1.39, "surprise": 0.07, "surprise_percent": 5.0}
        ])
        result = _parse_earnings_history(df)
        
        assert len(result) == 2
        assert result[0].eps_actual == 2.18
        assert result[0].surprise_percent == 3.8
        assert result[1].eps_actual == 1.46

    def test_parse_earnings_history_with_nan_values(self):
        """Test parsing earnings history DataFrame with NaN values"""
        df = pd.DataFrame([
            {"date": "2024-01-15", "eps_actual": 2.18, "eps_estimate": pd.NA, "surprise": 0.08, "surprise_percent": pd.NA}
        ])
        result = _parse_earnings_history(df)
        
        assert len(result) == 1
        assert result[0].eps_actual == 2.18
        assert result[0].eps_estimate is None
        assert result[0].surprise_percent is None


class TestParseSustainability:
    """Test suite for _parse_sustainability function."""

    def test_parse_sustainability_empty_dataframe(self):
        """Test parsing empty sustainability DataFrame"""
        empty_df = pd.DataFrame()
        result = _parse_sustainability(empty_df)
        
        assert result.scores == {}

    def test_parse_sustainability_with_data(self):
        """Test parsing sustainability DataFrame with data"""
        df = pd.DataFrame({
            "environmentScore": [75],
            "socialScore": [80],
            "governanceScore": [85]
        })
        result = _parse_sustainability(df)
        
        assert "environmentScore" in result.scores
        assert "socialScore" in result.scores
        assert "governanceScore" in result.scores
        assert result.scores["environmentScore"] == 75

    def test_parse_sustainability_with_nan_values(self):
        """Test parsing sustainability DataFrame with NaN values"""
        df = pd.DataFrame({
            "environmentScore": [75],
            "socialScore": [pd.NA],
            "governanceScore": [85]
        })
        result = _parse_sustainability(df)
        
        assert result.scores["environmentScore"] == 75
        assert result.scores["socialScore"] is None
        assert result.scores["governanceScore"] == 85
