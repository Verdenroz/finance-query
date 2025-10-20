from src.models.analysis import (
    AnalystPriceTargets,
    EarningsEstimate,
    RevenueEstimate,
    EarningsHistory,
    EPSTrend,
    EPSRevisions,
    GrowthEstimates,
)
from src.services.analysis.fetchers.analysis_fetcher import (
    fetch_analyst_price_targets,
    fetch_earnings_estimate,
    fetch_revenue_estimate,
    fetch_earnings_history,
    fetch_eps_trend,
    fetch_eps_revisions,
    fetch_growth_estimates,
)
from src.utils.cache import cache
from src.utils.retry import retry


@cache(expire=3600, market_closed_expire=7200)
async def get_analyst_price_targets(symbol: str) -> AnalystPriceTargets:
    """Get analyst price targets for a symbol with caching"""
    return await fetch_analyst_price_targets(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_earnings_estimate(symbol: str) -> EarningsEstimate:
    """Get earnings estimates for a symbol with caching"""
    return await fetch_earnings_estimate(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_revenue_estimate(symbol: str) -> RevenueEstimate:
    """Get revenue estimates for a symbol with caching"""
    return await fetch_revenue_estimate(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_earnings_history(symbol: str) -> EarningsHistory:
    """Get earnings history for a symbol with caching"""
    return await fetch_earnings_history(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_eps_trend(symbol: str) -> EPSTrend:
    """Get EPS trend for a symbol with caching"""
    return await fetch_eps_trend(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_eps_revisions(symbol: str) -> EPSRevisions:
    """Get EPS revisions for a symbol with caching"""
    return await fetch_eps_revisions(symbol)


@cache(expire=3600, market_closed_expire=7200)
async def get_growth_estimates(symbol: str) -> GrowthEstimates:
    """Get growth estimates for a symbol with caching"""
    return await fetch_growth_estimates(symbol)
