from utils.cache import cache
from src.models import SimpleQuote
from src.services.similar.fetchers import fetch_similar, scrape_similar_quotes


@cache(expire=15, market_closed_expire=600)
async def get_similar_quotes(symbol: str, cookies: dict, crumb: str, limit: int = 10) -> list[SimpleQuote]:
    """
    Get similar stocks by API or scrape if API fails
    :param cookies: authentication cookies for Yahoo Finance
    :param crumb: authentication crumb for Yahoo Finance
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return

    :return: a list of SimpleQuote objects
    """
    similar = await fetch_similar(symbol, limit, cookies, crumb)

    if not similar:
        return await scrape_similar_quotes(symbol, limit)

    return similar
