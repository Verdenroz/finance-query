from typing import Optional, Annotated

from fastapi import APIRouter, Security, Depends, Query
from fastapi.security import APIKeyHeader

from src.dependencies import get_yahoo_cookies, get_yahoo_crumb
from src.models import MarketIndex, Index
from src.services import get_indices

router = APIRouter()


@router.get(
    path="/indices",
    summary="Get major world market indices performance",
    description="Returns the major world market indices performance including the name, value, change, and percent change.",
    response_model=list[MarketIndex],
    tags=["Indices"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketIndex], "description": "Successfully retrieved indices"},
        500: {
            "description": "Failed to parse indices",
            "content": {"application/json": {"example": {"detail": "Failed to parse indices"}}}
        }
    }
)
async def market_indices(
        cookies: str = Depends(get_yahoo_cookies),
        crumb: str = Depends(get_yahoo_crumb),
        index: Annotated[list[Index] | None, Query()] = None
) -> list[MarketIndex]:
    # If no index is provided, return all indices
    if not index:
        index = list(Index)
    return await get_indices(cookies, crumb, index)
