from fastapi import HTTPException
from utils.dependencies import FinanceClient

from src.models import SimpleQuote
from src.services import get_simple_quotes


async def fetch_similar(finance_client: FinanceClient, symbol: str, limit: int) -> list[SimpleQuote]:
    """
    Get similar stocks by API
    :param finance_client: The Yahoo Finance client to use for API requests
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the maximum number of results to return

    :return: a list of SimpleQuote objects
    """
    symbols = await _fetch_yahoo_recommended_symbols(finance_client, symbol, limit)
    return await get_simple_quotes(finance_client, symbols)


async def _fetch_yahoo_recommended_symbols(finance_client: FinanceClient, symbol: str, limit: int) -> list[str]:
    """
    Fetch similar symbols from Yahoo Finance API
    :param finance_client: The Yahoo Finance client to use for API requests
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the maximum number of results to return

    :return: a list of symbols recommended by Yahoo Finance
    """
    response = await finance_client.get_similar_quotes(symbol, limit)

    data = response.get("finance", {}).get("result", [{}])[0]
    recommendations = data.get("recommendedSymbols", [])

    if not recommendations:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

    symbols = []
    for recommendation in recommendations:
        symbol = recommendation.get("symbol")
        if symbol:
            symbols.append(symbol)

    return symbols[:limit]
