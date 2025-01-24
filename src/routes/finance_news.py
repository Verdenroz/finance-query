from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas import News
from src.services import scrape_news_for_quote, scrape_general_news

router = APIRouter()


@router.get("/news",
            summary="Returns news for a single stock or general market news",
            description="Get relevant stock news for a single stock or general market news.",
            response_model=list[News],
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            tags=["News"],
            responses={400: {"description": "Symbol parameter is required"}})
async def get_news(
        symbol: Optional[str] = Query(
            None,
            description="Optional symbol to get news for. If not provided, general market news is returned")
):
    if not symbol:
        return await scrape_general_news()
    else:
        return await scrape_news_for_quote(symbol)
