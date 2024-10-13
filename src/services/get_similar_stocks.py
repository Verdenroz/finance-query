from decimal import Decimal

from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from typing_extensions import List

from src.redis import cache
from src.schemas import SimpleQuote
from src.utils import fetch


@cache(expire=15, after_market_expire=600)
async def scrape_similar_stocks(symbol: str, limit: int = 10) -> List[SimpleQuote]:
    url = 'https://finance.yahoo.com/quote/' + symbol
    html = await fetch(url)

    parse_only = SoupStrainer(['div'], attrs={'class': ['main-div yf-15b2o7n', 'carousel-top  yf-1pws7a4']})
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    similar_stocks = soup.find_all("div", class_="main-div yf-15b2o7n", limit=limit)
    similar = await _parse_stocks(similar_stocks, symbol)

    # If similar_stocks is empty, try to scrape ETF data
    if not similar:
        etf_stocks = soup.find_all("div", class_="ticker-container yf-1pws7a4 enforceMaxWidth", limit=limit)
        similar = await _parse_etfs(etf_stocks)

    # If similar is still empty, the symbol is probably invalid
    if not similar:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

    return similar


async def _parse_stocks(stocks_divs, symbol) -> List[SimpleQuote]:
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

        stock = SimpleQuote(
            symbol=div_symbol,
            name=name,
            price=price,
            change=change_str,
            percent_change=percent_change,
        )
        stocks.append(stock)

    return stocks


async def _parse_etfs(etf_divs):
    etfs = []
    for div in etf_divs:
        symbol_element = div.find("span", class_="symbol yf-138ga19")
        if not symbol_element:
            continue
        symbol = symbol_element.text

        name_element = div.find("span", class_="tw-text-sm yf-138ga19 longName")
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

        etf = SimpleQuote(
            symbol=symbol,
            name=name,
            price=price,
            change=change_str,
            percent_change=percent_change,
        )
        etfs.append(etf)

    return etfs
