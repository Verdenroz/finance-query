from fastapi import APIRouter, Security, HTTPException
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas import SearchResult, Type
from src.services import get_search

router = APIRouter()


@router.get("/search",
            summary="Search for a stock",
            description="Search for a stock by name or symbol.",
            response_model=list[SearchResult],
            response_description="List of search results",
            tags=["Search"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Query parameter is required"}})
async def search(query: str, type: Optional[Type] = None, hits: Optional[int] = 10):
    if not query:
        raise HTTPException(status_code=400, detail="Query parameter is required")

    if hits < 1 or hits > 20:
        raise HTTPException(status_code=400, detail="Hits must be between 1 and 20")

    return await get_search(query, type, hits)
