from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import Quote, SimpleQuote, ValidationErrorResponse
from src.services import get_quotes, get_simple_quotes
from utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/quotes",
    summary="Get detailed data for multiple stocks",
    description="Returns detailed quote data including all available fields for multiple stocks.",
    response_model=list[Quote],
    response_model_exclude_none=True,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[Quote], "description": "Successfully retrieved quotes"},
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {"application/json": {"example": {"detail": "Invalid request", "errors": {"symbols": ["Field required"]}}}},
        },
    },
)
async def get_quote(
    finance_client: FinanceClient,
    symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols"),
):
    symbols = list(set(symbols.upper().replace(" ", "").split(",")))
    return await get_quotes(finance_client, symbols)


@router.get(
    path="/simple-quotes",
    summary="Get summary data for multiple stocks",
    description="Returns a simplified version of quote data for multiple stocks, including only symbols, names, prices, changes, and logos.",
    response_model=list[SimpleQuote],
    response_model_exclude_none=True,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": list[SimpleQuote], "description": "Successfully retrieved quotes"},
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {"application/json": {"example": {"detail": "Invalid request", "errors": {"symbols": ["Field required"]}}}},
        },
    },
)
async def get_simple_quote(
    finance_client: FinanceClient,
    symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols"),
):
    symbols = list(set(symbols.upper().replace(" ", "").split(",")))
    return await get_simple_quotes(finance_client, symbols)
