from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas import News
from src.schemas.validation_error import ValidationErrorResponse
from src.services import scrape_news_for_quote, scrape_general_news

router = APIRouter()


@router.get(
    path="/news",
    summary="Retrieve stock or market news",
    description="Fetch news for a specific stock, ETF, or general market news. "
                "Supports global stock exchanges and provides flexible symbol lookup.",
    response_model=list[News],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    tags=["News"],
    responses={
        200: {
            "description": "Successfully retrieved news",
            "content": {
                "application/json": {
                    "example": [
                        {
                            "title": "Tech Giant Announces Quarterly Earnings",
                            "link": "https://example.com/news/tech-earnings",
                            "source": "Financial Times",
                            "img": "https://example.com/news-image.jpg",
                            "time": "2 hours ago"
                        }
                    ]
                }
            }
        },
        404: {"description": "No news found for the given symbol"},
        422: {"model": ValidationErrorResponse}
    })
async def get_news(
        symbol: Optional[str] = Query(
            None,
            description="Optional symbol to get news for. If not provided, general market news is returned")
):
    if not symbol:
        return await scrape_general_news()
    else:
        return await scrape_news_for_quote(symbol)
