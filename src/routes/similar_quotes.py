from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader

from src.schemas import SimpleQuote, ValidationErrorResponse
from src.services import scrape_similar_quotes

router = APIRouter()


@router.get(
    path="/similar",
    summary="Get similar quotes to a queried symbol",
    description="Returns simplified quote data for similar stocks to a queried symbol,"
                "including symbol, name, price, and percent change.",
    response_model=list[SimpleQuote],
    response_model_exclude_none=True,
    tags=["Similar Quotes"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": list[SimpleQuote],
            "description": "Similar stocks found.",
            "content": {
                "application/json": {
                    "example": [
                        {
                            "symbol": "AAPL",
                            "name": "Apple Inc.",
                            "price": "146.06",
                            "percent_change": "-0.11%"
                        }
                    ]
                }
            }
        },
        404: {
            "description": "No similar stocks found or invalid symbol.",
            "content": {"application/json": {"example": {"detail": "No similar stocks found or invalid symbol"}}}
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbol": ["Field required"],
                            "limit": ["Input should be a valid integer, unable to parse string as an integer"]
                        }
                    }
                }
            }
        }
    }
)
async def get_similar_quotes(
        symbol: str = Query(..., title="Symbol", description="Stock to find similar stocks around"),
        limit: int = Query(default=10, title="Limit", description="Number of similar stocks to return")
):
    return await scrape_similar_quotes(symbol.upper(), limit)
