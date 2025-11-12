from datetime import datetime
from unittest.mock import AsyncMock

import pytest
from fastapi import HTTPException

from src.models.analysis import AnalysisType
from src.services.analysis.get_analysis import (
    _parse_earnings_estimate,
    _parse_earnings_history,
    _parse_price_targets,
    _parse_recommendations,
    _parse_revenue_estimate,
    _parse_upgrades_downgrades,
    _safe_extract_value,
    get_analysis_data,
)


@pytest.fixture
def mock_finance_client():
    """Create a mock YahooFinanceClient"""
    return AsyncMock()


@pytest.fixture
def sample_recommendations_response():
    """Sample Yahoo Finance quoteSummary API response for recommendations"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "recommendationTrend": {
                        "trend": [
                            {
                                "period": "0m",
                                "strongBuy": 15,
                                "buy": 25,
                                "hold": 10,
                                "sell": 2,
                                "strongSell": 1,
                            },
                            {
                                "period": "-1m",
                                "strongBuy": 14,
                                "buy": 24,
                                "hold": 11,
                                "sell": 3,
                                "strongSell": 1,
                            },
                        ]
                    }
                }
            ]
        }
    }


@pytest.fixture
def sample_upgrades_downgrades_response():
    """Sample Yahoo Finance quoteSummary API response for upgrades/downgrades"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "upgradeDowngradeHistory": {
                        "history": [
                            {
                                "firm": "Goldman Sachs",
                                "toGrade": "Buy",
                                "fromGrade": "Hold",
                                "action": "upgrade",
                                "epochGradeDate": 1705363200,  # 2024-01-16
                            },
                            {
                                "firm": "Morgan Stanley",
                                "toGrade": "Hold",
                                "fromGrade": "Buy",
                                "action": "downgrade",
                                "epochGradeDate": 1704931200,  # 2024-01-11
                            },
                        ]
                    }
                }
            ]
        }
    }


@pytest.fixture
def sample_price_targets_response():
    """Sample Yahoo Finance quoteSummary API response for price targets"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "financialData": {
                        "currentPrice": {"raw": 150.25, "fmt": "$150.25"},
                        "targetMeanPrice": {"raw": 175.50, "fmt": "$175.50"},
                        "targetMedianPrice": {"raw": 172.00, "fmt": "$172.00"},
                        "targetLowPrice": {"raw": 140.00, "fmt": "$140.00"},
                        "targetHighPrice": {"raw": 220.00, "fmt": "$220.00"},
                    }
                }
            ]
        }
    }


@pytest.fixture
def sample_earnings_estimate_response():
    """Sample Yahoo Finance quoteSummary API response for earnings estimate"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "earningsTrend": {
                        "trend": [
                            {
                                "period": "0q",
                                "earningsEstimate": {
                                    "avg": {"raw": 1.55},
                                    "low": {"raw": 1.45},
                                    "high": {"raw": 1.65},
                                    "numberOfAnalysts": {"raw": 35},
                                    "yearAgoEps": {"raw": 1.42},
                                    "growth": {"raw": 0.092},
                                },
                            },
                            {
                                "period": "+1q",
                                "earningsEstimate": {
                                    "avg": {"raw": 1.72},
                                    "low": {"raw": 1.60},
                                    "high": {"raw": 1.85},
                                    "numberOfAnalysts": {"raw": 32},
                                    "yearAgoEps": {"raw": 1.58},
                                    "growth": {"raw": 0.089},
                                },
                            },
                        ]
                    }
                }
            ]
        }
    }


@pytest.fixture
def sample_earnings_history_response():
    """Sample Yahoo Finance quoteSummary API response for earnings history"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "earningsHistory": {
                        "history": [
                            {
                                "quarter": {"raw": 1698796800},  # 2023-11-01
                                "epsActual": {"raw": 1.46},
                                "epsEstimate": {"raw": 1.39},
                                "epsDifference": {"raw": 0.07},
                                "surprisePercent": {"raw": 0.05},
                            },
                            {
                                "quarter": {"raw": 1706745600},  # 2024-02-01
                                "epsActual": {"raw": 1.53},
                                "epsEstimate": {"raw": 1.50},
                                "epsDifference": {"raw": 0.03},
                                "surprisePercent": {"raw": 0.02},
                            },
                        ]
                    }
                }
            ]
        }
    }


# Test helper function
def test_safe_extract_value_dict():
    """Test safe value extraction from dict with 'raw' key"""
    value = {"raw": 123.45, "fmt": "$123.45"}
    assert _safe_extract_value(value) == 123.45


def test_safe_extract_value_numeric():
    """Test safe value extraction from numeric values"""
    assert _safe_extract_value(100.5) == 100.5
    assert _safe_extract_value(42) == 42.0


def test_safe_extract_value_none():
    """Test safe value extraction from None"""
    assert _safe_extract_value(None) is None


# Test parser functions
def test_parse_recommendations():
    """Test parsing recommendations list from Yahoo Finance API"""
    trend_list = [
        {"period": "0m", "strongBuy": 15, "buy": 25, "hold": 10, "sell": 2, "strongSell": 1},
        {"period": "-1m", "strongBuy": 14, "buy": 24, "hold": 11, "sell": 3, "strongSell": 1},
    ]

    result = _parse_recommendations(trend_list)

    assert len(result) == 2
    assert result[0].period == "0m"
    assert result[0].strong_buy == 15
    assert result[0].buy == 25
    assert result[1].period == "-1m"
    assert result[1].strong_sell == 1


def test_parse_recommendations_empty():
    """Test parsing empty recommendations list"""
    result = _parse_recommendations([])
    assert result == []


def test_parse_upgrades_downgrades():
    """Test parsing upgrades/downgrades list from Yahoo Finance API"""
    history_list = [
        {
            "firm": "Goldman Sachs",
            "toGrade": "Buy",
            "fromGrade": "Hold",
            "action": "upgrade",
            "epochGradeDate": 1705363200,
        },
        {
            "firm": "Morgan Stanley",
            "toGrade": "Hold",
            "fromGrade": "Buy",
            "action": "downgrade",
            "epochGradeDate": 1704931200,
        },
    ]

    result = _parse_upgrades_downgrades(history_list)

    assert len(result) == 2
    assert result[0].firm == "Goldman Sachs"
    assert result[0].to_grade == "Buy"
    assert result[0].action == "upgrade"
    assert result[0].date == datetime.fromtimestamp(1705363200)
    assert result[1].firm == "Morgan Stanley"
    assert result[1].action == "downgrade"


def test_parse_upgrades_downgrades_empty():
    """Test parsing empty upgrades/downgrades list"""
    result = _parse_upgrades_downgrades([])
    assert result == []


def test_parse_price_targets():
    """Test parsing analyst price targets from Yahoo Finance API"""
    data = {
        "currentPrice": {"raw": 150.25},
        "targetMeanPrice": {"raw": 175.50},
        "targetMedianPrice": {"raw": 172.00},
        "targetLowPrice": {"raw": 140.00},
        "targetHighPrice": {"raw": 220.00},
    }

    result = _parse_price_targets(data)

    assert result.current == 150.25
    assert result.mean == 175.50
    assert result.median == 172.00
    assert result.low == 140.00
    assert result.high == 220.00


def test_parse_price_targets_empty():
    """Test parsing empty price targets data"""
    result = _parse_price_targets({})
    assert result.current is None
    assert result.mean is None


def test_parse_earnings_estimate():
    """Test parsing earnings estimate from Yahoo Finance API"""
    trend_list = [
        {
            "period": "0q",
            "earningsEstimate": {
                "avg": {"raw": 1.55},
                "low": {"raw": 1.45},
                "high": {"raw": 1.65},
                "numberOfAnalysts": {"raw": 35},
                "yearAgoEps": {"raw": 1.42},
                "growth": {"raw": 0.092},
            },
        },
        {
            "period": "+1q",
            "earningsEstimate": {
                "avg": {"raw": 1.72},
                "low": {"raw": 1.60},
                "high": {"raw": 1.85},
                "numberOfAnalysts": {"raw": 32},
            },
        },
    ]

    result = _parse_earnings_estimate(trend_list)

    assert "0q" in result.estimates
    assert "+1q" in result.estimates
    assert result.estimates["0q"]["avg"] == 1.55
    assert result.estimates["0q"]["low"] == 1.45
    assert result.estimates["0q"]["numberOfAnalysts"] == 35
    assert result.estimates["+1q"]["avg"] == 1.72


def test_parse_earnings_estimate_empty():
    """Test parsing empty earnings estimate list"""
    result = _parse_earnings_estimate([])
    assert result.estimates == {}


def test_parse_revenue_estimate():
    """Test parsing revenue estimate from Yahoo Finance API"""
    trend_list = [
        {
            "period": "0q",
            "revenueEstimate": {
                "avg": {"raw": 100000000000},
                "low": {"raw": 95000000000},
                "high": {"raw": 105000000000},
                "numberOfAnalysts": {"raw": 30},
                "yearAgoRevenue": {"raw": 92000000000},
                "growth": {"raw": 0.087},
            },
        }
    ]

    result = _parse_revenue_estimate(trend_list)

    assert "0q" in result.estimates
    assert result.estimates["0q"]["avg"] == 100000000000
    assert result.estimates["0q"]["low"] == 95000000000
    assert result.estimates["0q"]["growth"] == 0.087


def test_parse_revenue_estimate_empty():
    """Test parsing empty revenue estimate list"""
    result = _parse_revenue_estimate([])
    assert result.estimates == {}


def test_parse_earnings_history():
    """Test parsing earnings history from Yahoo Finance API"""
    history_list = [
        {
            "quarter": {"raw": 1698796800},
            "epsActual": {"raw": 1.46},
            "epsEstimate": {"raw": 1.39},
            "epsDifference": {"raw": 0.07},
            "surprisePercent": {"raw": 0.05},
        },
        {
            "quarter": {"raw": 1706745600},
            "epsActual": {"raw": 1.53},
            "epsEstimate": {"raw": 1.50},
            "epsDifference": {"raw": 0.03},
            "surprisePercent": {"raw": 0.02},
        },
    ]

    result = _parse_earnings_history(history_list)

    assert len(result) == 2
    assert result[0].eps_actual == 1.46
    assert result[0].eps_estimate == 1.39
    assert result[0].surprise == 0.07
    assert result[0].surprise_percent == 0.05
    assert result[0].date == datetime.fromtimestamp(1698796800)
    assert result[1].eps_actual == 1.53


def test_parse_earnings_history_empty():
    """Test parsing empty earnings history list"""
    result = _parse_earnings_history([])
    assert result == []


# Integration tests with mocked finance client


async def test_get_analysis_data_recommendations(mock_finance_client, sample_recommendations_response, bypass_cache):
    """Test getting recommendations analysis"""
    mock_finance_client.get_quote_summary.return_value = sample_recommendations_response

    result = await get_analysis_data(mock_finance_client, "AAPL", AnalysisType.RECOMMENDATIONS)

    assert result["symbol"] == "AAPL"
    assert "recommendations" in result
    assert len(result["recommendations"]) == 2
    assert result["recommendations"][0].period == "0m"
    assert result["recommendations"][0].strong_buy == 15


async def test_get_analysis_data_upgrades_downgrades(mock_finance_client, sample_upgrades_downgrades_response, bypass_cache):
    """Test getting upgrades/downgrades analysis"""
    mock_finance_client.get_quote_summary.return_value = sample_upgrades_downgrades_response

    result = await get_analysis_data(mock_finance_client, "MSFT", AnalysisType.UPGRADES_DOWNGRADES)

    assert result["symbol"] == "MSFT"
    assert "upgrades_downgrades" in result
    assert len(result["upgrades_downgrades"]) == 2
    assert result["upgrades_downgrades"][0].firm == "Goldman Sachs"


async def test_get_analysis_data_price_targets(mock_finance_client, sample_price_targets_response, bypass_cache):
    """Test getting price targets analysis"""
    mock_finance_client.get_quote_summary.return_value = sample_price_targets_response

    result = await get_analysis_data(mock_finance_client, "GOOG", AnalysisType.PRICE_TARGETS)

    assert result["symbol"] == "GOOG"
    assert "price_targets" in result
    assert result["price_targets"].current == 150.25
    assert result["price_targets"].mean == 175.50


async def test_get_analysis_data_earnings_estimate(mock_finance_client, sample_earnings_estimate_response, bypass_cache):
    """Test getting earnings estimate analysis"""
    mock_finance_client.get_quote_summary.return_value = sample_earnings_estimate_response

    result = await get_analysis_data(mock_finance_client, "TSLA", AnalysisType.EARNINGS_ESTIMATE)

    assert result["symbol"] == "TSLA"
    assert "earnings_estimate" in result
    assert "0q" in result["earnings_estimate"].estimates
    assert result["earnings_estimate"].estimates["0q"]["avg"] == 1.55


async def test_get_analysis_data_revenue_estimate(mock_finance_client, bypass_cache):
    """Test getting revenue estimate analysis"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "earningsTrend": {
                        "trend": [
                            {
                                "period": "0q",
                                "revenueEstimate": {
                                    "avg": {"raw": 100000000000},
                                    "low": {"raw": 95000000000},
                                    "high": {"raw": 105000000000},
                                },
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_analysis_data(mock_finance_client, "NVDA", AnalysisType.REVENUE_ESTIMATE)

    assert result["symbol"] == "NVDA"
    assert "revenue_estimate" in result
    assert "0q" in result["revenue_estimate"].estimates


async def test_get_analysis_data_earnings_history(mock_finance_client, sample_earnings_history_response, bypass_cache):
    """Test getting earnings history analysis"""
    mock_finance_client.get_quote_summary.return_value = sample_earnings_history_response

    result = await get_analysis_data(mock_finance_client, "AAPL", AnalysisType.EARNINGS_HISTORY)

    assert result["symbol"] == "AAPL"
    assert "earnings_history" in result
    assert len(result["earnings_history"]) == 2
    assert result["earnings_history"][0].eps_actual == 1.46


async def test_get_analysis_data_empty_result(mock_finance_client, bypass_cache):
    """Test error handling when no data is returned"""
    mock_response = {"quoteSummary": {"result": []}}
    mock_finance_client.get_quote_summary.return_value = mock_response

    with pytest.raises(HTTPException) as exc_info:
        await get_analysis_data(mock_finance_client, "INVALID", AnalysisType.RECOMMENDATIONS)

    assert exc_info.value.status_code == 404
    assert "No recommendations data found" in exc_info.value.detail


async def test_get_analysis_data_api_error(mock_finance_client, bypass_cache):
    """Test error handling when API call fails"""
    mock_finance_client.get_quote_summary.side_effect = Exception("Yahoo Finance API error")

    with pytest.raises(HTTPException) as exc_info:
        await get_analysis_data(mock_finance_client, "ERROR", AnalysisType.RECOMMENDATIONS)

    assert exc_info.value.status_code == 500
    assert "Failed to fetch analysis data" in exc_info.value.detail


async def test_get_analysis_data_calls_api_with_correct_params(mock_finance_client, sample_recommendations_response, bypass_cache):
    """Test that API is called with correct parameters"""
    mock_finance_client.get_quote_summary.return_value = sample_recommendations_response

    await get_analysis_data(mock_finance_client, "AAPL", AnalysisType.RECOMMENDATIONS)

    # Verify API was called
    mock_finance_client.get_quote_summary.assert_called_once()
    call_args = mock_finance_client.get_quote_summary.call_args

    assert call_args.kwargs["symbol"] == "AAPL"
    assert "recommendationTrend" in call_args.kwargs["modules"]


async def test_get_analysis_data_all_types(mock_finance_client, bypass_cache):
    """Test all analysis type values work"""
    analysis_types = [
        AnalysisType.RECOMMENDATIONS,
        AnalysisType.UPGRADES_DOWNGRADES,
        AnalysisType.PRICE_TARGETS,
        AnalysisType.EARNINGS_ESTIMATE,
        AnalysisType.REVENUE_ESTIMATE,
        AnalysisType.EARNINGS_HISTORY,
    ]

    for analysis_type in analysis_types:
        # Create minimal mock response for each type
        if analysis_type == AnalysisType.RECOMMENDATIONS:
            mock_response = {"quoteSummary": {"result": [{"recommendationTrend": {"trend": []}}]}}
        elif analysis_type == AnalysisType.UPGRADES_DOWNGRADES:
            mock_response = {"quoteSummary": {"result": [{"upgradeDowngradeHistory": {"history": []}}]}}
        elif analysis_type == AnalysisType.PRICE_TARGETS:
            mock_response = {"quoteSummary": {"result": [{"financialData": {}}]}}
        elif analysis_type == AnalysisType.EARNINGS_ESTIMATE:
            mock_response = {"quoteSummary": {"result": [{"earningsTrend": {"trend": []}}]}}
        elif analysis_type == AnalysisType.REVENUE_ESTIMATE:
            mock_response = {"quoteSummary": {"result": [{"earningsTrend": {"trend": []}}]}}
        else:  # EARNINGS_HISTORY
            mock_response = {"quoteSummary": {"result": [{"earningsHistory": {"history": []}}]}}

        mock_finance_client.get_quote_summary.return_value = mock_response

        result = await get_analysis_data(mock_finance_client, "TEST", analysis_type)
        assert result["symbol"] == "TEST"
