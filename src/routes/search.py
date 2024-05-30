from fastapi import APIRouter, Security, HTTPException, Response
from fastapi.security import APIKeyHeader
from typing_extensions import Optional, List

from src.schemas import SearchResult
from src.services import get_search
from src.services.get_search import Type

router = APIRouter()


@router.get("/search",
            summary="Search for a stock",
            description="Search for a stock by name or symbol.",
            response_model=List[SearchResult],
            response_description="List of search results",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Query parameter is required"}})
async def search(response: Response, query: str, type: Optional[Type] = None):
    if not query:
        raise HTTPException(status_code=400, detail="Query parameter is required")
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await get_search(query, type)
