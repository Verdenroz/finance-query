from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List

from src.schemas import Index
from src.services import scrape_indices

router = APIRouter()


@router.get("/indices",
            summary="Returns US indices",
            description="Get the latest US indices data.",
            response_model=List[Index],
            tags=["Indices"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_indices():
    return await scrape_indices()
