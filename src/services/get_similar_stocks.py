from decimal import Decimal

from bs4 import BeautifulSoup
from fastapi import HTTPException
from requests import Session
from typing_extensions import List

from src.constants import headers
from src.schemas import SimpleQuote
from src.utils import cache


def parse_stocks(stocks_divs, symbol):
    stocks = []
    for div in stocks_divs:
        symbol_element = div.find("span")
        if not symbol_element:
            continue
        div_symbol = symbol_element.text
        if div_symbol.lower() == symbol.lower():
            continue

        name_element = div.find("div", class_="longName yf-15b2o7n")
        if not name_element:
            continue
        name = name_element.text

        price_element = div.find("span", class_="price yf-15b2o7n")
        if not price_element:
            continue
        price_text = price_element.text.replace(',', '')
        price = Decimal(price_text)

        change_element = (div.find("span", class_="positive yf-15b2o7n") or
                          div.find("span", class_="negative yf-15b2o7n"))
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

    return stocks


def parse_etfs(etf_divs):
    etfs = []
    for div in etf_divs:
        symbol_element = div.find("span", class_="symbol yf-ravs5v")
        if not symbol_element:
            continue
        symbol = symbol_element.text

        name_element = div.find("span", class_="tw-text-sm yf-ravs5v longName")
        if not name_element:
            continue
        name = name_element.text

        price_element = div.find("strong")
        if not price_element:
            continue
        price_text = price_element.text.replace(',', '')
        price = Decimal(price_text)

        change_element = (div.find("span", class_="txt-negative yf-1pws7a4")
                          or div.find("span", class_="txt-positive yf-1pws7a4"))
        if not change_element:
            continue
        percent_change = change_element.text

        change = price / (1 + Decimal(percent_change.strip('%')) / 100) - price
        change = round(change, 2)
        if percent_change.startswith('-'):
            change_str = '-' + str(abs(change))
        else:
            change_str = '+' + str(abs(change))

        etf = SimpleQuote(symbol=symbol, name=name, price=price, change=change_str, percent_change=percent_change)
        etfs.append(etf)
    return etfs


@cache(expire=15, after_market_expire=600)
async def scrape_similar_stocks(symbol: str, limit: int) -> List[SimpleQuote]:
    url = 'https://finance.yahoo.com/quote/' + symbol
    with Session() as session:
        html = session.get(url, headers=headers).text
    soup = BeautifulSoup(html, 'lxml')

    similar_stocks = soup.find_all("div", class_="main-div yf-15b2o7n", limit=limit)
    stocks = parse_stocks(similar_stocks, symbol)

    # If similar_stocks is empty, try to scrape ETF data
    if not stocks:
        etf_stocks = soup.find_all("div", class_="ticker-container yf-1pws7a4 enforceMaxWidth", limit=limit)
        stocks = parse_etfs(etf_stocks)

    # If stocks is empty, the symbol is probably invalid
    if not stocks:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")
    return stocks
