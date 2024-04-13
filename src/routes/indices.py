from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader
from typing_extensions import List
from src.services.scrape_indices import scrape_indices
from src import schemas

router = APIRouter()


@router.get("/v1/indices",
            summary="Returns US indices",
            description="Get the latest US indices data. Invalid API keys are limited to 5 requests per minute.",
            response_model=List[schemas.Index],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_indices():
    return await scrape_indices()
