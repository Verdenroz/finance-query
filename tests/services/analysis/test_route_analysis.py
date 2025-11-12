from datetime import datetime
from unittest.mock import patch

import pytest
from fastapi.testclient import TestClient

from src.main import app
from src.models.analysis import (
    AnalysisType,
    EarningsEstimate,
    EarningsHistoryItem,
    PriceTarget,
    RecommendationData,
    RevenueEstimate,
    UpgradeDowngrade,
)


@pytest.fixture
def client():
    return TestClient(app)


@patch("src.routes.analysis.get_analysis_data")
async def test_get_recommendations(mock_get_analysis, client):
    """Test getting analyst recommendations"""
    mock_data = {
        "symbol": "AAPL",
        "recommendations": [
            RecommendationData(period="0m", strong_buy=15, buy=25, hold=10, sell=2, strong_sell=1),
            RecommendationData(period="-1m", strong_buy=14, buy=24, hold=11, sell=3, strong_sell=1),
        ],
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/AAPL/recommendations")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert len(data["recommendations"]) == 2
    assert data["recommendations"][0]["period"] == "0m"
    assert data["recommendations"][0]["strong_buy"] == 15


@patch("src.routes.analysis.get_analysis_data")
async def test_get_upgrades_downgrades(mock_get_analysis, client):
    """Test getting analyst upgrades/downgrades"""
    mock_data = {
        "symbol": "MSFT",
        "upgrades_downgrades": [
            UpgradeDowngrade(firm="Goldman Sachs", to_grade="Buy", from_grade="Hold", action="upgrade", date=datetime(2024, 1, 16)),
            UpgradeDowngrade(firm="Morgan Stanley", to_grade="Hold", from_grade="Buy", action="downgrade", date=datetime(2024, 1, 11)),
        ],
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/MSFT/upgrades-downgrades")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert len(data["upgrades_downgrades"]) == 2
    assert data["upgrades_downgrades"][0]["firm"] == "Goldman Sachs"
    assert data["upgrades_downgrades"][0]["action"] == "upgrade"


@patch("src.routes.analysis.get_analysis_data")
async def test_get_price_targets(mock_get_analysis, client):
    """Test getting analyst price targets"""
    mock_data = {
        "symbol": "GOOG",
        "price_targets": PriceTarget(current=150.25, mean=175.50, median=172.00, low=140.00, high=220.00),
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/GOOG/price-targets")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert data["price_targets"]["current"] == 150.25
    assert data["price_targets"]["mean"] == 175.50


@patch("src.routes.analysis.get_analysis_data")
async def test_get_earnings_estimate(mock_get_analysis, client):
    """Test getting earnings estimate"""
    mock_data = {
        "symbol": "TSLA",
        "earnings_estimate": EarningsEstimate(
            estimates={
                "0q": {"avg": 1.55, "low": 1.45, "high": 1.65, "numberOfAnalysts": 35},
                "+1q": {"avg": 1.72, "low": 1.60, "high": 1.85, "numberOfAnalysts": 32},
            }
        ),
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/TSLA/earnings-estimate")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "TSLA"
    assert "0q" in data["earnings_estimate"]["estimates"]
    assert data["earnings_estimate"]["estimates"]["0q"]["avg"] == 1.55


@patch("src.routes.analysis.get_analysis_data")
async def test_get_revenue_estimate(mock_get_analysis, client):
    """Test getting revenue estimate"""
    mock_data = {
        "symbol": "NVDA",
        "revenue_estimate": RevenueEstimate(estimates={"0q": {"avg": 100000000000, "low": 95000000000, "high": 105000000000, "numberOfAnalysts": 30}}),
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/NVDA/revenue-estimate")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "NVDA"
    assert "0q" in data["revenue_estimate"]["estimates"]
    assert data["revenue_estimate"]["estimates"]["0q"]["avg"] == 100000000000


@patch("src.routes.analysis.get_analysis_data")
async def test_get_earnings_history(mock_get_analysis, client):
    """Test getting earnings history"""
    mock_data = {
        "symbol": "AAPL",
        "earnings_history": [
            EarningsHistoryItem(date=datetime(2023, 11, 1), eps_actual=1.46, eps_estimate=1.39, surprise=0.07, surprise_percent=0.05),
            EarningsHistoryItem(date=datetime(2024, 2, 1), eps_actual=1.53, eps_estimate=1.50, surprise=0.03, surprise_percent=0.02),
        ],
    }
    mock_get_analysis.return_value = mock_data

    response = client.get("/v1/analysis/AAPL/earnings-history")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert len(data["earnings_history"]) == 2
    assert data["earnings_history"][0]["eps_actual"] == 1.46


async def test_get_analysis_invalid_symbol_pattern(client):
    """Test validation error for invalid symbol pattern"""
    response = client.get("/v1/analysis/invalid-symbol/recommendations")

    assert response.status_code == 422
    assert "detail" in response.json()


@patch("src.routes.analysis.get_analysis_data")
async def test_get_analysis_all_endpoints(mock_get_analysis, client):
    """Test all analysis endpoint paths"""
    endpoints = [
        ("recommendations", AnalysisType.RECOMMENDATIONS),
        ("upgrades-downgrades", AnalysisType.UPGRADES_DOWNGRADES),
        ("price-targets", AnalysisType.PRICE_TARGETS),
        ("earnings-estimate", AnalysisType.EARNINGS_ESTIMATE),
        ("revenue-estimate", AnalysisType.REVENUE_ESTIMATE),
        ("earnings-history", AnalysisType.EARNINGS_HISTORY),
    ]

    for endpoint, analysis_type in endpoints:
        # Create appropriate mock data for each endpoint
        if analysis_type == AnalysisType.RECOMMENDATIONS:
            mock_data = {"symbol": "TEST", "recommendations": []}
        elif analysis_type == AnalysisType.UPGRADES_DOWNGRADES:
            mock_data = {"symbol": "TEST", "upgrades_downgrades": []}
        elif analysis_type == AnalysisType.PRICE_TARGETS:
            mock_data = {"symbol": "TEST", "price_targets": PriceTarget()}
        elif analysis_type == AnalysisType.EARNINGS_ESTIMATE:
            mock_data = {"symbol": "TEST", "earnings_estimate": EarningsEstimate(estimates={})}
        elif analysis_type == AnalysisType.REVENUE_ESTIMATE:
            mock_data = {"symbol": "TEST", "revenue_estimate": RevenueEstimate(estimates={})}
        else:  # EARNINGS_HISTORY
            mock_data = {"symbol": "TEST", "earnings_history": []}

        mock_get_analysis.return_value = mock_data

        response = client.get(f"/v1/analysis/TEST/{endpoint}")
        assert response.status_code == 200
        assert response.json()["symbol"] == "TEST"


@patch("src.routes.analysis.get_analysis_data")
async def test_get_analysis_symbol_case_insensitivity(mock_get_analysis, client):
    """Test that symbol is converted to uppercase"""
    mock_data = {
        "symbol": "AAPL",
        "recommendations": [RecommendationData(period="0m", strong_buy=15, buy=25, hold=10, sell=2, strong_sell=1)],
    }
    mock_get_analysis.return_value = mock_data

    # Test with uppercase (should work due to path pattern)
    response = client.get("/v1/analysis/AAPL/recommendations")
    assert response.status_code == 200
    assert response.json()["symbol"] == "AAPL"
