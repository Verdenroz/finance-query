from fastapi import APIRouter, Security, Response
from fastapi.security import APIKeyHeader
from typing_extensions import List

from src.schemas import Index
from src.services import scrape_indices

router = APIRouter()


@router.get("/indices",
            summary="Returns US indices",
            description="Get the latest US indices data.",
            response_model=List[Index],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_indices(response: Response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await scrape_indices()
