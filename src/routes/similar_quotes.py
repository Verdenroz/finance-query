from fastapi import APIRouter, Security, Query, Depends
from fastapi.security import APIKeyHeader

from src.dependencies import get_yahoo_cookies, get_yahoo_crumb
from src.schemas import SimpleQuote
from src.services.similar import get_similar_quotes

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
            "detail": "Invalid request",
            "errors": {
                "limit": [
                    "Input should be greater than or equal to 1 and less than or equal to 20"
                ],
            }
        }
    }
)
async def similar_quotes(
        cookies: str = Depends(get_yahoo_cookies),
        crumb: str = Depends(get_yahoo_crumb),
        symbol: str = Query(..., title="Symbol", description="Stock to find similar stocks around"),
        limit: int = Query(default=10, title="Limit", description="Number of similar stocks to return", ge=1, le=20),
):
    return await get_similar_quotes(cookies, crumb, symbol.upper(), limit)
