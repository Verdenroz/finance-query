from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas.sector import Sector
from src.services import get_sectors
from src.services.get_sectors import get_sector_for_symbol, get_sector_details

router = APIRouter()


@router.get("/sectors",
            summary="Get all sectors",
            description="Get all sectors available in the stock market",
            response_description="Summary of all sectors",
            tags=["Sectors"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
async def sector(
        symbol: Optional[str] = Query(
            None,
            description="Optional symbol to get info for. If not provided, all sectors are returned with summary "
                        "information"),
        name: Optional[Sector] = Query(
            None,
            description="Optional sector name to get detailed info for. If not provided, all sectors are returned with "
                        "summary information"
        )
):
    if symbol and not name:
        return await get_sector_for_symbol(symbol)

    if name and not symbol:
        return await get_sector_details(name)

    return await get_sectors()
