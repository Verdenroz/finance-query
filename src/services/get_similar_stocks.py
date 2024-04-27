from decimal import Decimal

from bs4 import BeautifulSoup, SoupStrainer
from httpx import AsyncClient
from typing_extensions import List

from src.constants import headers
from src.schemas import Stock


async def fetch(url: str, client: AsyncClient):
    response = await client.get(url, headers=headers)
    return response.text


async def scrape_similar_stocks(symbol: str) -> List[Stock]:
    url = 'https://finance.yahoo.com/quote/' + symbol
    html = await fetch(url, AsyncClient())
    soup = BeautifulSoup(html, 'lxml', parse_only=SoupStrainer('div'))
    similar_stocks = soup.find_all("div", class_="main-div svelte-15b2o7n")
    stocks = []

    for div in similar_stocks:
        symbol_element = div.find("span")
        if not symbol_element:
            continue
        div_symbol = symbol_element.text
        if div_symbol.lower() == symbol.lower():
            continue

        name_element = div.find("div", class_="longName svelte-15b2o7n")
        if not name_element:
            continue
        name = name_element.text

        price_element = div.find("span", class_="price svelte-15b2o7n")
        if not price_element:
            continue
        price_text = price_element.text.replace(',', '')
        price = Decimal(price_text)

        change_element = (div.find("span", class_="positive svelte-15b2o7n") or
                          div.find("span", class_="negative svelte-15b2o7n"))
        if not change_element:
            continue
        percent_change = change_element.text

        change = price / (1 + Decimal(percent_change.strip('%')) / 100) - price
        change = round(change, 2)
        if percent_change.startswith('-'):
            change = -change
        else:
            change = +change

        stock = Stock(symbol=div_symbol, name=name, price=price, change=change, percent_change=percent_change)
        stocks.append(stock)
        if len(stocks) == 5:
            break
    return stocks
