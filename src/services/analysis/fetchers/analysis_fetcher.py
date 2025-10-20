import pandas as pd
from typing import Optional
from src.yfinance_client.ticker import Ticker
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


async def fetch_analyst_price_targets(symbol: str) -> AnalystPriceTargets:
    """Fetch analyst price targets for a symbol"""
    ticker = Ticker(symbol)
    data = ticker.get_analyst_price_targets()
    
    return AnalystPriceTargets(
        symbol=symbol,
        current=data.get('current'),
        low=data.get('low'),
        high=data.get('high'),
        mean=data.get('mean'),
        median=data.get('median'),
    )


async def fetch_earnings_estimate(symbol: str) -> EarningsEstimate:
    """Fetch earnings estimates for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_earnings_estimate()
    
    estimates = []
    if not df.empty:
        for period, row in df.iterrows():
            estimates.append(EstimateData(
                period=str(period),
                number_of_analysts=row.get('numberOfAnalysts'),
                avg=row.get('avg'),
                low=row.get('low'),
                high=row.get('high'),
                year_ago_eps=row.get('yearAgoEps'),
                growth=row.get('growth'),
            ))
    
    return EarningsEstimate(symbol=symbol, estimates=estimates)


async def fetch_revenue_estimate(symbol: str) -> RevenueEstimate:
    """Fetch revenue estimates for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_revenue_estimate()
    
    estimates = []
    if not df.empty:
        for period, row in df.iterrows():
            estimates.append(EstimateData(
                period=str(period),
                number_of_analysts=row.get('numberOfAnalysts'),
                avg=row.get('avg'),
                low=row.get('low'),
                high=row.get('high'),
                year_ago_eps=row.get('yearAgoRevenue'),  # Note: using year_ago_eps field for revenue
                growth=row.get('growth'),
            ))
    
    return RevenueEstimate(symbol=symbol, estimates=estimates)


async def fetch_earnings_history(symbol: str) -> EarningsHistory:
    """Fetch earnings history for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_earnings_history()
    
    history = []
    if not df.empty:
        for quarter, row in df.iterrows():
            # Convert pandas datetime to ISO string
            quarter_str = quarter.strftime('%Y-%m-%d') if hasattr(quarter, 'strftime') else str(quarter)
            history.append(EarningsHistoryItem(
                quarter=quarter_str,
                eps_estimate=row.get('epsEstimate'),
                eps_actual=row.get('epsActual'),
                eps_difference=row.get('epsDifference'),
                surprise_percent=row.get('surprisePercent'),
            ))
    
    return EarningsHistory(symbol=symbol, history=history)


async def fetch_eps_trend(symbol: str) -> EPSTrend:
    """Fetch EPS trend for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_eps_trend()
    
    trends = []
    if not df.empty:
        for period, row in df.iterrows():
            trends.append(EPSTrendItem(
                period=str(period),
                current=row.get('current'),
                seven_days_ago=row.get('7daysAgo'),
                thirty_days_ago=row.get('30daysAgo'),
                sixty_days_ago=row.get('60daysAgo'),
                ninety_days_ago=row.get('90daysAgo'),
            ))
    
    return EPSTrend(symbol=symbol, trends=trends)


async def fetch_eps_revisions(symbol: str) -> EPSRevisions:
    """Fetch EPS revisions for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_eps_revisions()
    
    revisions = []
    if not df.empty:
        for period, row in df.iterrows():
            revisions.append(EPSRevisionItem(
                period=str(period),
                up_last_7days=row.get('upLast7days'),
                up_last_30days=row.get('upLast30days'),
                down_last_7days=row.get('downLast7days'),
                down_last_30days=row.get('downLast30days'),
            ))
    
    return EPSRevisions(symbol=symbol, revisions=revisions)


async def fetch_growth_estimates(symbol: str) -> GrowthEstimates:
    """Fetch growth estimates for a symbol"""
    ticker = Ticker(symbol)
    df = ticker.get_growth_estimates()
    
    estimates = []
    if not df.empty:
        for period, row in df.iterrows():
            estimates.append(GrowthEstimateItem(
                period=str(period),
                stock=row.get('stockTrend'),
                industry=row.get('industryTrend'),
                sector=row.get('sectorTrend'),
                index=row.get('indexTrend'),
            ))
    
    return GrowthEstimates(symbol=symbol, estimates=estimates)
