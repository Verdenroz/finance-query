from fastapi import APIRouter, Security, HTTPException
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
        400: {
            "description": "Bad request",
            "content": {
                "application/json": {
                    "examples": {
                        "empty_query": {
                            "summary": "Empty query parameter",
                            "value": {"detail": "Query parameter should not be empty"}
                        },
                        "invalid_hits": {
                            "summary": "Hits out of range",
                            "value": {"detail": "Hits must be between 1 and 20"}
                        }
                    }
                }
            }
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "query": ["Field required"],
                            "type": ["Input should be 'stock', 'etf', or 'trust'"],
                            "hits": ["Input should be a valid integer, unable to parse string as an integer"]
                        }
                    }
                }
            }
        }
    }
)
async def search(query: str, type: Optional[Type] = None, hits: Optional[int] = 10):
    if not query:
        raise HTTPException(status_code=400, detail="Query parameter should not be empty")

    if hits < 1 or hits > 20:
        raise HTTPException(status_code=400, detail="Hits must be between 1 and 20")

    return await get_search(query, type, hits)
