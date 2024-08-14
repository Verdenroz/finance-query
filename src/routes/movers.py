from fastapi import APIRouter, Security, Response
from fastapi.security import APIKeyHeader
from typing_extensions import List

from src.schemas import MarketMover
from src.services import scrape_actives, scrape_gainers, scrape_losers

router = APIRouter()


@router.get("/actives",
            summary="Returns most active stocks",
            description="Get the stocks or funds with the highest trading volume during the current trading session",
            response_model=List[MarketMover],
            tags=["Market Movers"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_actives(response: Response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await scrape_actives()


@router.get("/gainers",
            summary="Returns stocks with the highest price increase",
            description="The top gaining stocks or funds during the current trading session.",
            response_model=List[MarketMover],
            tags=["Market Movers"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_gainers(response: Response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await scrape_gainers()


@router.get("/losers",
            summary="Returns stocks with the highest price decrease",
            description="The top losing stocks or funds during the current trading session.",
            response_model=List[MarketMover],
            tags=["Market Movers"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_losers(response: Response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await scrape_losers()
