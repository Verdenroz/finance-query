from fastapi import APIRouter, Security, HTTPException, Query
from fastapi.security import APIKeyHeader
from typing_extensions import List
from src.schemas import Stock
from src.services import scrape_similar_stocks
from src.utils import cache

router = APIRouter()


@router.get("/similar-stocks",
            summary="Returns similar stocks of a queried single stock",
            description="Get relevant stock information for similar stocks."
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[Stock],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
@cache(600)
async def get_similar_stocks(
        symbol: str = Query(..., title="Symbol", description="Stock to find similar stocks around")):
    if not symbol:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    return await scrape_similar_stocks(symbol)
