from datetime import datetime
from typing import Any

from fastapi import HTTPException

from src.models.analysis import (
    AnalysisType,
    EarningsEstimate,
    EarningsHistoryItem,
    PriceTarget,
    RecommendationData,
    RevenueEstimate,
    UpgradeDowngrade,
)
from src.utils.cache import cache
from src.utils.dependencies import FinanceClient

# Mapping of analysis types to their corresponding Yahoo Finance modules
ANALYSIS_TYPE_MODULES = {
    AnalysisType.RECOMMENDATIONS: ["recommendationTrend"],
    AnalysisType.UPGRADES_DOWNGRADES: ["upgradeDowngradeHistory"],
    AnalysisType.PRICE_TARGETS: ["financialData"],
    AnalysisType.EARNINGS_ESTIMATE: ["earningsTrend"],
    AnalysisType.REVENUE_ESTIMATE: ["earningsTrend"],
    AnalysisType.EARNINGS_HISTORY: ["earningsHistory"],
}

# Mapping of analysis types to their data extraction and parsing configuration
ANALYSIS_TYPE_CONFIG = {
    AnalysisType.RECOMMENDATIONS: {
        "data_path": lambda d: d.get("recommendationTrend", {}).get("trend", []),
        "parser": "_parse_recommendations",
        "field_name": "recommendations",
    },
    AnalysisType.UPGRADES_DOWNGRADES: {
        "data_path": lambda d: d.get("upgradeDowngradeHistory", {}).get("history", []),
        "parser": "_parse_upgrades_downgrades",
        "field_name": "upgrades_downgrades",
    },
    AnalysisType.PRICE_TARGETS: {
        "data_path": lambda d: d.get("financialData", {}),
        "parser": "_parse_price_targets",
        "field_name": "price_targets",
    },
    AnalysisType.EARNINGS_ESTIMATE: {
        "data_path": lambda d: d.get("earningsTrend", {}).get("trend", []),
        "parser": "_parse_earnings_estimate",
        "field_name": "earnings_estimate",
    },
    AnalysisType.REVENUE_ESTIMATE: {
        "data_path": lambda d: d.get("earningsTrend", {}).get("trend", []),
        "parser": "_parse_revenue_estimate",
        "field_name": "revenue_estimate",
    },
    AnalysisType.EARNINGS_HISTORY: {
        "data_path": lambda d: d.get("earningsHistory", {}).get("history", []),
        "parser": "_parse_earnings_history",
        "field_name": "earnings_history",
    },
}


@cache(expire=3600)
async def get_analysis_data(finance_client: FinanceClient, symbol: str, analysis_type: AnalysisType) -> dict[str, Any]:
    """
    Get analysis data for a symbol using Yahoo Finance API directly.

    Args:
        finance_client: Yahoo Finance client
        symbol: Stock symbol
        analysis_type: Type of analysis data to fetch

    Returns:
        Dictionary with symbol and parsed analysis data

    Raises:
        HTTPException: 400 for invalid type, 404 if no data found, 500 for other errors
    """
    if analysis_type not in ANALYSIS_TYPE_MODULES:
        raise HTTPException(status_code=400, detail="Invalid analysis type")

    try:
        # Get required modules
        modules = ANALYSIS_TYPE_MODULES[analysis_type]

        # Fetch data from Yahoo Finance
        response = await finance_client.get_quote_summary(symbol=symbol.upper(), modules=modules)

        # Extract result
        result = response.get("quoteSummary", {}).get("result", [])
        if not result:
            raise HTTPException(status_code=404, detail=f"No {analysis_type.value} data found for {symbol}")

        data = result[0]

        # Get configuration for this analysis type
        config = ANALYSIS_TYPE_CONFIG.get(analysis_type)
        if not config:
            raise HTTPException(status_code=400, detail=f"Invalid analysis type: {analysis_type}")

        # Extract data using configured path
        raw_data = config["data_path"](data)

        # Get the parser function by name
        parser_name = config["parser"]
        parser_func = globals()[parser_name]

        # Parse the data
        parsed_data = parser_func(raw_data)

        # Return dictionary with symbol and parsed data
        return {"symbol": symbol.upper(), config["field_name"]: parsed_data}

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch analysis data: {str(e)}") from e


def _parse_recommendations(trend_list: list[dict[str, Any]]) -> list[RecommendationData]:
    """Parse recommendations list from Yahoo Finance API"""
    if not trend_list:
        return []

    recommendations = []
    for trend_data in trend_list:
        recommendation = RecommendationData(
            period=trend_data.get("period", ""),
            strong_buy=trend_data.get("strongBuy"),
            buy=trend_data.get("buy"),
            hold=trend_data.get("hold"),
            sell=trend_data.get("sell"),
            strong_sell=trend_data.get("strongSell"),
        )
        recommendations.append(recommendation)

    return recommendations


def _parse_upgrades_downgrades(history_list: list[dict[str, Any]]) -> list[UpgradeDowngrade]:
    """Parse upgrades/downgrades list from Yahoo Finance API"""
    if not history_list:
        return []

    upgrades_downgrades = []
    for item in history_list:
        # Convert epoch to datetime
        epoch_time = item.get("epochGradeDate")
        grade_date = datetime.fromtimestamp(epoch_time) if epoch_time else None

        upgrade_downgrade = UpgradeDowngrade(
            firm=item.get("firm", ""),
            to_grade=item.get("toGrade"),
            from_grade=item.get("fromGrade"),
            action=item.get("action"),
            date=grade_date,
        )
        upgrades_downgrades.append(upgrade_downgrade)

    return upgrades_downgrades


def _safe_extract_value(value: Any) -> float | None:
    """
    Safely extract numeric value from Yahoo Finance response.
    Handles both dict format {"raw": value} and direct numeric values.

    Args:
        value: Value from Yahoo API (dict, float, int, or None)

    Returns:
        Extracted numeric value or None
    """
    if value is None:
        return None

    # If it's a dict with "raw" key, extract it
    if isinstance(value, dict):
        return value.get("raw")

    # If it's already a numeric type, return it
    if isinstance(value, int | float):
        return float(value)

    return None


def _parse_price_targets(data: dict[str, Any]) -> PriceTarget:
    """Parse analyst price targets from Yahoo Finance API"""
    if not data:
        return PriceTarget()

    return PriceTarget(
        current=_safe_extract_value(data.get("currentPrice")),
        mean=_safe_extract_value(data.get("targetMeanPrice")),
        median=_safe_extract_value(data.get("targetMedianPrice")),
        low=_safe_extract_value(data.get("targetLowPrice")),
        high=_safe_extract_value(data.get("targetHighPrice")),
    )


def _parse_earnings_estimate(trend_list: list[dict[str, Any]]) -> EarningsEstimate:
    """Parse earnings estimate from Yahoo Finance API"""
    if not trend_list:
        return EarningsEstimate(estimates={})

    # Build estimates dict from trend data
    estimates_dict = {}
    for trend_data in trend_list:
        period = trend_data.get("period", "")
        earnings_estimate = trend_data.get("earningsEstimate", {})

        estimates_dict[period] = {
            "avg": earnings_estimate.get("avg", {}).get("raw"),
            "low": earnings_estimate.get("low", {}).get("raw"),
            "high": earnings_estimate.get("high", {}).get("raw"),
            "numberOfAnalysts": earnings_estimate.get("numberOfAnalysts", {}).get("raw"),
            "yearAgoEps": earnings_estimate.get("yearAgoEps", {}).get("raw"),
            "growth": earnings_estimate.get("growth", {}).get("raw"),
        }

    return EarningsEstimate(estimates=estimates_dict)


def _parse_revenue_estimate(trend_list: list[dict[str, Any]]) -> RevenueEstimate:
    """Parse revenue estimate from Yahoo Finance API"""
    if not trend_list:
        return RevenueEstimate(estimates={})

    # Build estimates dict from trend data
    estimates_dict = {}
    for trend_data in trend_list:
        period = trend_data.get("period", "")
        revenue_estimate = trend_data.get("revenueEstimate", {})

        estimates_dict[period] = {
            "avg": revenue_estimate.get("avg", {}).get("raw"),
            "low": revenue_estimate.get("low", {}).get("raw"),
            "high": revenue_estimate.get("high", {}).get("raw"),
            "numberOfAnalysts": revenue_estimate.get("numberOfAnalysts", {}).get("raw"),
            "yearAgoRevenue": revenue_estimate.get("yearAgoRevenue", {}).get("raw"),
            "growth": revenue_estimate.get("growth", {}).get("raw"),
        }

    return RevenueEstimate(estimates=estimates_dict)


def _parse_earnings_history(history_list: list[dict[str, Any]]) -> list[EarningsHistoryItem]:
    """Parse earnings history from Yahoo Finance API"""
    if not history_list:
        return []

    earnings_history = []
    for item in history_list:
        # Convert quarter date
        quarter = item.get("quarter", {}).get("raw")
        if quarter:
            quarter_date = datetime.fromtimestamp(quarter)
        else:
            quarter_date = datetime.now()

        earnings_item = EarningsHistoryItem(
            date=quarter_date,
            eps_actual=item.get("epsActual", {}).get("raw"),
            eps_estimate=item.get("epsEstimate", {}).get("raw"),
            surprise=item.get("epsDifference", {}).get("raw"),
            surprise_percent=item.get("surprisePercent", {}).get("raw"),
        )
        earnings_history.append(earnings_item)

    return earnings_history
