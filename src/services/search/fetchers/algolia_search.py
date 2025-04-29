import os
from typing import Optional

from algoliasearch.search_client import SearchClient

from src.models import SearchResult, Type


async def fetch_algolia_search_results(query: str, hits: int, type: Optional[Type]) -> list[SearchResult]:
    """
    Search for a stock by name or symbol, filtering by its type and limiting the number of hits to 1-20
    :param query: the search query
    :param type: the type of security to filter by (stock, etf, trust)
    :param hits: the number of hits to return (1-20)
    """

    """    
    My personal Algolia credentials are on free tier and are safe to be shared
    This is out of trust that the search will not be abused
    The API key is search-only and can't be used to modify the data
    If you would like your own personal index, email me and I can send you a json of all tradeable stocks
    """
    client = SearchClient.create(
        app_id=os.environ.get("ALGOLIA_APP_ID", "ZTZOECLXBC"),
        api_key=os.environ.get("ALGOLIA_API_KEY", "a3882d6ec31c0b1063ede94374616d8a"),
    )
    index = client.init_index("stocks")

    # Search parameters
    params = {
        "attributesToRetrieve": ["name", "symbol", "exchangeShortName", "type"],
        "hitsPerPage": hits,
    }

    # If type is not None, add a facetFilters parameter to filter the results by type
    if type is not None:
        params["facetFilters"] = [f"type:{type.value}"]

    results = index.search(query, params)

    stocks = []
    for result in results["hits"]:
        stocks.append(SearchResult(name=result["name"], symbol=result["symbol"], exchange=result["exchangeShortName"], type=result["type"]))

    return stocks
