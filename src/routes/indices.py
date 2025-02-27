from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from src.models import Index
from src.services import scrape_indices
from src.models import MarketIndex

router = APIRouter()


@router.get(
    path="/indices",
    summary="Get major world market indices performance",
    description="Returns the major world market indices performance including the name, value, change, and percent change.",
    response_model=list[Index],
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
async def get_indices():
    return await scrape_indices()
