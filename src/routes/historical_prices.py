from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import HistoricalData, Interval, TimeRange, ValidationErrorResponse
from src.services import get_historical
from src.utils.dependencies import FinanceClient
from src.utils.logging import get_logger

router = APIRouter()
logger = get_logger(__name__)


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
    },
)
async def get_time_series(
    finance_client: FinanceClient,
    symbol: str = Query(..., description="The symbol of the stock to get historical data for."),
    time_range: TimeRange = Query(..., description="The range of time for the historical data.", alias="range"),
    interval: Interval = Query(..., description="The interval for the historical data."),
    epoch: bool = Query(False, description="Whether to format dates as strings or use epoch timestamps."),
):
    logger.info(
        "Fetching historical data", 
        extra={
            "symbol": symbol.upper(), 
            "time_range": time_range.value, 
            "interval": interval.value,
            "epoch": epoch
        }
    )
    
    try:
        result = await get_historical(finance_client, symbol, time_range, interval, epoch)
        # Log result count if result is a dict with historical data
        data_points = len(result.get(symbol.upper(), {}).get("data", [])) if isinstance(result, dict) else 0
        logger.info(
            "Successfully fetched historical data",
            extra={
                "symbol": symbol.upper(),
                "time_range": time_range.value,
                "interval": interval.value,
                "data_points": data_points
            }
        )
        return result
    except Exception as e:
        logger.error(
            "Failed to fetch historical data",
            extra={
                "symbol": symbol.upper(),
                "time_range": time_range.value,
                "interval": interval.value,
                "error": str(e)
            },
            exc_info=True
        )
        raise
