from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import MarketMover, ValidationErrorResponse
from src.models.marketmover import MoverCount
from src.services import get_actives, get_gainers, get_losers
from src.utils.logging import get_logger, log_route_error, log_route_request, log_route_success

router = APIRouter()
logger = get_logger(__name__)


@router.get(
    path="/actives",
    summary="Get list of most active stocks",
    description="Returns summary data for the most active stocks during the current trading session, including the "
    "symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successful retrieved most active stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def actives(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    log_route_request(logger, "actives", {"count": count.value})
    try:
        result = await get_actives(count)
        log_route_success(logger, "actives", {"count": count.value}, {"result_count": len(result)})
        return result
    except Exception as e:
        log_route_error(logger, "actives", {"count": count.value}, e)
        raise


@router.get(
    path="/gainers",
    summary="Get list of stocks with the highest price increase",
    description="Returns the top gaining stocks or funds during the current trading session, including the symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successfully retrieved top gaining stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def gainers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    log_route_request(logger, "gainers", {"count": count.value})
    try:
        result = await get_gainers(count)
        log_route_success(logger, "gainers", {"count": count.value}, {"result_count": len(result)})
        return result
    except Exception as e:
        log_route_error(logger, "gainers", {"count": count.value}, e)
        raise


@router.get(
    path="/losers",
    summary="Get list of stocks with the highest price decrease",
    description="Returns the top losing stocks or funds during the current trading session, including the symbol, name, price, change, and percent change.",
    response_model=list[MarketMover],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketMover], "description": "Successfully retrieved top losing stocks"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {"count": ["Input should be '25', '50' or '100'"]},
                    }
                }
            },
        },
    },
)
async def losers(count: MoverCount = Query(MoverCount.FIFTY, description="Number of movers to retrieve")):
    log_route_request(logger, "losers", {"count": count.value})
    try:
        result = await get_losers(count)
        log_route_success(logger, "losers", {"count": count.value}, {"result_count": len(result)})
        return result
    except Exception as e:
        log_route_error(logger, "losers", {"count": count.value}, e)
        raise
