from typing import Annotated

from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import INDEX_REGIONS, Index, MarketIndex, Region, ValidationErrorResponse
from src.services import get_indices
from src.utils.dependencies import FinanceClient
from src.utils.logging import get_logger, log_route_error, log_route_request, log_route_success

router = APIRouter()
logger = get_logger(__name__)


@router.get(
    path="/indices",
    summary="Get major world market indices performance",
    description="Returns the major world market indices performance including the name, value, change, and percent change.",
    response_model=list[MarketIndex],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[MarketIndex], "description": "Successfully retrieved indices"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "index.0": [
                                """
                                Input should be 'snp', 'djia', 'nasdaq', 'nyse-composite', 'nyse-amex', 'rut', 'vix', 'tsx-composite', 'ibovespa',
                                'ipc-mexico', 'ipsa', 'merval', 'ivbx', 'ibrx-50', 'ftse-100', 'dax', 'cac-40', 'euro-stoxx-50', 'euronext-100', 'bel-20',
                                'moex', 'aex', 'ibex-35', 'ftse-mib', 'smi', 'psi', 'atx', 'omxs30', 'omxc25', 'wig20', 'budapest-se', 'moex-russia', 'rtsi',
                                'hang-seng', 'sti', 'sensex', 'idx-composite', 'ftse-bursa', 'kospi', 'twse', 'nikkei-225', 'shanghai', 'szse-component',
                                'set', 'nifty-50', 'nifty-200', 'psei-composite', 'china-a50', 'dj-shanghai', 'india-vix', 'egx-30', 'jse-40', 'ftse-jse',
                                'afr-40', 'raf-40', 'sa-40', 'alt-15', 'ta-125', 'ta-35', 'tadawul-all-share', 'tamayuz', 'bist-100', 'asx-200',
                                'all-ordinaries', 'nzx-50', 'usd', 'msci-europe', 'gbp', 'euro', 'yen', 'australian', 'msci-world' or 'cboe-uk-100'
                                """
                            ],
                            "region": ["Input should be 'US', 'NA', 'SA', 'EU', 'AS', 'AF', 'ME', 'OCE' or 'global'"],
                        },
                    }
                }
            },
        },
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

    params = {"index": [idx.value for idx in (index or [])], "region": region.value if region else None}
    log_route_request(logger, "indices", params)

    try:
        result = await get_indices(finance_client, ordered_indices)
        log_route_success(logger, "indices", params, {"result_count": len(result)})
        return result
    except Exception as e:
        log_route_error(logger, "indices", params, e)
        raise
