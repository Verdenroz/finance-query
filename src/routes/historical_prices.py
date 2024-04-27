from fastapi import APIRouter, Security, Query, HTTPException
from fastapi.security import APIKeyHeader

from src.schemas import TimeSeries
from src.schemas.time_series import TimePeriod, Interval
from src.utils import cache
from src.services import get_historical

router = APIRouter()


@router.get("/v1/historical",
            summary="Returns historical data for a stock",
            response_model=TimeSeries,
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=1800)
async def get_time_series(
        symbol: str = Query(..., description="The symbol of the stock to get historical data for."),
        time: TimePeriod = Query(..., description="The time period for the historical data."),
        interval: Interval = Query(..., description="The interval for the historical data.")
):
    # Validate the combination of time period and interval
    if (interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES] and time not in
            [TimePeriod.DAY,
             TimePeriod.FIVE_DAYS,
             TimePeriod.ONE_MONTH]):
        raise HTTPException(status_code=400, detail="If interval is 15m or 30m, time period must be 1mo, 5d, or 1d")

    return await get_historical(symbol, time, interval)
