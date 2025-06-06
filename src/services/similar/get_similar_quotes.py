from src.models import SimpleQuote
from src.services.similar.fetchers import fetch_similar, scrape_similar_quotes
from src.utils.cache import cache
from src.utils.dependencies import FinanceClient
from src.utils.retry import retry


@cache(expire=15, market_closed_expire=600)
@retry(scrape_similar_quotes)
async def get_similar_quotes(finance_client: FinanceClient, symbol: str, limit: int = 10) -> list[SimpleQuote]:
    """
    Get similar stocks by API or scrape if API fails
    :param finance_client: The Yahoo Finance client to use for API requests
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return

    :return: a list of SimpleQuote objects
    """
    return await fetch_similar(finance_client, symbol, limit)
