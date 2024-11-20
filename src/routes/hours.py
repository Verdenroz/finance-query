from datetime import datetime

import pytz
from fastapi import APIRouter, Depends, Security
from fastapi.security import APIKeyHeader

from src.market import MarketSchedule

router = APIRouter()


@router.get("/hours",
            summary="Get the current market status",
            description="Returns the current status of the market (open, closed, early close).",
            response_description="Market status",
            tags=["Hours"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
async def get_market_hours(market_schedule: MarketSchedule = Depends(MarketSchedule)):
    status, reason = market_schedule.get_market_status()
    return {
        "status": status,
        "reason": reason,
        "timestamp": datetime.now(pytz.UTC).isoformat()
    }
