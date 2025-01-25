from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List

from src.schemas import Index
from src.services import scrape_indices

router = APIRouter()


@router.get(
    path="/indices",
    summary="Get major world market indices performance",
    description="Returns the major world market indices performance including the name, value, change, and percent change.",
    response_model=List[Index],
    tags=["Indices"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": List[Index], "description": "Successfully retrieved indices"},
        500: {"description": "Failed to parse indices"}
    }
)
async def get_indices():
    return await scrape_indices()
