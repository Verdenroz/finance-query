from fastapi import APIRouter, Path, Security
from fastapi.security import APIKeyHeader

from src.models import ValidationErrorResponse
from src.models.analysis import (
    AnalysisType,
    EarningsEstimateResponse,
    EarningsHistoryResponse,
    PriceTargetsResponse,
    RecommendationsResponse,
    RevenueEstimateResponse,
    UpgradesDowngradesResponse,
)
from src.services.analysis.get_analysis import get_analysis_data
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/analysis/{symbol}/recommendations",
    summary="Get analyst recommendations",
    description="Returns analyst recommendation trends (strong buy, buy, hold, sell, strong sell) over time.",
    response_model=RecommendationsResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved analyst recommendations",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "recommendations": [
                            {
                                "period": "0m",
                                "strong_buy": 15,
                                "buy": 25,
                                "hold": 8,
                                "sell": 1,
                                "strong_sell": 0,
                            },
                            {
                                "period": "-1m",
                                "strong_buy": 14,
                                "buy": 26,
                                "hold": 8,
                                "sell": 1,
                                "strong_sell": 0,
                            },
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No recommendations data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_recommendations(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get analyst recommendations for a stock symbol.

    Returns recommendation trends showing how many analysts rate the stock at each level.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.RECOMMENDATIONS)
    return RecommendationsResponse(symbol=data["symbol"], recommendations=data["recommendations"])


@router.get(
    path="/analysis/{symbol}/upgrades-downgrades",
    summary="Get analyst upgrades and downgrades",
    description="Returns history of analyst rating changes (upgrades/downgrades) with firm names and dates.",
    response_model=UpgradesDowngradesResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved upgrades and downgrades",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "TSLA",
                        "upgrades_downgrades": [
                            {
                                "firm": "Morgan Stanley",
                                "to_grade": "Overweight",
                                "from_grade": "Equal-Weight",
                                "action": "up",
                                "date": "2024-11-01T00:00:00",
                            },
                            {
                                "firm": "Goldman Sachs",
                                "to_grade": "Buy",
                                "from_grade": "Neutral",
                                "action": "up",
                                "date": "2024-10-15T00:00:00",
                            },
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No upgrades_downgrades data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_upgrades_downgrades(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get analyst upgrades and downgrades for a stock symbol.

    Returns historical rating changes from analyst firms.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.UPGRADES_DOWNGRADES)
    return UpgradesDowngradesResponse(symbol=data["symbol"], upgrades_downgrades=data["upgrades_downgrades"])


@router.get(
    path="/analysis/{symbol}/price-targets",
    summary="Get analyst price targets",
    description="Returns consensus analyst price targets including mean, median, low, and high targets.",
    response_model=PriceTargetsResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved price targets",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "price_targets": {
                            "current": 145.23,
                            "mean": 160.50,
                            "median": 158.00,
                            "low": 120.00,
                            "high": 200.00,
                        },
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No price_targets data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_price_targets(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get analyst price targets for a stock symbol.

    Returns consensus price target data from analyst coverage.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.PRICE_TARGETS)
    return PriceTargetsResponse(symbol=data["symbol"], price_targets=data["price_targets"])


@router.get(
    path="/analysis/{symbol}/earnings-estimate",
    summary="Get earnings estimates",
    description="Returns analyst earnings estimates for future quarters including average, low, high, and growth projections.",
    response_model=EarningsEstimateResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved earnings estimates",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "MSFT",
                        "earnings_estimate": {
                            "estimates": {
                                "0q": {
                                    "avg": 2.85,
                                    "low": 2.75,
                                    "high": 2.95,
                                    "numberOfAnalysts": 42,
                                    "yearAgoEps": 2.45,
                                    "growth": 0.163,
                                },
                                "+1q": {
                                    "avg": 3.10,
                                    "low": 2.95,
                                    "high": 3.25,
                                    "numberOfAnalysts": 40,
                                    "yearAgoEps": 2.65,
                                    "growth": 0.170,
                                },
                            }
                        },
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No earnings_estimate data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_earnings_estimate(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get earnings estimates for a stock symbol.

    Returns analyst consensus earnings forecasts for upcoming quarters.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.EARNINGS_ESTIMATE)
    return EarningsEstimateResponse(symbol=data["symbol"], earnings_estimate=data["earnings_estimate"])


@router.get(
    path="/analysis/{symbol}/revenue-estimate",
    summary="Get revenue estimates",
    description="Returns analyst revenue estimates for future quarters including average, low, high, and growth projections.",
    response_model=RevenueEstimateResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved revenue estimates",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "GOOGL",
                        "revenue_estimate": {
                            "estimates": {
                                "0q": {
                                    "avg": 86500000000,
                                    "low": 84000000000,
                                    "high": 89000000000,
                                    "numberOfAnalysts": 38,
                                    "yearAgoRevenue": 76700000000,
                                    "growth": 0.128,
                                },
                                "+1q": {
                                    "avg": 92000000000,
                                    "low": 89000000000,
                                    "high": 95000000000,
                                    "numberOfAnalysts": 36,
                                    "yearAgoRevenue": 80500000000,
                                    "growth": 0.143,
                                },
                            }
                        },
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No revenue_estimate data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_revenue_estimate(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get revenue estimates for a stock symbol.

    Returns analyst consensus revenue forecasts for upcoming quarters.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.REVENUE_ESTIMATE)
    return RevenueEstimateResponse(symbol=data["symbol"], revenue_estimate=data["revenue_estimate"])


@router.get(
    path="/analysis/{symbol}/earnings-history",
    summary="Get historical earnings",
    description="Returns historical earnings data including actual vs estimated EPS and surprise percentages.",
    response_model=EarningsHistoryResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved earnings history",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "earnings_history": [
                            {
                                "date": "2024-09-30T00:00:00",
                                "eps_actual": 1.46,
                                "eps_estimate": 1.43,
                                "surprise": 0.03,
                                "surprise_percent": 0.021,
                            },
                            {
                                "date": "2024-06-30T00:00:00",
                                "eps_actual": 1.40,
                                "eps_estimate": 1.35,
                                "surprise": 0.05,
                                "surprise_percent": 0.037,
                            },
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No earnings_history data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_earnings_history(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get historical earnings for a stock symbol.

    Returns past earnings results with actual vs estimated performance.
    """
    data = await get_analysis_data(finance_client, symbol.upper(), AnalysisType.EARNINGS_HISTORY)
    return EarningsHistoryResponse(symbol=data["symbol"], earnings_history=data["earnings_history"])
