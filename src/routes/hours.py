from datetime import datetime

import pytz
from fastapi import APIRouter, Depends, Security
from fastapi.security import APIKeyHeader

from src.market import MarketSchedule

router = APIRouter()


@router.get(
    path="/hours",
    summary="Get the current market status",
    description="Returns the current status of the market (open, closed, early close, pre-market, after-market).",
    tags=["Hours"],
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
                                "timestamp": "2021-09-22T14:00:00.000Z"
                            }
                        },
                        "closed": {
                            "summary": "Market is closed",
                            "value": {
                                "status": "closed",
                                "reason": "Weekend",
                                "timestamp": "2021-09-25T14:00:00.000Z"
                            }
                        },
                        "early_close": {
                            "summary": "Market closed early",
                            "value": {
                                "status": "early_close",
                                "reason": "Early Close: Christmas Eve",
                                "timestamp": "2021-12-24T18:00:00.000Z"
                            }
                        },
                        "pre_market": {
                            "summary": "Market is in pre-market",
                            "value": {
                                "status": "closed",
                                "reason": "Pre-market",
                                "timestamp": "2021-09-22T12:00:00.000Z"
                            }
                        },
                        "after_market": {
                            "summary": "Market is in after-market",
                            "value": {
                                "status": "closed",
                                "reason": "After-hours",
                                "timestamp": "2021-09-22T22:00:00.000Z"
                            }
                        }
                    }
                }
            }
        }
    }
)
async def get_market_hours(market_schedule: MarketSchedule = Depends(MarketSchedule)):
    status, reason = market_schedule.get_market_status()
    return {
        "status": status,
        "reason": reason,
        "timestamp": datetime.now(pytz.UTC).isoformat()
    }
