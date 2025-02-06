from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas import SearchResult, Type, ValidationErrorResponse
from src.services import get_search

router = APIRouter()


@router.get(
    path="/search",
    summary="Get stocks by name or symbol",
    description="Search for a stock by name or symbol, filtering by its type (stock, etf, trust) and limiting the "
                "number of hits to 1-20",
    response_model=list[SearchResult],
    tags=["Search"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": list[SearchResult],
            "description": "Search results returned successfully"
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "examples": {
                        "empty_query": {
                            "summary": "Empty query parameter",
                            "value": {
                                "detail": "Invalid request",
                                "errors": {
                                    "query": ["Field required"],
                                }
                            }
                        },
                        "invalid_hits": {
                            "summary": "Hits out of range",
                            "value": {
                                "detail": "Invalid request",
                                "errors": {
                                    "hits": [
                                        "Input should be less than or equal to 20"
                                    ]
                                }
                            }
                        },
                        "invalid_request": {
                            "summary": "Invalid query parameters",
                            "value": {
                                "detail": "Invalid request",
                                "errors": {
                                    "type": ["Input should be 'stock', 'etf', or 'trust'"],
                                    "hits": ["Input should be a valid integer, unable to parse string as an integer"]
                                }
                            }
                        }
                    }
                }
            }
        }
    }
)
async def search(
        query: str,
        type: Optional[Type] = None,
        hits: Optional[int] = Query(default=10, ge=1, le=20)
):
    return await get_search(query, type, hits)
