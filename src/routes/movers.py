from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from src.schemas import MarketMover
from src.services import scrape_actives, scrape_gainers, scrape_losers
from src.utils import cache

router = APIRouter()


@router.get("/v1/actives",
            summary="Returns most active stocks",
            description="Get the stocks or funds with the highest trading volume during the current trading session "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=60, check_market=True)
async def get_actives():
    return await scrape_actives()


@router.get("/v1/gainers",
            summary="Returns stocks with the highest price increase",
            description="The top gaining stocks or funds during the current trading session. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=60, check_market=True)
async def get_gainers():
    return await scrape_gainers()


@router.get("/v1/losers",
            summary="Returns stocks with the highest price decrease",
            description="The top losing stocks or funds during the current trading session. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=60, check_market=True)
async def get_losers():
    return await scrape_losers()
