from pprint import pprint
from typing import Optional

from orjson import orjson

from utils.dependencies import fetch
from src.models import SearchResult, Type


async def fetch_yahoo_search_results(query: str, hits: int, type: Optional[Type]) -> list[SearchResult]:
    """
    Fetch search results from Yahoo Finance
    :param query: the search query
    :param hits: the number of hits to return
    :param type: the type of security to filter by

    :return: a list of search results
    """
    url = "https://query1.finance.yahoo.com/v1/finance/search"
    params = {
        "q": query,
        "quotesCount": hits,
    }

    type_to_yf = {
        Type.STOCK: "EQUITY",
        Type.ETF: "ETF",
        Type.TRUST: "MUTUALFUND",
    }

    yf_to_type = {
        "EQUITY": Type.STOCK,
        "ETF": Type.ETF,
        "MUTUALFUND": Type.TRUST,
    }

    response = await fetch(url=url, params=params)
    response = orjson.loads(response)
    data = response.get("quotes", [])
    pprint(data)
    results = []
    for item in data:
        if len(results) >= hits:
            break

        # If a type is provided, filter the results by that type
        quote_type = item.get("quoteType")
        if type and quote_type != type_to_yf.get(type):
            continue

        # If the quote type is not recognized, skip the item (this is usually for futures and indices)
        if quote_type not in yf_to_type:
            continue

        result = SearchResult(
            name=item.get("shortname", item.get("longname")),
            symbol=item.get("symbol"),
            exchange=item.get("exchange"),
            type=yf_to_type.get(item.get("quoteType"), Type.STOCK.value),
        )
        results.append(result)

    return results
