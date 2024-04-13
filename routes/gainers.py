from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from services.scrape_gainers import scrape_gainers
import models

router = APIRouter()


@router.get("/v1/gainers",
            summary="Returns stocks with the highest price increase",
            description="The top gaining stocks or funds during the current trading session. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[models.MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_gainers():
    return await scrape_gainers()
