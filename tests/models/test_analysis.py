import pytest
from datetime import datetime
from pydantic import ValidationError

from src.models.analysis import (
    AnalystPriceTargets,
    EarningsEstimate,
    RevenueEstimate,
    EarningsHistory,
    EPSTrend,
    EPSRevisions,
    GrowthEstimates,
    EstimateData,
    EarningsHistoryItem,
    EPSTrendItem,
    EPSRevisionItem,
    GrowthEstimateItem,
)


class TestAnalystPriceTargets:
    """Test AnalystPriceTargets model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "symbol": "AAPL",
            "current": 150.0,
            "low": 140.0,
            "high": 160.0,
            "mean": 150.0,
            "median": 150.0,
        }
        model = AnalystPriceTargets(**data)
        
        assert model.symbol == "AAPL"
        assert model.current == 150.0
        assert model.low == 140.0
        assert model.high == 160.0
        assert model.mean == 150.0
        assert model.median == 150.0

    def test_partial_data(self):
        """Test with partial data (some fields None)"""
        data = {
            "symbol": "AAPL",
            "current": 150.0,
            "low": None,
            "high": 160.0,
            "mean": None,
            "median": 150.0,
        }
        model = AnalystPriceTargets(**data)
        
        assert model.symbol == "AAPL"
        assert model.current == 150.0
        assert model.low is None
        assert model.high == 160.0
        assert model.mean is None
        assert model.median == 150.0

    def test_all_none(self):
        """Test with all fields None"""
        data = {
            "symbol": "AAPL",
            "current": None,
            "low": None,
            "high": None,
            "mean": None,
            "median": None,
        }
        model = AnalystPriceTargets(**data)
        
        assert model.symbol == "AAPL"
        assert model.current is None
        assert model.low is None
        assert model.high is None
        assert model.mean is None
        assert model.median is None

    def test_required_symbol(self):
        """Test that symbol is required"""
        data = {
            "current": 150.0,
            "low": 140.0,
            "high": 160.0,
            "mean": 150.0,
            "median": 150.0,
        }
        
        with pytest.raises(ValidationError) as exc_info:
            AnalystPriceTargets(**data)
        
        assert "symbol" in str(exc_info.value)

    def test_negative_values(self):
        """Test with negative values"""
        data = {
            "symbol": "AAPL",
            "current": -10.0,
            "low": -20.0,
            "high": -5.0,
            "mean": -10.0,
            "median": -10.0,
        }
        model = AnalystPriceTargets(**data)
        
        assert model.symbol == "AAPL"
        assert model.current == -10.0
        assert model.low == -20.0
        assert model.high == -5.0
        assert model.mean == -10.0
        assert model.median == -10.0


class TestEstimateData:
    """Test EstimateData model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "period": "0q",
            "number_of_analysts": 25,
            "avg": 2.50,
            "low": 2.00,
            "high": 3.00,
            "year_ago_eps": 2.20,
            "growth": 0.136,
        }
        model = EstimateData(**data)
        
        assert model.period == "0q"
        assert model.number_of_analysts == 25
        assert model.avg == 2.50
        assert model.low == 2.00
        assert model.high == 3.00
        assert model.year_ago_eps == 2.20
        assert model.growth == 0.136

    def test_partial_data(self):
        """Test with partial data"""
        data = {
            "period": "+1q",
            "number_of_analysts": None,
            "avg": 2.75,
            "low": None,
            "high": 3.50,
            "year_ago_eps": None,
            "growth": None,
        }
        model = EstimateData(**data)
        
        assert model.period == "+1q"
        assert model.number_of_analysts is None
        assert model.avg == 2.75
        assert model.low is None
        assert model.high == 3.50
        assert model.year_ago_eps is None
        assert model.growth is None

    def test_required_period(self):
        """Test that period is required"""
        data = {
            "number_of_analysts": 25,
            "avg": 2.50,
        }
        
        with pytest.raises(ValidationError) as exc_info:
            EstimateData(**data)
        
        assert "period" in str(exc_info.value)


class TestEarningsEstimate:
    """Test EarningsEstimate model"""

    def test_valid_data(self):
        """Test with valid data"""
        estimates = [
            EstimateData(
                period="0q",
                number_of_analysts=25,
                avg=2.50,
                low=2.00,
                high=3.00,
                year_ago_eps=2.20,
                growth=0.136
            ),
            EstimateData(
                period="+1q",
                number_of_analysts=20,
                avg=2.75,
                low=2.25,
                high=3.25,
                year_ago_eps=2.50,
                growth=0.100
            )
        ]
        
        model = EarningsEstimate(symbol="AAPL", estimates=estimates)
        
        assert model.symbol == "AAPL"
        assert len(model.estimates) == 2
        assert model.estimates[0].period == "0q"
        assert model.estimates[0].avg == 2.50
        assert model.estimates[1].period == "+1q"
        assert model.estimates[1].avg == 2.75

    def test_empty_estimates(self):
        """Test with empty estimates list"""
        model = EarningsEstimate(symbol="AAPL", estimates=[])
        
        assert model.symbol == "AAPL"
        assert len(model.estimates) == 0

    def test_required_symbol(self):
        """Test that symbol is required"""
        estimates = [EstimateData(period="0q", avg=2.50)]
        
        with pytest.raises(ValidationError) as exc_info:
            EarningsEstimate(estimates=estimates)
        
        assert "symbol" in str(exc_info.value)


class TestRevenueEstimate:
    """Test RevenueEstimate model"""

    def test_valid_data(self):
        """Test with valid data"""
        estimates = [
            EstimateData(
                period="0y",
                number_of_analysts=20,
                avg=1000000000.0,
                low=950000000.0,
                high=1050000000.0,
                year_ago_eps=900000000.0,
                growth=0.111
            )
        ]
        
        model = RevenueEstimate(symbol="AAPL", estimates=estimates)
        
        assert model.symbol == "AAPL"
        assert len(model.estimates) == 1
        assert model.estimates[0].period == "0y"
        assert model.estimates[0].avg == 1000000000.0


class TestEarningsHistoryItem:
    """Test EarningsHistoryItem model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "quarter": "2024-03-31",
            "eps_estimate": 2.50,
            "eps_actual": 2.75,
            "eps_difference": 0.25,
            "surprise_percent": 10.0,
        }
        model = EarningsHistoryItem(**data)
        
        assert model.quarter == "2024-03-31"
        assert model.eps_estimate == 2.50
        assert model.eps_actual == 2.75
        assert model.eps_difference == 0.25
        assert model.surprise_percent == 10.0

    def test_required_quarter(self):
        """Test that quarter is required"""
        data = {
            "eps_estimate": 2.50,
            "eps_actual": 2.75,
        }
        
        with pytest.raises(ValidationError) as exc_info:
            EarningsHistoryItem(**data)
        
        assert "quarter" in str(exc_info.value)


class TestEarningsHistory:
    """Test EarningsHistory model"""

    def test_valid_data(self):
        """Test with valid data"""
        history_items = [
            EarningsHistoryItem(
                quarter="2024-03-31",
                eps_estimate=2.50,
                eps_actual=2.75,
                eps_difference=0.25,
                surprise_percent=10.0
            ),
            EarningsHistoryItem(
                quarter="2023-12-31",
                eps_estimate=2.20,
                eps_actual=2.30,
                eps_difference=0.10,
                surprise_percent=4.55
            )
        ]
        
        model = EarningsHistory(symbol="AAPL", history=history_items)
        
        assert model.symbol == "AAPL"
        assert len(model.history) == 2
        assert model.history[0].quarter == "2024-03-31"
        assert model.history[0].eps_actual == 2.75
        assert model.history[1].quarter == "2023-12-31"
        assert model.history[1].eps_actual == 2.30

    def test_empty_history(self):
        """Test with empty history list"""
        model = EarningsHistory(symbol="AAPL", history=[])
        
        assert model.symbol == "AAPL"
        assert len(model.history) == 0


class TestEPSTrendItem:
    """Test EPSTrendItem model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "period": "0q",
            "current": 2.50,
            "seven_days_ago": 2.45,
            "thirty_days_ago": 2.40,
            "sixty_days_ago": 2.35,
            "ninety_days_ago": 2.30,
        }
        model = EPSTrendItem(**data)
        
        assert model.period == "0q"
        assert model.current == 2.50
        assert model.seven_days_ago == 2.45
        assert model.thirty_days_ago == 2.40
        assert model.sixty_days_ago == 2.35
        assert model.ninety_days_ago == 2.30

    def test_required_period(self):
        """Test that period is required"""
        data = {
            "current": 2.50,
            "seven_days_ago": 2.45,
        }
        
        with pytest.raises(ValidationError) as exc_info:
            EPSTrendItem(**data)
        
        assert "period" in str(exc_info.value)


class TestEPSTrend:
    """Test EPSTrend model"""

    def test_valid_data(self):
        """Test with valid data"""
        trend_items = [
            EPSTrendItem(
                period="0q",
                current=2.50,
                seven_days_ago=2.45,
                thirty_days_ago=2.40,
                sixty_days_ago=2.35,
                ninety_days_ago=2.30
            ),
            EPSTrendItem(
                period="+1q",
                current=2.75,
                seven_days_ago=2.70,
                thirty_days_ago=2.65,
                sixty_days_ago=2.60,
                ninety_days_ago=2.55
            )
        ]
        
        model = EPSTrend(symbol="AAPL", trends=trend_items)
        
        assert model.symbol == "AAPL"
        assert len(model.trends) == 2
        assert model.trends[0].period == "0q"
        assert model.trends[0].current == 2.50
        assert model.trends[1].period == "+1q"
        assert model.trends[1].current == 2.75


class TestEPSRevisionItem:
    """Test EPSRevisionItem model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "period": "0q",
            "up_last_7days": 5,
            "up_last_30days": 12,
            "down_last_7days": 2,
            "down_last_30days": 8,
        }
        model = EPSRevisionItem(**data)
        
        assert model.period == "0q"
        assert model.up_last_7days == 5
        assert model.up_last_30days == 12
        assert model.down_last_7days == 2
        assert model.down_last_30days == 8

    def test_zero_values(self):
        """Test with zero values"""
        data = {
            "period": "0y",
            "up_last_7days": 0,
            "up_last_30days": 0,
            "down_last_7days": 0,
            "down_last_30days": 0,
        }
        model = EPSRevisionItem(**data)
        
        assert model.up_last_7days == 0
        assert model.up_last_30days == 0
        assert model.down_last_7days == 0
        assert model.down_last_30days == 0


class TestEPSRevisions:
    """Test EPSRevisions model"""

    def test_valid_data(self):
        """Test with valid data"""
        revision_items = [
            EPSRevisionItem(
                period="0q",
                up_last_7days=5,
                up_last_30days=12,
                down_last_7days=2,
                down_last_30days=8
            ),
            EPSRevisionItem(
                period="+1q",
                up_last_7days=3,
                up_last_30days=10,
                down_last_7days=1,
                down_last_30days=6
            )
        ]
        
        model = EPSRevisions(symbol="AAPL", revisions=revision_items)
        
        assert model.symbol == "AAPL"
        assert len(model.revisions) == 2
        assert model.revisions[0].period == "0q"
        assert model.revisions[0].up_last_7days == 5
        assert model.revisions[1].period == "+1q"
        assert model.revisions[1].up_last_7days == 3


class TestGrowthEstimateItem:
    """Test GrowthEstimateItem model"""

    def test_valid_data(self):
        """Test with valid data"""
        data = {
            "period": "+5y",
            "stock": 0.15,
            "industry": 0.12,
            "sector": 0.10,
            "index": 0.08,
        }
        model = GrowthEstimateItem(**data)
        
        assert model.period == "+5y"
        assert model.stock == 0.15
        assert model.industry == 0.12
        assert model.sector == 0.10
        assert model.index == 0.08

    def test_negative_growth(self):
        """Test with negative growth values"""
        data = {
            "period": "0y",
            "stock": -0.10,
            "industry": -0.08,
            "sector": -0.05,
            "index": -0.03,
        }
        model = GrowthEstimateItem(**data)
        
        assert model.stock == -0.10
        assert model.industry == -0.08
        assert model.sector == -0.05
        assert model.index == -0.03


class TestGrowthEstimates:
    """Test GrowthEstimates model"""

    def test_valid_data(self):
        """Test with valid data"""
        estimate_items = [
            GrowthEstimateItem(
                period="+5y",
                stock=0.15,
                industry=0.12,
                sector=0.10,
                index=0.08
            ),
            GrowthEstimateItem(
                period="-5y",
                stock=-0.05,
                industry=-0.03,
                sector=-0.02,
                index=-0.01
            )
        ]
        
        model = GrowthEstimates(symbol="AAPL", estimates=estimate_items)
        
        assert model.symbol == "AAPL"
        assert len(model.estimates) == 2
        assert model.estimates[0].period == "+5y"
        assert model.estimates[0].stock == 0.15
        assert model.estimates[1].period == "-5y"
        assert model.estimates[1].stock == -0.05


class TestModelSerialization:
    """Test model serialization and deserialization"""

    def test_analyst_price_targets_serialization(self):
        """Test AnalystPriceTargets serialization"""
        model = AnalystPriceTargets(
            symbol="AAPL",
            current=150.0,
            low=140.0,
            high=160.0,
            mean=150.0,
            median=150.0
        )
        
        data = model.model_dump()
        assert data["symbol"] == "AAPL"
        assert data["current"] == 150.0
        assert data["low"] == 140.0
        assert data["high"] == 160.0
        assert data["mean"] == 150.0
        assert data["median"] == 150.0

    def test_earnings_estimate_serialization(self):
        """Test EarningsEstimate serialization"""
        estimates = [
            EstimateData(
                period="0q",
                number_of_analysts=25,
                avg=2.50,
                low=2.00,
                high=3.00,
                year_ago_eps=2.20,
                growth=0.136
            )
        ]
        model = EarningsEstimate(symbol="AAPL", estimates=estimates)
        
        data = model.model_dump()
        assert data["symbol"] == "AAPL"
        assert len(data["estimates"]) == 1
        assert data["estimates"][0]["period"] == "0q"
        assert data["estimates"][0]["avg"] == 2.50

    def test_earnings_history_serialization(self):
        """Test EarningsHistory serialization"""
        history_items = [
            EarningsHistoryItem(
                quarter="2024-03-31",
                eps_estimate=2.50,
                eps_actual=2.75,
                eps_difference=0.25,
                surprise_percent=10.0
            )
        ]
        model = EarningsHistory(symbol="AAPL", history=history_items)
        
        data = model.model_dump()
        assert data["symbol"] == "AAPL"
        assert len(data["history"]) == 1
        assert data["history"][0]["quarter"] == "2024-03-31"
        assert data["history"][0]["eps_estimate"] == 2.50

    def test_model_json_serialization(self):
        """Test JSON serialization"""
        model = AnalystPriceTargets(symbol="AAPL", current=150.0, high=160.0)
        json_str = model.model_dump_json()
        
        # Should be valid JSON
        import json
        data = json.loads(json_str)
        assert data["symbol"] == "AAPL"
        assert data["current"] == 150.0
        assert data["high"] == 160.0
        assert data["low"] is None