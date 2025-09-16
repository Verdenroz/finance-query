from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models.holders import HoldersData, HolderType
from src.services.holders.get_holders import get_holders_data

router = APIRouter()


@router.get(
    path="/holders/{symbol}",
    summary="Get holders data for a stock",
    description="Returns the requested holders information for a stock symbol.",
    response_model=HoldersData,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": HoldersData, "description": "Successfully retrieved holders data"},
        404: {
            "description": "Symbol not found or no holders data available",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def holders(
    symbol: str,
    holder_type: HolderType = Query(..., description="The type of holders data to retrieve."),
):
    return await get_holders_data(symbol, holder_type)
