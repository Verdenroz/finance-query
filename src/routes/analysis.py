from fastapi import APIRouter, Path, Security
from fastapi.security import APIKeyHeader

from src.models import (
    AnalystPriceTargets,
    EarningsEstimate,
    RevenueEstimate,
    EarningsHistory,
    EPSTrend,
    EPSRevisions,
    GrowthEstimates,
    ValidationErrorResponse,
)
from src.services import (
    get_analyst_price_targets,
    get_earnings_estimate,
    get_revenue_estimate,
    get_earnings_history,
    get_eps_trend,
    get_eps_revisions,
    get_growth_estimates,
)

router = APIRouter()


@router.get(
    path="/analysis/{symbol}/price-targets",
    summary="Get analyst price targets for a stock",
    description="Returns analyst price targets including current, low, high, mean, and median targets.",
    response_model=AnalystPriceTargets,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": AnalystPriceTargets,
            "description": "Successfully retrieved analyst price targets",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "current": 150.00,
                        "low": 120.00,
                        "high": 180.00,
                        "mean": 150.00,
                        "median": 148.00,
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def analyst_price_targets(symbol: str = Path(..., description="Stock symbol")):
    return await get_analyst_price_targets(symbol.upper())


@router.get(
    path="/analysis/{symbol}/earnings-estimate",
    summary="Get earnings estimates",
    description="Returns quarterly and yearly earnings estimates from analysts.",
    response_model=EarningsEstimate,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": EarningsEstimate,
            "description": "Successfully retrieved earnings estimates",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "estimates": [
                            {
                                "period": "0q",
                                "number_of_analysts": 25,
                                "avg": 2.50,
                                "low": 2.20,
                                "high": 2.80,
                                "year_ago_eps": 2.10,
                                "growth": 0.19,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def earnings_estimate(symbol: str = Path(..., description="Stock symbol")):
    return await get_earnings_estimate(symbol.upper())


@router.get(
    path="/analysis/{symbol}/revenue-estimate",
    summary="Get revenue estimates",
    description="Returns quarterly and yearly revenue estimates from analysts.",
    response_model=RevenueEstimate,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": RevenueEstimate,
            "description": "Successfully retrieved revenue estimates",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "estimates": [
                            {
                                "period": "0q",
                                "number_of_analysts": 25,
                                "avg": 95.0,
                                "low": 90.0,
                                "high": 100.0,
                                "year_ago_eps": 89.0,
                                "growth": 0.067,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def revenue_estimate(symbol: str = Path(..., description="Stock symbol")):
    return await get_revenue_estimate(symbol.upper())


@router.get(
    path="/analysis/{symbol}/earnings-history",
    summary="Get earnings history",
    description="Returns historical earnings data including estimates, actuals, and surprises.",
    response_model=EarningsHistory,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": EarningsHistory,
            "description": "Successfully retrieved earnings history",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "history": [
                            {
                                "quarter": "2024-03-31",
                                "eps_estimate": 2.50,
                                "eps_actual": 2.65,
                                "eps_difference": 0.15,
                                "surprise_percent": 6.0,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def earnings_history(symbol: str = Path(..., description="Stock symbol")):
    return await get_earnings_history(symbol.upper())


@router.get(
    path="/analysis/{symbol}/eps-trend",
    summary="Get EPS trend",
    description="Returns EPS trend over time showing how estimates have changed.",
    response_model=EPSTrend,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": EPSTrend,
            "description": "Successfully retrieved EPS trend",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "trends": [
                            {
                                "period": "0q",
                                "current": 2.50,
                                "seven_days_ago": 2.48,
                                "thirty_days_ago": 2.45,
                                "sixty_days_ago": 2.42,
                                "ninety_days_ago": 2.40,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def eps_trend(symbol: str = Path(..., description="Stock symbol")):
    return await get_eps_trend(symbol.upper())


@router.get(
    path="/analysis/{symbol}/eps-revisions",
    summary="Get EPS revisions",
    description="Returns EPS revisions showing analyst upgrades and downgrades.",
    response_model=EPSRevisions,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": EPSRevisions,
            "description": "Successfully retrieved EPS revisions",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "revisions": [
                            {
                                "period": "0q",
                                "up_last_7days": 5,
                                "up_last_30days": 12,
                                "down_last_7days": 2,
                                "down_last_30days": 3,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def eps_revisions(symbol: str = Path(..., description="Stock symbol")):
    return await get_eps_revisions(symbol.upper())


@router.get(
    path="/analysis/{symbol}/growth-estimates",
    summary="Get growth estimates",
    description="Returns growth estimates comparing stock performance to industry, sector, and index.",
    response_model=GrowthEstimates,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": GrowthEstimates,
            "description": "Successfully retrieved growth estimates",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "estimates": [
                            {
                                "period": "0q",
                                "stock": 0.15,
                                "industry": 0.12,
                                "sector": 0.13,
                                "index": 0.10,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def growth_estimates(symbol: str = Path(..., description="Stock symbol")):
    return await get_growth_estimates(symbol.upper())
