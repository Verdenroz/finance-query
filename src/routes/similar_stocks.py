from fastapi import APIRouter, Security, HTTPException, Query, Response
from fastapi.security import APIKeyHeader
from typing_extensions import List

from src.schemas import SimpleQuote
from src.services import scrape_similar_stocks

router = APIRouter()


@router.get("/similar-stocks",
            summary="Returns similar stocks of a queried single stock",
            description="Get relevant stock information for similar stocks.",
            response_model=List[SimpleQuote],
            tags=["Similar Stocks"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
async def get_similar_stocks(
        response: Response,
        symbol: str = Query(..., title="Symbol", description="Stock to find similar stocks around")):
    if not symbol:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await scrape_similar_stocks(symbol)
