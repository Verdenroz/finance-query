from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader

from src.schemas import MarketMover, ValidationErrorResponse
from src.schemas.marketmover import MoverCount
from src.services import scrape_actives, scrape_gainers, scrape_losers

router = APIRouter()


@router.get(
    path="/actives",
    summary="Get list of most active stocks",
    description="Returns summary data for the most active stocks during the current trading session, including the "
                "symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    tags=["Market Movers"],
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
                        "errors": {
                            "count": [
                                "Input should be '25', '50' or '100'"
                            ]
                        }
                    }
                }
            }
        }
    }
)
async def get_actives(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await scrape_actives(count)


@router.get(
    path="/gainers",
    summary="Get list of stocks with the highest price increase",
    description="Returns the top gaining stocks or funds during the current trading session, including the "
                "symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    tags=["Market Movers"],
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
                        "errors": {
                            "count": [
                                "Input should be '25', '50' or '100'"
                            ]
                        }
                    }
                }
            }
        }
    }
)
async def get_gainers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await scrape_gainers(count)


@router.get(
    path="/losers",
    summary="Get list of stocks with the highest price decrease",
    description="Returns the top losing stocks or funds during the current trading session, including the "
                "symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    tags=["Market Movers"],
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
                        "errors": {
                            "count": [
                                "Input should be '25', '50' or '100'"
                            ]
                        }
                    }
                }
            }
        }
    }
)
async def get_losers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    return await scrape_losers(count)
