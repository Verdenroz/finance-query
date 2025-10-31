from datetime import datetime
from typing import Any

import pandas as pd
import yfinance as yf
from fastapi import HTTPException

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

# Mapping of analysis types to their corresponding parser functions, ticker attributes, and field names
ANALYSIS_TYPE_MAPPING = {
    AnalysisType.RECOMMENDATIONS: {
        "parser": None,  # Will be set to _parse_recommendations after function definition
        "ticker_attr": "recommendations",
        "field_name": "recommendations",
    },
    AnalysisType.UPGRADES_DOWNGRADES: {
        "parser": None,  # Will be set to _parse_upgrades_downgrades after function definition
        "ticker_attr": "upgrades_downgrades",
        "field_name": "upgrades_downgrades",
    },
    AnalysisType.PRICE_TARGETS: {
        "parser": None,  # Will be set to _parse_price_targets after function definition
        "ticker_attr": "analyst_price_targets",
        "field_name": "price_targets",
    },
    AnalysisType.EARNINGS_ESTIMATE: {
        "parser": None,  # Will be set to _parse_earnings_estimate after function definition
        "ticker_attr": "earnings_estimate",
        "field_name": "earnings_estimate",
    },
    AnalysisType.REVENUE_ESTIMATE: {
        "parser": None,  # Will be set to _parse_revenue_estimate after function definition
        "ticker_attr": "revenue_estimate",
        "field_name": "revenue_estimate",
    },
    AnalysisType.EARNINGS_HISTORY: {
        "parser": None,  # Will be set to _parse_earnings_history after function definition
        "ticker_attr": "earnings_history",
        "field_name": "earnings_history",
    },
    AnalysisType.SUSTAINABILITY: {
        "parser": None,  # Will be set to _parse_sustainability after function definition
        "ticker_attr": "sustainability",
        "field_name": "sustainability",
    },
}


async def get_analysis_data(symbol: str, analysis_type: AnalysisType) -> AnalysisData:
    """
    Get analysis data for a symbol using yfinance.
    :param symbol: the stock symbol
    :param analysis_type: the type of analysis data to fetch
    :return: an AnalysisData object

    :raises HTTPException: with status code 404 if the symbol cannot be found, or 500 for any other error
    """
    ticker = yf.Ticker(symbol)

    try:
        # Validate analysis type
        if analysis_type not in ANALYSIS_TYPE_MAPPING:
            raise HTTPException(status_code=400, detail="Invalid analysis type")

        # Get mapping configuration
        mapping = ANALYSIS_TYPE_MAPPING[analysis_type]

        # Extract data from ticker using the mapped attribute
        ticker_data = getattr(ticker, mapping["ticker_attr"])

        # Parse the data using the mapped parser function
        parsed_data = mapping["parser"](ticker_data)

        # Create AnalysisData object with dynamic field assignment
        analysis_data_kwargs = {"symbol": symbol, "analysis_type": analysis_type, mapping["field_name"]: parsed_data}

        return AnalysisData(**analysis_data_kwargs)

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e)) from e


def _parse_recommendations(df: pd.DataFrame) -> list[RecommendationData]:
    """Parse recommendations DataFrame"""
    if df.empty:
        return []

    recommendations = []
    for _, row in df.iterrows():
        recommendation = RecommendationData(
            period=str(row.get("period", "")),
            strong_buy=row.get("strongBuy") if pd.notna(row.get("strongBuy")) else None,
            buy=row.get("buy") if pd.notna(row.get("buy")) else None,
            hold=row.get("hold") if pd.notna(row.get("hold")) else None,
            sell=row.get("sell") if pd.notna(row.get("sell")) else None,
            strong_sell=row.get("strongSell") if pd.notna(row.get("strongSell")) else None,
        )
        recommendations.append(recommendation)

    return recommendations


def _parse_upgrades_downgrades(df: pd.DataFrame) -> list[UpgradeDowngrade]:
    """Parse upgrades/downgrades DataFrame"""
    if df.empty:
        return []

    upgrades_downgrades = []
    for _, row in df.iterrows():
        upgrade_downgrade = UpgradeDowngrade(
            firm=str(row.get("firm", "")),
            to_grade=row.get("toGrade") if pd.notna(row.get("toGrade")) else None,
            from_grade=row.get("fromGrade") if pd.notna(row.get("fromGrade")) else None,
            action=row.get("action") if pd.notna(row.get("action")) else None,
            date=row.get("date") if pd.notna(row.get("date")) else None,
        )
        upgrades_downgrades.append(upgrade_downgrade)

    return upgrades_downgrades


def _parse_price_targets(data: Any) -> PriceTarget:
    """Parse analyst price targets"""
    if data is None or (hasattr(data, "empty") and data.empty):
        return PriceTarget()

    # Handle different data types that yfinance might return
    if isinstance(data, dict):
        return PriceTarget(
            current=data.get("current") if pd.notna(data.get("current")) else None,
            mean=data.get("mean") if pd.notna(data.get("mean")) else None,
            median=data.get("median") if pd.notna(data.get("median")) else None,
            low=data.get("low") if pd.notna(data.get("low")) else None,
            high=data.get("high") if pd.notna(data.get("high")) else None,
        )
    elif isinstance(data, pd.Series):
        return PriceTarget(
            current=data.get("current") if pd.notna(data.get("current")) else None,
            mean=data.get("mean") if pd.notna(data.get("mean")) else None,
            median=data.get("median") if pd.notna(data.get("median")) else None,
            low=data.get("low") if pd.notna(data.get("low")) else None,
            high=data.get("high") if pd.notna(data.get("high")) else None,
        )
    else:
        return PriceTarget()


def _parse_earnings_estimate(df: pd.DataFrame) -> EarningsEstimate:
    """Parse earnings estimate DataFrame"""
    if df.empty:
        return EarningsEstimate(estimates={})

    # Convert DataFrame to dict format
    estimates_dict = {}
    for column in df.columns:
        estimates_dict[column] = df[column].to_dict()

    return EarningsEstimate(estimates=estimates_dict)


def _parse_revenue_estimate(df: pd.DataFrame) -> RevenueEstimate:
    """Parse revenue estimate DataFrame"""
    if df.empty:
        return RevenueEstimate(estimates={})

    # Convert DataFrame to dict format
    estimates_dict = {}
    for column in df.columns:
        estimates_dict[column] = df[column].to_dict()

    return RevenueEstimate(estimates=estimates_dict)


def _parse_earnings_history(df: pd.DataFrame) -> list[EarningsHistoryItem]:
    """Parse earnings history DataFrame"""
    if df.empty:
        return []

    earnings_history = []
    for _, row in df.iterrows():
        earnings_item = EarningsHistoryItem(
            date=row.get("date", datetime.now()),
            eps_actual=row.get("eps_actual") if pd.notna(row.get("eps_actual")) else None,
            eps_estimate=row.get("eps_estimate") if pd.notna(row.get("eps_estimate")) else None,
            surprise=row.get("surprise") if pd.notna(row.get("surprise")) else None,
            surprise_percent=row.get("surprise_percent") if pd.notna(row.get("surprise_percent")) else None,
        )
        earnings_history.append(earnings_item)

    return earnings_history


def _parse_sustainability(df: pd.DataFrame) -> SustainabilityScores:
    """Parse sustainability DataFrame"""
    if df.empty:
        return SustainabilityScores(scores={})

    # Convert DataFrame to dict format
    scores_dict = {}
    for column in df.columns:
        value = df[column].iloc[0] if len(df) > 0 else None
        scores_dict[column] = value if pd.notna(value) else None

    return SustainabilityScores(scores=scores_dict)


# Update the mapping with actual parser function references
ANALYSIS_TYPE_MAPPING[AnalysisType.RECOMMENDATIONS]["parser"] = _parse_recommendations
ANALYSIS_TYPE_MAPPING[AnalysisType.UPGRADES_DOWNGRADES]["parser"] = _parse_upgrades_downgrades
ANALYSIS_TYPE_MAPPING[AnalysisType.PRICE_TARGETS]["parser"] = _parse_price_targets
ANALYSIS_TYPE_MAPPING[AnalysisType.EARNINGS_ESTIMATE]["parser"] = _parse_earnings_estimate
ANALYSIS_TYPE_MAPPING[AnalysisType.REVENUE_ESTIMATE]["parser"] = _parse_revenue_estimate
ANALYSIS_TYPE_MAPPING[AnalysisType.EARNINGS_HISTORY]["parser"] = _parse_earnings_history
ANALYSIS_TYPE_MAPPING[AnalysisType.SUSTAINABILITY]["parser"] = _parse_sustainability
