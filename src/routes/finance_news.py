from typing import Optional

from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models import News, ValidationErrorResponse
from src.services import scrape_general_news, scrape_news_for_quote

router = APIRouter()


@router.get(
    path="/news",
    summary="Get financial news",
    description="Fetch news for a specific stock, ETF, or general market news. Supports global stock exchanges and provides flexible symbol lookup.",
    response_model=list[News],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": list[News],
            "description": "Successfully retrieved news",
        },
        404: {
            "description": "No news found",
            "content": {"application/json": {"example": {"detail": "No news found for the given symbol"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {"application/json": {"example": {"detail": "Invalid request"}}},
        },
    },
)
async def get_news(
    symbol: Optional[str] = Query(None, description="Optional symbol to get news for. If not provided, general market news is returned"),
):
    if not symbol:
        result = await scrape_general_news()
    else:
        result = await scrape_news_for_quote(symbol)

    return result
