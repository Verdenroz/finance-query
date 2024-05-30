from fastapi import APIRouter, Security, Query, HTTPException, Response
from fastapi.security import APIKeyHeader

from src.schemas import TimeSeries
from src.schemas.time_series import TimePeriod, Interval
from src.utils import cache
from src.services import get_historical

router = APIRouter()


@router.get("/historical",
            summary="Returns historical data for a stock",
            response_model=TimeSeries,
            description="Get the latest US indices data.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=60, after_market_expire=600)
async def get_time_series(
        response: Response,
        symbol: str = Query(..., description="The symbol of the stock to get historical data for."),
        time: TimePeriod = Query(..., description="The time period for the historical data."),
        interval: Interval = Query(..., description="The interval for the historical data.")
):
    response.headers["Access-Control-Allow-Origin"] = "*"
    # Validate the combination of time period and interval
    if (interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES] and time not in
            [TimePeriod.DAY,
             TimePeriod.FIVE_DAYS,
             TimePeriod.ONE_MONTH]):
        raise HTTPException(status_code=400, detail="If interval is 15m or 30m, time period must be 1mo, 5d, or 1d")

    if interval == Interval.ONE_HOUR and time not in [TimePeriod.DAY, TimePeriod.FIVE_DAYS, TimePeriod.ONE_MONTH,
                                                      TimePeriod.THREE_MONTHS, TimePeriod.SIX_MONTHS, TimePeriod.YTD,
                                                      TimePeriod.YEAR]:
        raise HTTPException(status_code=400, detail="If interval is 1h, time period must be 1Y or less")

    return await get_historical(symbol, time, interval)
