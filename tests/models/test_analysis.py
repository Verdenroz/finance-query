from datetime import datetime

import pytest
from pydantic import ValidationError

from src.models.analysis import (
    AnalysisData,
    AnalysisType,
    EarningsEstimate,
    EarningsHistoryItem,
    PriceTarget,
    RecommendationData,
    RevenueEstimate,
    SustainabilityScores,
    UpgradeDowngrade,
)


class TestAnalysisType:
    """Test suite for AnalysisType enum."""

    def test_analysis_type_values(self):
        """Test that AnalysisType has all expected values"""
        expected_values = [
            "recommendations",
            "upgrades_downgrades",
            "price_targets",
            "earnings_estimate",
            "revenue_estimate",
            "earnings_history",
            "sustainability",
        ]

        actual_values = [e.value for e in AnalysisType]
        assert set(actual_values) == set(expected_values)

    def test_analysis_type_enum_behavior(self):
        """Test AnalysisType enum behavior"""
        assert AnalysisType.RECOMMENDATIONS == "recommendations"
        assert AnalysisType.PRICE_TARGETS == "price_targets"
        assert AnalysisType.SUSTAINABILITY == "sustainability"


class TestRecommendationData:
    """Test suite for RecommendationData model."""

    def test_recommendation_data_valid(self):
        """Test RecommendationData with valid data"""
        data = RecommendationData(period="3m", strong_buy=5, buy=10, hold=3, sell=1, strong_sell=0)

        assert data.period == "3m"
        assert data.strong_buy == 5
        assert data.buy == 10
        assert data.hold == 3
        assert data.sell == 1
        assert data.strong_sell == 0

    def test_recommendation_data_with_none_values(self):
        """Test RecommendationData with None values"""
        data = RecommendationData(period="3m", strong_buy=None, buy=10, hold=None, sell=1, strong_sell=0)

        assert data.period == "3m"
        assert data.strong_buy is None
        assert data.buy == 10
        assert data.hold is None
        assert data.sell == 1
        assert data.strong_sell == 0

    def test_recommendation_data_minimal(self):
        """Test RecommendationData with minimal required data"""
        data = RecommendationData(period="3m")

        assert data.period == "3m"
        assert data.strong_buy is None
        assert data.buy is None
        assert data.hold is None
        assert data.sell is None
        assert data.strong_sell is None


class TestUpgradeDowngrade:
    """Test suite for UpgradeDowngrade model."""

    def test_upgrade_downgrade_valid(self):
        """Test UpgradeDowngrade with valid data"""
        data = UpgradeDowngrade(firm="Goldman Sachs", to_grade="Buy", from_grade="Hold", action="upgrade", date=datetime(2024, 1, 15))

        assert data.firm == "Goldman Sachs"
        assert data.to_grade == "Buy"
        assert data.from_grade == "Hold"
        assert data.action == "upgrade"
        assert data.date == datetime(2024, 1, 15)

    def test_upgrade_downgrade_with_none_values(self):
        """Test UpgradeDowngrade with None values"""
        data = UpgradeDowngrade(firm="Goldman Sachs", to_grade=None, from_grade="Hold", action=None, date=None)

        assert data.firm == "Goldman Sachs"
        assert data.to_grade is None
        assert data.from_grade == "Hold"
        assert data.action is None
        assert data.date is None

    def test_upgrade_downgrade_minimal(self):
        """Test UpgradeDowngrade with minimal required data"""
        data = UpgradeDowngrade(firm="Goldman Sachs")

        assert data.firm == "Goldman Sachs"
        assert data.to_grade is None
        assert data.from_grade is None
        assert data.action is None
        assert data.date is None


class TestPriceTarget:
    """Test suite for PriceTarget model."""

    def test_price_target_valid(self):
        """Test PriceTarget with valid data"""
        data = PriceTarget(current=150.0, mean=160.0, median=155.0, low=140.0, high=180.0)

        assert data.current == 150.0
        assert data.mean == 160.0
        assert data.median == 155.0
        assert data.low == 140.0
        assert data.high == 180.0

    def test_price_target_with_none_values(self):
        """Test PriceTarget with None values"""
        data = PriceTarget(current=150.0, mean=None, median=155.0, low=None, high=180.0)

        assert data.current == 150.0
        assert data.mean is None
        assert data.median == 155.0
        assert data.low is None
        assert data.high == 180.0

    def test_price_target_empty(self):
        """Test PriceTarget with all None values"""
        data = PriceTarget()

        assert data.current is None
        assert data.mean is None
        assert data.median is None
        assert data.low is None
        assert data.high is None


class TestEarningsEstimate:
    """Test suite for EarningsEstimate model."""

    def test_earnings_estimate_valid(self):
        """Test EarningsEstimate with valid data"""
        estimates = {"2024-12-31": {"avg": 6.5, "low": 6.0, "high": 7.0}, "2025-12-31": {"avg": 7.2, "low": 6.8, "high": 7.6}}
        data = EarningsEstimate(estimates=estimates)

        assert data.estimates == estimates
        assert "2024-12-31" in data.estimates
        assert "2025-12-31" in data.estimates
        assert data.estimates["2024-12-31"]["avg"] == 6.5

    def test_earnings_estimate_empty(self):
        """Test EarningsEstimate with empty estimates"""
        data = EarningsEstimate(estimates={})

        assert data.estimates == {}


class TestRevenueEstimate:
    """Test suite for RevenueEstimate model."""

    def test_revenue_estimate_valid(self):
        """Test RevenueEstimate with valid data"""
        estimates = {
            "2024-12-31": {"avg": 400000000000, "low": 380000000000, "high": 420000000000},
            "2025-12-31": {"avg": 420000000000, "low": 400000000000, "high": 440000000000},
        }
        data = RevenueEstimate(estimates=estimates)

        assert data.estimates == estimates
        assert "2024-12-31" in data.estimates
        assert "2025-12-31" in data.estimates
        assert data.estimates["2024-12-31"]["avg"] == 400000000000

    def test_revenue_estimate_empty(self):
        """Test RevenueEstimate with empty estimates"""
        data = RevenueEstimate(estimates={})

        assert data.estimates == {}


class TestEarningsHistoryItem:
    """Test suite for EarningsHistoryItem model."""

    def test_earnings_history_item_valid(self):
        """Test EarningsHistoryItem with valid data"""
        data = EarningsHistoryItem(date=datetime(2024, 1, 15), eps_actual=2.18, eps_estimate=2.10, surprise=0.08, surprise_percent=3.8)

        assert data.date == datetime(2024, 1, 15)
        assert data.eps_actual == 2.18
        assert data.eps_estimate == 2.10
        assert data.surprise == 0.08
        assert data.surprise_percent == 3.8

    def test_earnings_history_item_with_none_values(self):
        """Test EarningsHistoryItem with None values"""
        data = EarningsHistoryItem(date=datetime(2024, 1, 15), eps_actual=None, eps_estimate=2.10, surprise=None, surprise_percent=3.8)

        assert data.date == datetime(2024, 1, 15)
        assert data.eps_actual is None
        assert data.eps_estimate == 2.10
        assert data.surprise is None
        assert data.surprise_percent == 3.8

    def test_earnings_history_item_minimal(self):
        """Test EarningsHistoryItem with minimal required data"""
        data = EarningsHistoryItem(date=datetime(2024, 1, 15))

        assert data.date == datetime(2024, 1, 15)
        assert data.eps_actual is None
        assert data.eps_estimate is None
        assert data.surprise is None
        assert data.surprise_percent is None


class TestSustainabilityScores:
    """Test suite for SustainabilityScores model."""

    def test_sustainability_scores_valid(self):
        """Test SustainabilityScores with valid data"""
        scores = {"environmentScore": 75, "socialScore": 80, "governanceScore": 85, "totalEsg": 80}
        data = SustainabilityScores(scores=scores)

        assert data.scores == scores
        assert data.scores["environmentScore"] == 75
        assert data.scores["socialScore"] == 80
        assert data.scores["governanceScore"] == 85
        assert data.scores["totalEsg"] == 80

    def test_sustainability_scores_empty(self):
        """Test SustainabilityScores with empty scores"""
        data = SustainabilityScores(scores={})

        assert data.scores == {}


class TestAnalysisData:
    """Test suite for AnalysisData model."""

    def test_analysis_data_recommendations(self):
        """Test AnalysisData with recommendations type"""
        recommendations = [RecommendationData(period="3m", strong_buy=5, buy=10, hold=3, sell=1, strong_sell=0)]

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.RECOMMENDATIONS, recommendations=recommendations)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.RECOMMENDATIONS
        assert data.recommendations == recommendations
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_price_targets(self):
        """Test AnalysisData with price targets type"""
        price_targets = PriceTarget(current=150.0, mean=160.0, median=155.0, low=140.0, high=180.0)

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.PRICE_TARGETS, price_targets=price_targets)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.PRICE_TARGETS
        assert data.price_targets == price_targets
        assert data.recommendations is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_earnings_estimate(self):
        """Test AnalysisData with earnings estimate type"""
        earnings_estimate = EarningsEstimate(estimates={"2024-12-31": {"avg": 6.5, "low": 6.0, "high": 7.0}})

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.EARNINGS_ESTIMATE, earnings_estimate=earnings_estimate)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.EARNINGS_ESTIMATE
        assert data.earnings_estimate == earnings_estimate
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_revenue_estimate(self):
        """Test AnalysisData with revenue estimate type"""
        revenue_estimate = RevenueEstimate(estimates={"2024-12-31": {"avg": 400000000000, "low": 380000000000, "high": 420000000000}})

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.REVENUE_ESTIMATE, revenue_estimate=revenue_estimate)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.REVENUE_ESTIMATE
        assert data.revenue_estimate == revenue_estimate
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_earnings_history(self):
        """Test AnalysisData with earnings history type"""
        earnings_history = [EarningsHistoryItem(date=datetime(2024, 1, 15), eps_actual=2.18, eps_estimate=2.10, surprise=0.08, surprise_percent=3.8)]

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.EARNINGS_HISTORY, earnings_history=earnings_history)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.EARNINGS_HISTORY
        assert data.earnings_history == earnings_history
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_sustainability(self):
        """Test AnalysisData with sustainability type"""
        sustainability = SustainabilityScores(scores={"environmentScore": 75, "socialScore": 80, "governanceScore": 85})

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.SUSTAINABILITY, sustainability=sustainability)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.SUSTAINABILITY
        assert data.sustainability == sustainability
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_upgrades_downgrades(self):
        """Test AnalysisData with upgrades/downgrades type"""
        upgrades_downgrades = [UpgradeDowngrade(firm="Goldman Sachs", to_grade="Buy", from_grade="Hold", action="upgrade", date=datetime(2024, 1, 15))]

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.UPGRADES_DOWNGRADES, upgrades_downgrades=upgrades_downgrades)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.UPGRADES_DOWNGRADES
        assert data.upgrades_downgrades == upgrades_downgrades
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None

    def test_analysis_data_minimal(self):
        """Test AnalysisData with minimal required data"""
        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.RECOMMENDATIONS)

        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.RECOMMENDATIONS
        assert data.recommendations is None
        assert data.price_targets is None
        assert data.earnings_estimate is None
        assert data.revenue_estimate is None
        assert data.earnings_history is None
        assert data.sustainability is None
        assert data.upgrades_downgrades is None

    def test_analysis_data_validation_error(self):
        """Test AnalysisData with invalid data"""
        with pytest.raises(ValidationError):
            AnalysisData(
                symbol="",  # Empty symbol should fail validation
                analysis_type=AnalysisType.RECOMMENDATIONS,
            )

    def test_analysis_data_json_serialization(self):
        """Test AnalysisData JSON serialization"""
        recommendations = [RecommendationData(period="3m", strong_buy=5, buy=10, hold=3, sell=1, strong_sell=0)]

        data = AnalysisData(symbol="AAPL", analysis_type=AnalysisType.RECOMMENDATIONS, recommendations=recommendations)

        # Test that the model can be serialized to JSON
        json_data = data.model_dump()
        assert json_data["symbol"] == "AAPL"
        assert json_data["analysis_type"] == "recommendations"
        assert len(json_data["recommendations"]) == 1
        assert json_data["recommendations"][0]["period"] == "3m"

    def test_analysis_data_json_deserialization(self):
        """Test AnalysisData JSON deserialization"""
        json_data = {
            "symbol": "AAPL",
            "analysis_type": "recommendations",
            "recommendations": [{"period": "3m", "strong_buy": 5, "buy": 10, "hold": 3, "sell": 1, "strong_sell": 0}],
        }

        data = AnalysisData.model_validate(json_data)
        assert data.symbol == "AAPL"
        assert data.analysis_type == AnalysisType.RECOMMENDATIONS
        assert len(data.recommendations) == 1
        assert data.recommendations[0].period == "3m"
        assert data.recommendations[0].strong_buy == 5
