from datetime import datetime

import pytz
from fastapi import APIRouter, Depends, Security
from fastapi.security import APIKeyHeader

from src.utils.logging import get_logger, log_route_error, log_route_request, log_route_success
from src.utils.market import MarketSchedule

router = APIRouter()
logger = get_logger(__name__)


@router.get(
    path="/hours",
    summary="Get the current market status",
    description="Returns the current status of the market (open, closed, early close, pre-market, after-market).",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Market status",
            "content": {
                "application/json": {
                    "examples": {
                        "open": {
                            "summary": "Market is open",
                            "value": {
                                "status": "open",
                                "reason": "Regular trading hours.",
                                "timestamp": "2021-09-22T14:00:00.000Z",
                            },
                        },
                        "closed": {
                            "summary": "Market is closed",
                            "value": {"status": "closed", "reason": "Weekend", "timestamp": "2021-09-25T14:00:00.000Z"},
                        },
                        "early_close": {
                            "summary": "Market closed early",
                            "value": {
                                "status": "early_close",
                                "reason": "Early Close: Christmas Eve",
                                "timestamp": "2021-12-24T18:00:00.000Z",
                            },
                        },
                        "pre_market": {
                            "summary": "Pre-market hours",
                            "value": {
                                "status": "closed",
                                "reason": "Pre-market",
                                "timestamp": "2021-09-22T12:00:00.000Z",
                            },
                        },
                        "after_market": {
                            "summary": "After-market hours",
                            "value": {
                                "status": "closed",
                                "reason": "After-hours",
                                "timestamp": "2021-09-22T22:00:00.000Z",
                            },
                        },
                    }
                }
            },
        }
    },
)
async def get_market_hours(market_schedule: MarketSchedule = Depends(MarketSchedule)):
    params = {}
    log_route_request(logger, "hours", params)

    try:
        status, reason = market_schedule.get_market_status()
        result = {"status": status, "reason": reason, "timestamp": datetime.now(pytz.UTC).isoformat()}
        log_route_success(logger, "hours", params, {"status": status, "reason": reason})
        return result
    except Exception as e:
        log_route_error(logger, "hours", params, e)
        raise
