from decimal import Decimal

from bs4 import BeautifulSoup
from fastapi import HTTPException
from requests import Session
from typing_extensions import List

from src.constants import headers
from src.schemas import SimpleQuote


def parse_stocks(stocks_divs, symbol):
    stocks = []
    for div in stocks_divs:
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
            change_str = '-' + str(abs(change))
        else:
            change_str = '+' + str(abs(change))

        stock = SimpleQuote(symbol=div_symbol, name=name, price=price, change=change_str, percent_change=percent_change)
        stocks.append(stock)
        if len(stocks) == 5:
            break
    return stocks


def parse_etfs(etf_divs):
    etfs = []
    for div in etf_divs:
        symbol_element = div.find("span", class_="symbol svelte-1ts22zv")
        if not symbol_element:
            continue
        symbol = symbol_element.text

        name_element = div.find("span", class_="tw-text-sm svelte-1ts22zv longName")
        if not name_element:
            continue
        name = name_element.text

        price_element = div.find("strong")
        if not price_element:
            continue
        price_text = price_element.text.replace(',', '')
        price = Decimal(price_text)

        change_element = (div.find("span", class_="txt-negative svelte-1pws7a4")
                          or div.find("span", class_="txt-positive svelte-1pws7a4"))
        if not change_element:
            continue
        percent_change = change_element.text

        change = price / (1 + Decimal(percent_change.strip('%')) / 100) - price
        change = round(change, 2)
        if percent_change.startswith('-'):
            change_str = '-' + str(abs(change))
        else:
            change_str = '+' + str(change) if not str(change).startswith('+') else str(change)

        etf = SimpleQuote(symbol=symbol, name=name, price=price, change=change_str, percent_change=percent_change)
        etfs.append(etf)
    return etfs


async def scrape_similar_stocks(symbol: str) -> List[SimpleQuote]:
    url = 'https://finance.yahoo.com/quote/' + symbol
    with Session() as session:
        html = session.get(url, headers=headers).text
    soup = BeautifulSoup(html, 'lxml')

    similar_stocks = soup.find_all("div", class_="main-div svelte-15b2o7n", limit=6)
    stocks = parse_stocks(similar_stocks, symbol)

    # If similar_stocks is empty, try to scrape ETF data
    if not stocks:
        etf_stocks = soup.find_all("div", class_="ticker-container svelte-1pws7a4 enforceMaxWidth", limit=6)
        stocks = parse_etfs(etf_stocks)

    # If stocks is empty, the symbol is probably invalid
    if len(stocks) == 0:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")
    return stocks
