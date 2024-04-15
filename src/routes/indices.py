from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from ..services.scrape_indices import scrape_indices
from ..schemas.index import Index

router = APIRouter()


@router.get("/v1/indices",
            summary="Returns US indices",
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            response_model=List[Index],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_indices():
    return await scrape_indices()
