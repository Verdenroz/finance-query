from fastapi import APIRouter, Security, HTTPException, Query
from fastapi.security import APIKeyHeader
from typing_extensions import List
from src.schemas import Quote, Stock
from src.services import scrape_quotes, scrape_simple_quotes
from src.utils import cache

router = APIRouter()


@router.get("/quotes/",
            summary="Returns quote data of multiple stocks",
            description="Get relevant stock information for multiple stocks. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[Quote],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbols parameter is required"}})
@cache(30, after_market_expire=600)
async def get_quotes(symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbols parameter is required")
    symbols = list(set(symbols.upper().replace(' ', '').split(',', )))
    return await scrape_quotes(symbols)


@router.get("/simple-quotes/",
            summary="Returns summary quote data of a single stock",
            description="Get relevant stock information for a single stock. "
                        "Invalid API keys are limited to 5 requests per minute.",
            response_model=List[Stock],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
@cache(30, after_market_expire=600)
async def get_simple_quote(symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    return await scrape_simple_quotes(symbols)