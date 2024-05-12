from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from src.services import get_sectors
from src.utils import cache

router = APIRouter()


@router.get("/sectors",
            summary="Get all sectors",
            description="Get all sectors available in the stock market",
            response_description="Summary of all sectors",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
@cache(expire=300, after_market_expire=3600)
async def sector():
    return await get_sectors()
