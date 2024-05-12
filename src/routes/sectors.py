from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from src.services import get_sectors

router = APIRouter()


@router.get("/sectors",
            summary="Get all sectors",
            description="Get all sectors available in the stock market",
            response_description="Summary of all sectors",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
async def sector():
    return await get_sectors()
