from fastapi import APIRouter, Security, Query, HTTPException
from fastapi.security import APIKeyHeader

from src.schemas import TimeSeries, ValidationErrorResponse, TimePeriod, Interval
from src.services import get_historical

router = APIRouter()


@router.get(
    path="/historical",
    summary="Get historical data for a stock",
    description="Returns historical data, including its date and OHLCV, for a stock symbol based on the time "
                "period and interval provided.",
    response_model=TimeSeries,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    tags=["Historical Data"],
    responses={
        200: {
            "model": TimeSeries,
            "description": "Successfully retrieved historical data"
        },
        400: {
            "description": "Bad request",
            "content": {"application/json":
                {
                    "example": {"detail": "If interval is 1m, 5m, 15m or 30m, time period must be 1mo or less"}
                }
            }
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}}
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbol": ["Field required"],
                            "time": [
                                "Field required",
                                "Input should be '1d', '5d', '7d', '1mo', '3mo', '6mo', 'YTD', '1Y', '5Y', '10Y' or 'max'"
                            ],
                            "interval": [
                                "Field required",
                                "Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', '1mo' or '3mo'"
                            ]
                        }
                    }
                }
            }
        },
        500: {
            "description": "Internal server error",
            "content": {"application/json": {"example": {"detail": "Failed to retrieve historical data"}}}
        }
    }
)
async def get_time_series(
        symbol: str = Query(..., description="The symbol of the stock to get historical data for."),
        time: TimePeriod = Query(..., description="The time period for the historical data."),
        interval: Interval = Query(..., description="The interval for the historical data."),
        epoch: bool = Query(False, description="Whether to format dates as strings or use epoch timestamps.")
):
    # Validate the combination of time period and interval
    if (interval in [Interval.ONE_MINUTE, Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES] and
            time not in [TimePeriod.DAY, TimePeriod.FIVE_DAYS, TimePeriod.SEVEN_DAYS, TimePeriod.ONE_MONTH]):
        raise HTTPException(status_code=400,
                            detail="If interval is 1m, 5m, 15m or 30m, time period must be 1mo or less")

    if interval == Interval.ONE_HOUR and time not in [TimePeriod.DAY, TimePeriod.FIVE_DAYS, TimePeriod.SEVEN_DAYS,
                                                      TimePeriod.ONE_MONTH, TimePeriod.THREE_MONTHS,
                                                      TimePeriod.SIX_MONTHS,
                                                      TimePeriod.YTD, TimePeriod.YEAR]:
        raise HTTPException(status_code=400, detail="If interval is 1h, time period must be 1Y or less")

    try:
        return await get_historical(symbol, time, interval, epoch)
    except HTTPException as e:
        # Re-raise HTTPExceptions with their status code and detail
        raise e
