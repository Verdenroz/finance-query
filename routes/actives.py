from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from services.scrape_actives import scrape_actives
import models

router = APIRouter()


@router.get("/v1/actives",
            summary="Returns most active stocks",
            description="Get the stocks or funds with the highest trading volume during the current trading session "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[models.MarketMover],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_actives():
    return await scrape_actives()
