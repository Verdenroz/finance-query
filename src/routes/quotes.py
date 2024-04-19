from fastapi import APIRouter, Security, HTTPException, Query
from fastapi.security import APIKeyHeader
from typing_extensions import List

from ..schemas.quote import Quote
from ..services.scrape_quotes import scrape_quotes

router = APIRouter()


@router.get("/v1/quotes/",
            summary="Returns quote data of multiple stocks",
            description="Get relevant stock information for multiple stocks. Invalid API keys are limited to 5 requests per "
                        "minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbols parameter is required"}})
async def get_quotes(symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbols parameter is required")
    symbols = symbols.split(',')
    return await scrape_quotes(symbols)
