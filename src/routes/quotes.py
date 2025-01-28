from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader

from src.schemas import Quote, SimpleQuote, ValidationErrorResponse
from src.services import scrape_quotes, scrape_simple_quotes

router = APIRouter()


@router.get(
    path="/quotes",
    summary="Get detailed data for multiple stocks",
    description="Returns detailed quote data including all available fields for multiple stocks.",
    response_model=list[Quote],
    response_model_exclude_none=True,
    tags=["Quotes"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[Quote], "description": "Successfully retrieved quotes"},
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}}
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbols": ["Field required"]
                        }
                    }
                }
            }
        },
        500: {
            "description": "Failed to get quote",
            "content": {"application/json": {"example": {"detail": "Failed to get quote for {symbol}"}}}
        }
    }
)
async def get_quotes(symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    quotes = await scrape_quotes(symbols)

    return [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]


@router.get(
    path="/simple-quotes",
    summary="Get summary data for multiple stocks",
    description="Returns a simplified version of quote data for multiple stocks, including only symbols, names, "
                "prices, changes, and logos.",
    response_model=list[SimpleQuote],
    response_model_exclude_none=True,
    tags=["Quotes"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[SimpleQuote], "description": "Successfully retrieved quotes"},
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}}
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbols": ["Field required"]
                        }
                    }
                }
            }
        },
        500: {
            "description": "Failed to get simple quote",
            "content": {"application/json": {"example": {"detail": "Failed to get simple quote for {symbol}"}}}
        }
    }
)
async def get_simple_quote(
        symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    quotes = await scrape_simple_quotes(symbols)

    return [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]
