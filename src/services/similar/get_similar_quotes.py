from fastapi import HTTPException

from src.cache import cache
from src.models import SimpleQuote
from src.services.similar.fetchers import fetch_similar, scrape_similar_quotes


@cache(expire=15, market_closed_expire=600)
async def get_similar_quotes(symbol: str, cookies: str, crumb: str, limit: int = 10) -> list[SimpleQuote]:
    """
    Scrape similar stocks from Yahoo Finance for a single symbol
    :param cookies: authentication cookies for Yahoo Finance
    :param crumb: authentication crumb for Yahoo Finance
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return
    :return:
    """
    try:
        return await fetch_similar(symbol, limit, cookies, crumb)
    except HTTPException:
        raise
    except Exception as e:
        print("Error fetching similar stocks by API:", e)
        return await scrape_similar_quotes(symbol, limit)
