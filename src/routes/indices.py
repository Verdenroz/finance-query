from typing import Annotated

from fastapi import APIRouter, Security, Depends, Query
from fastapi.security import APIKeyHeader

from src.dependencies import get_yahoo_cookies, get_yahoo_crumb
from src.models import MarketIndex, Index, Region, INDEX_REGIONS
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
    }
)
async def market_indices(
        cookies: str = Depends(get_yahoo_cookies),
        crumb: str = Depends(get_yahoo_crumb),
        index: Annotated[list[Index] | None, Query(description="Specific indices to include")] = None,
        region: Annotated[Region | None, Query(description="Filter indices by region")] = None
) -> list[MarketIndex]:
    selected_indices = set(index or [])

    # Add indices from selected region to the set
    if region:
        region_indices = [idx for idx in Index if INDEX_REGIONS.get(idx) == region or
                          (INDEX_REGIONS.get(idx) == Region.UNITED_STATES and region == Region.NORTH_AMERICA)]
        selected_indices.update(region_indices)

    # If nothing was selected, use all indices
    if not selected_indices and not index and not region:
        return await get_indices(cookies, crumb, list(Index))

    # Convert back to ordered list by iterating through the original enum order
    # Only include indices that were selected
    ordered_indices = [idx for idx in Index if idx in selected_indices]

    return await get_indices(cookies, crumb, ordered_indices)
