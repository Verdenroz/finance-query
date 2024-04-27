from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from src.schemas import Index
from src.services import scrape_indices
from src.utils import cache

router = APIRouter()


@router.get("/v1/indices",
            summary="Returns US indices",
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            response_model=List[Index],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
@cache(expire=60, check_market=True)
async def get_indices():
    return await scrape_indices()
