from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import HistoricalData, Interval, TimeRange, ValidationErrorResponse
from src.services import get_historical
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/historical",
    summary="Get historical data for a stock",
    description="Returns historical data, including its date and OHLCV, for a stock symbol based on the time period and interval provided.",
    response_model=dict[str, HistoricalData],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": dict[str, HistoricalData], "description": "Successfully retrieved historical data"},
        400: {
            "description": "Bad request",
            "content": {"application/json": {"example": {"detail": "If interval is 1m, 5m, 15m or 30m, time period must be 1mo or less"}}},
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error of query parameters"},
        500: {
            "description": "Internal server error",
            "content": {"application/json": {"example": {"detail": "Failed to retrieve historical data"}}},
        },
    },
)
async def get_time_series(
    finance_client: FinanceClient,
    symbol: str = Query(..., description="The symbol of the stock to get historical data for."),
    time_range: TimeRange = Query(..., description="The range of time for the historical data.", alias="range"),
    interval: Interval = Query(..., description="The interval for the historical data."),
    epoch: bool = Query(False, description="Whether to format dates as strings or use epoch timestamps."),
):
    return await get_historical(finance_client, symbol, time_range, interval, epoch)
