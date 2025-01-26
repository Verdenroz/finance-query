from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from src.schemas import MarketMover
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
        500: {"description": "Failed to parse market movers"}
    }
)
async def get_actives():
    return await scrape_actives()


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
        500: {"description": "Failed to parse market movers"}
    }
)
async def get_gainers():
    return await scrape_gainers()


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
        500: {"description": "Failed to parse market movers"}
    }
)
async def get_losers():
    return await scrape_losers()
