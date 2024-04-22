from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from src.schemas import TimeSeries
from src.utils import TimePeriod, Interval
from src.services import scrape_historical
router = APIRouter()


@router.get("/v1/historical",
            summary="Returns historical data for a stock",
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_time_series(symbol: str, time: TimePeriod, interval: Interval):
    return scrape_historical(symbol, time, interval)
