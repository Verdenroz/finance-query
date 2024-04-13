from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from services.scrape_losers import scrape_losers
import models

router = APIRouter()


@router.get("/v1/losers",
            summary="Returns stocks with the highest price decrease",
            description="The top losing stocks or funds during the current trading session. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[models.MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_gainers():
    return await scrape_losers()
