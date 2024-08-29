from fastapi import APIRouter, Security, HTTPException, Query, Response
from fastapi.security import APIKeyHeader

from src.services import scrape_quotes, scrape_simple_quotes

router = APIRouter()


@router.get("/quotes",
            summary="Returns quote data of multiple stocks",
            description="Get relevant stock information for multiple stocks.",
            response_model_exclude_none=True,
            tags=["Quotes"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbols parameter is required"}})
async def get_quotes(
        response: Response,
        symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols"),
):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbols parameter is required")
    response.headers["Access-Control-Allow-Origin"] = "*"
    symbols = list(set(symbols.upper().replace(' ', '').split(',', )))
    return await scrape_quotes(symbols)


@router.get("/simple-quotes",
            summary="Returns summary quote data of a single stock",
            description="Get relevant stock information for a single stock.",
            tags=["Quotes"],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
async def get_simple_quote(
        response: Response,
        symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")
):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    response.headers["Access-Control-Allow-Origin"] = "*"
    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    return await scrape_simple_quotes(symbols)
