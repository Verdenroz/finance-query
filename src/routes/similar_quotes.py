from fastapi import APIRouter, Security, HTTPException, Query
from fastapi.security import APIKeyHeader

from src.schemas import SimpleQuote
from src.services import scrape_similar_quotes

router = APIRouter()


@router.get("/similar",
            summary="Returns similar stocks of a queried single stock",
            description="Get relevant stock information for similar stocks.",
            response_model=list[SimpleQuote],
            response_model_exclude_none=True,
            tags=["Similar Quotes"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
async def get_similar_quotes(
        symbol: str = Query(..., title="Symbol", description="Stock to find similar stocks around"),
        limit: int = Query(default=10, title="Limit", description="Number of similar stocks to return")
):
    if not symbol:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    return await scrape_similar_quotes(symbol.upper(), limit)
