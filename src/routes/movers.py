from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import MarketMover, ValidationErrorResponse
from src.models.marketmover import MoverCount
from src.services import get_actives, get_gainers, get_losers

router = APIRouter()


@router.get(
    path="/actives",
    summary="Get list of most active stocks",
    description="Returns summary data for the most active stocks during the current trading session, including the "
    "symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successful retrieved most active stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def actives(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await get_actives(count)


@router.get(
    path="/gainers",
    summary="Get list of stocks with the highest price increase",
    description="Returns the top gaining stocks or funds during the current trading session, including the symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successfully retrieved top gaining stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def gainers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await get_gainers(count)


@router.get(
    path="/losers",
    summary="Get list of stocks with the highest price decrease",
    description="Returns the top losing stocks or funds during the current trading session, including the symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successfully retrieved top losing stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def losers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await get_losers(count)
