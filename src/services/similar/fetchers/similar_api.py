from fastapi import HTTPException
from orjson import orjson

from utils.dependencies import fetch
from src.models import SimpleQuote
from src.services import get_simple_quotes


async def fetch_similar(symbol: str, limit: int, cookies: dict, crumb: str) -> list[SimpleQuote]:
    """
    Get similar stocks by API
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the maximum number of results to return
    :param cookies: authentication cookies for Yahoo Finance
    :param crumb: authentication crumb for Yahoo Finance

    :return: a list of SimpleQuote objects
    """
    symbols = await _fetch_yahoo_recommended_symbols(symbol, limit)
    return await get_simple_quotes(symbols, cookies, crumb)


async def _fetch_yahoo_recommended_symbols(symbol: str, limit: int) -> list[str]:
    """
    Fetch similar symbols from Yahoo Finance API
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the maximum number of results to return

    :return: a list of symbols recommended by Yahoo Finance
    """
    YAHOO_RECOMMENDATION_URL = f"https://query1.finance.yahoo.com/v6/finance/recommendationsbysymbol/{symbol}"
    params = {"count": limit}
    response = await fetch(url=YAHOO_RECOMMENDATION_URL, params=params)
    response = orjson.loads(response)

    data = response.get("finance", {}).get("result", [{}])[0]
    recommendations = data.get("recommendedSymbols", [])

    if not recommendations:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

    symbols = []
    for recommendation in recommendations:
        symbol = recommendation.get("symbol")
        if symbol:
            symbols.append(symbol)

    return symbols
