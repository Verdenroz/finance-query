from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from src.schemas import TimeSeries
from src.services import scrape_indices

router = APIRouter()


@router.get("/v1/historical",
            summary="Returns historical data for a stock",
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            response_model=TimeSeries,
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_time_series(symbol: str, interval: str):
    return await scrape_indices()
