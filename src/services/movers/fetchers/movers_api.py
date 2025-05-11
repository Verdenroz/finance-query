from orjson import orjson

from utils.dependencies import fetch
from src.models import MarketMover


async def fetch_movers(url: str) -> list[MarketMover]:
    """
    Fetch the most active, gainers, or losers from Yahoo Finance
    :param url: the Yahoo Finance URL to scrape
    """
    params = {
        "fields": "symbol,longName,shortName,regularMarketPrice,regularMarketChange,regularMarketChangePercent",
    }

    response = await fetch(url=url, params=params)
    response = orjson.loads(response)

    data = response.get("finance", {}).get("result", [{}])[0].get("quotes", [])
    movers = []

    def get_fmt(field: dict, key: str):
        return field.get(key, {}).get("fmt")

    for item in data:
        symbol = item.get("symbol")
        name = item.get("longName", item.get("shortName"))
        price = get_fmt(item, "regularMarketPrice")
        change = get_fmt(item, "regularMarketChange")
        percent_change = get_fmt(item, "regularMarketChangePercent")

        mover = MarketMover(symbol=symbol, name=name, price=price, change=change, percent_change=percent_change)
        movers.append(mover)

    return movers
