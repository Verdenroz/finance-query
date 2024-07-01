from typing_extensions import Optional

from fastapi import APIRouter, Security, Response, Query
from fastapi.security import APIKeyHeader

from src.services import get_sectors
from src.services.get_sectors import get_sector_for_symbol

router = APIRouter()


@router.get("/sectors",
            summary="Get all sectors",
            description="Get all sectors available in the stock market",
            response_description="Summary of all sectors",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
async def sector(
        response: Response,
        symbol: Optional[str] = Query(
            None,
            description="Optional symbol to get news for. If not provided, general market news is returned"),

):
    response.headers["Access-Control-Allow-Origin"] = "*"
    if not symbol:
        return await get_sectors()
    else:
        return await get_sector_for_symbol(symbol)
