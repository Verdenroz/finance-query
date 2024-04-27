import asyncio
from decimal import Decimal
from typing import List

import yfinance
from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from httpx import AsyncClient

from src.schemas import Quote, Stock
from ..constants import headers
from ..utils import is_market_open


async def fetch(url: str, client: AsyncClient):
    response = await client.get(url, headers=headers)
    return response.text


async def get_post_price(symbol: str, client: AsyncClient):
    url = 'https://finance.yahoo.com/quote/' + symbol
    html = await fetch(url, client)

    parse_only = SoupStrainer(['fin-streamer', 'data-testid', 'qsp-post-price'])
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    # After hours price
    post_price_element = soup.find("fin-streamer", {"data-testid": "qsp-post-price"})
    if not post_price_element:
        post_price = None
    else:
        post_price = round(Decimal(post_price_element["data-value"]), 2)
    return post_price


async def scrape_quote(symbol: str, client: AsyncClient):
    stock = yfinance.Ticker(symbol)
    info = stock.get_info()
    # Check if the symbol is valid
    try:
        name = info['longName']
    except Exception:
        raise HTTPException(400, detail="Invalid symbol")

    hist = stock.history(period="2d")
    calendar = stock.get_calendar()

    price = round(Decimal(info.get('currentPrice', hist['Close'].iloc[-1])), 2)
    price_change = round(Decimal(hist['Close'].diff().iloc[-1]), 2)
    percent_change = round(Decimal(hist['Close'].pct_change().iloc[-1] * 100), 2)
    open_price = round(Decimal(info['regularMarketOpen']), 2)
    high = round(Decimal(info['regularMarketDayHigh']), 2)
    low = round(Decimal(info['regularMarketDayLow']), 2)
    year_high = round(Decimal(info['fiftyTwoWeekHigh']), 2)
    year_low = round(Decimal(info['fiftyTwoWeekLow']), 2)
    volume = info['regularMarketVolume']
    avg_volume = info['averageDailyVolume10Day']
    market_cap = info.get('marketCap', None)
    beta = info.get('beta', None)
    pe = info.get('trailingPE', None)
    eps = info.get('trailingEps', None)
    dividends = info.get('dividendRate', None)
    ex_dividend = calendar.get('Ex-Dividend Date')
    if ex_dividend is not None:
        ex_dividend = ex_dividend.strftime('%m-%d-%Y')

    earnings_date = calendar.get('Earnings Date')
    if earnings_date is not None:
        earnings_date = earnings_date[0].strftime('%m-%d-%Y')
    sector = info.get('sector', None)
    industry = info.get('industry', None)
    about = info.get('longBusinessSummary', None)

    post_price = None
    if not is_market_open():
        post_price = await get_post_price(symbol, client)

    quote = Quote(
        symbol=symbol.upper(),
        name=name,
        price=price,
        after_hours_price=post_price,
        change=price_change,
        percent_change=percent_change,
        open=open_price,
        high=high,
        low=low,
        year_high=year_high,
        year_low=year_low,
        volume=volume,
        avg_volume=avg_volume,
        market_cap=market_cap,
        beta=beta,
        pe=pe,
        eps=eps,
        dividend=dividends,
        ex_dividend=ex_dividend,
        earnings_date=earnings_date,
        sector=sector,
        industry=industry,
        about=about,
    )

    return quote.dict()


async def scrape_quotes(symbols: List[str]):
    async with AsyncClient(http2=True) as client:
        tasks = [scrape_quote(symbol, client) for symbol in symbols]
        quotes = await asyncio.gather(*tasks)
    return quotes


async def scrape_simple_quote(symbol: str):
    stock = yfinance.Ticker(symbol)
    info = stock.get_info()
    hist = stock.history(period="2d")
    print(f"{symbol}: {info.keys()}")
    name = info['longName']
    price = round(Decimal(info.get('currentPrice', hist['Close'].iloc[-1])), 2)
    if price == 0:
        price = round(Decimal(info.get('regularMarketPreviousClose', 0)), 2)
    price_change = round(Decimal(hist['Close'].diff().iloc[-1]), 2)
    percent_change = round(Decimal(hist['Close'].pct_change().iloc[-1] * 100), 2)

    stock = Stock(
        symbol=symbol.upper(),
        name=name,
        price=price,
        change=price_change,
        percent_change=percent_change
    )

    return stock.dict()


async def scrape_simple_quotes(symbols: List[str]):
    tasks = [scrape_simple_quote(symbol) for symbol in symbols]
    quotes = await asyncio.gather(*tasks)
    return quotes
