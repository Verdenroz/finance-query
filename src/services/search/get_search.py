from src.models import SearchResult, Type
from src.services.search.fetchers import fetch_algolia_search_results, fetch_yahoo_search_results
from src.utils.dependencies import FinanceClient
from src.utils.retry import retry


@retry(fetch_yahoo_search_results)
async def get_search(finance_client: FinanceClient, query: str, hits: int = 10, type: Type = None, yahoo: bool = False) -> list[SearchResult]:
    """
    Search for a stock by name or symbol, filtering by its type and limiting the number of hits to 1-20
    :param finance_client: the finance client to use for fetching data
    :param query: the search query
    :param hits: the number of hits to return (1-20)
    :param type: the type of security to filter by (stock, etf, trust)
    :param yahoo: whether to use Yahoo Finance for search results
    """
    if yahoo:
        return await fetch_yahoo_search_results(finance_client, query, hits, type)

    return await fetch_algolia_search_results(query, hits, type)
