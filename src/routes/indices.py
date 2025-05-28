from typing import Annotated

from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import INDEX_REGIONS, Index, MarketIndex, Region
from src.services import get_indices
from utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/indices",
    summary="Get major world market indices performance",
    description="Returns the major world market indices performance including the name, value, change, and percent change.",
    response_model=list[MarketIndex],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketIndex], "description": "Successfully retrieved indices"},
    },
)
async def market_indices(
    finance_client: FinanceClient,
    index: Annotated[list[Index] | None, Query(description="Specific indices to include")] = None,
    region: Annotated[Region | None, Query(description="Filter indices by region")] = None,
) -> list[MarketIndex]:
    selected_indices = set(index or [])
    # Add indices from selected region to the set
    if region:
        region_indices = [
            idx for idx in Index if INDEX_REGIONS.get(idx) == region or (INDEX_REGIONS.get(idx) == Region.UNITED_STATES and region == Region.NORTH_AMERICA)
        ]
        selected_indices.update(region_indices)

    # Convert back to ordered list by iterating through the original enum order
    # Only include indices that were selected
    ordered_indices = [idx for idx in Index if idx in selected_indices]

    return await get_indices(finance_client, ordered_indices)
