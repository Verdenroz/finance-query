from fastapi import APIRouter, Security, HTTPException
from fastapi.security import APIKeyHeader
from ..schemas.quote import Quote
from ..services.scrape_quote import scrape_quote

router = APIRouter()


@router.get("/v1/quote/",
            summary="Returns quote data of a stock",
            description="Get relevant stock information for a stock. Invalid API keys are limited to 5 requests per "
                        "minute.",
            response_model=Quote,
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            responses={400: {"description": "Symbol parameter is required"}})
async def get_quote(symbol: str):
    if not symbol:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")
    return await scrape_quote(symbol)
