import os
from enum import Enum

from algoliasearch.search_client import SearchClient
from dotenv import load_dotenv
from typing_extensions import Optional

from src.schemas import SearchResult

load_dotenv()


class Type(Enum):
    STOCK = "stock"
    ETF = "etf"
    TRUST = "trust"


async def get_search(query: str, type: Optional[Type] = None):
    client = SearchClient.create(os.getenv("ALGOLIA_APP_ID"), os.getenv("ALGOLIA_KEY"))
    index = client.init_index("stocks")

    # Search parameters
    params = {
        "attributesToRetrieve": ['name', 'symbol', 'exchangeShortName', 'type'],
        "hitsPerPage": 10,
    }

    # If type is not None, add a facetFilters parameter to filter the results by type
    if type is not None:
        params["facetFilters"] = [f"type:{type.value}"]

    results = index.search(query, params)

    stocks = []
    for result in results['hits']:
        stocks.append(SearchResult(name=result['name'], symbol=result['symbol'], exchange=result['exchangeShortName'],
                                   type=result['type']))

    return stocks
