from datetime import datetime

import pytz
from fastapi import APIRouter, Depends

from src.market import MarketSchedule

router = APIRouter()


@router.get("/hours")
async def get_market_status(market_schedule: MarketSchedule = Depends(MarketSchedule)):
    status, reason = market_schedule.get_market_status()
    return {
        "status": status,
        "reason": reason,
        "timestamp": datetime.now(pytz.UTC).isoformat()
    }
