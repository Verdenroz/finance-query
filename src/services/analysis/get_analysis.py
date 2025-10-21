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
        if analysis_type == AnalysisType.RECOMMENDATIONS:
            data = _parse_recommendations(ticker.recommendations)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, recommendations=data)

        elif analysis_type == AnalysisType.UPGRADES_DOWNGRADES:
            data = _parse_upgrades_downgrades(ticker.upgrades_downgrades)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, upgrades_downgrades=data)

        elif analysis_type == AnalysisType.PRICE_TARGETS:
            data = _parse_price_targets(ticker.analyst_price_targets)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, price_targets=data)

        elif analysis_type == AnalysisType.EARNINGS_ESTIMATE:
            data = _parse_earnings_estimate(ticker.earnings_estimate)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, earnings_estimate=data)

        elif analysis_type == AnalysisType.REVENUE_ESTIMATE:
            data = _parse_revenue_estimate(ticker.revenue_estimate)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, revenue_estimate=data)

        elif analysis_type == AnalysisType.EARNINGS_HISTORY:
            data = _parse_earnings_history(ticker.earnings_history)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, earnings_history=data)

        elif analysis_type == AnalysisType.SUSTAINABILITY:
            data = _parse_sustainability(ticker.sustainability)
            return AnalysisData(symbol=symbol, analysis_type=analysis_type, sustainability=data)

        else:
            raise HTTPException(status_code=400, detail="Invalid analysis type")

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
