import asyncio
from decimal import Decimal
from typing import List

import requests
from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from httpx import AsyncClient

from src.schemas import Quote, SimpleQuote
from ..constants import headers
from ..utils import cache


async def fetch(url: str, client: AsyncClient):
    response = await client.get(url, headers=headers)
    return response.text


async def extract_sector_and_industry(soup: BeautifulSoup):
    info_sections = soup.find_all("div", class_="infoSection svelte-1xu2f9r")

    sector = None
    industry = None

    for section in info_sections:
        h3_text = section.find("h3").text
        a_element = section.find("a")
        a_text = a_element.text if a_element else None
        if h3_text == "Sector":
            sector = a_text.strip()
        elif h3_text == "Industry":
            industry = a_text.strip()

    return sector, industry


async def get_logo(url: str):
    response = requests.get(f"https://logo.clearbit.com/{url}")
    if response.status_code == 200:
        return response.url
    else:
        return None


def get_decimal(data, key):
    value = data.get(key)
    dec_value = Decimal(value) if value and value.replace('.', '', 1).isdigit() else None
    if dec_value is None:
        return None
    return dec_value


async def scrape_quote(symbol: str, client: AsyncClient):
    url = 'https://finance.yahoo.com/quote/' + symbol + "/"
    html = await fetch(url, client)

    parse_only = SoupStrainer(['h1', 'div', 'card'])
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    symbol_name_element = soup.select_one('h1.svelte-3a2v0c')
    if not symbol_name_element:
        raise HTTPException(status_code=404, detail="Symbol not found")

    name = symbol_name_element.text.split('(')[0].strip()

    regular_price = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price"})["data-value"]), 2)
    regular_change_value = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change"})["data-value"]),
                                 2)
    regular_percent_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change-percent"})["data-value"]), 2)

    # Add + or - sign and % for percent_change
    regular_change = '+' + str(regular_change_value) if regular_change_value > 0 else str(regular_change_value)
    regular_percent_change = '+' + str(
        regular_percent_change_value) + '%' if regular_percent_change_value > 0 else str(
        regular_percent_change_value) + '%'

    # After hours price
    post_price_element = soup.find("fin-streamer", {"data-testid": "qsp-post-price"})
    if not post_price_element:
        post_price = None
    else:
        post_price = round(Decimal(post_price_element["data-value"]), 2)

    list_items = soup.select('li.svelte-tx3nkj')

    data = {}

    for item in list_items:
        label = item.find("span", class_="label").text.strip()
        value = item.find("span", class_="value").text.strip()
        data[label] = value

    open_price = Decimal(data.get("Open").replace(',', ''))
    market_cap = data.get("Market Cap (intraday)")
    beta = get_decimal(data, "Beta (5Y Monthly)")
    pe = get_decimal(data, "PE Ratio (TTM)")
    eps = get_decimal(data, "EPS (TTM)")
    earnings_date = data.get("Earnings Date")
    forward_dividend_yield = data.get("Forward Dividend & Yield")
    dividend, yield_percent = (None, data.get("Yield")) if not forward_dividend_yield \
        else (None, None) if not any(char.isdigit() for char in forward_dividend_yield) \
        else forward_dividend_yield.replace("(", "").replace(")", "").split()
    ex_dividend = None if data.get("Ex-Dividend Date") == "--" else data.get("Ex-Dividend Date")
    net_assets = data.get("Net Assets")
    nav = data.get("NAV")
    expense_ratio = data.get("Expense Ratio (net)")

    # Day's range
    days_range = data.get("Day's Range")
    if not days_range:
        raise HTTPException(status_code=500, detail="Error parsing days range")
    low, high = [Decimal(x.replace(',', '')) for x in days_range.split(' - ')]

    # 52-week range
    fifty_two_week_range = data.get("52 Week Range")
    year_low, year_high = [Decimal(x.replace(',', '')) for x in fifty_two_week_range.split(' - ')] \
        if fifty_two_week_range else (None, None)

    # Volume
    volume = int(data.get("Volume").replace(',', '')) if data.get("Volume") else None
    avg_volume = int(data.get("Avg. Volume").replace(',', '')) if data.get("Avg. Volume") else None

    # About the company
    about = soup.find('p', class_='svelte-1xu2f9r').text
    # Logo
    logo_element = soup.find('a', class_='subtle-link fin-size-medium svelte-wdkn18')
    logo_url = logo_element['href'] if logo_element else None

    # Scrape sector, industry, and logo concurrently
    sector_and_industry_future = asyncio.create_task(extract_sector_and_industry(soup))
    logo_future = asyncio.create_task(get_logo(logo_url))

    (sector, industry), logo = await asyncio.gather(sector_and_industry_future, logo_future)

    # Scrape performance:
    returns = soup.find_all('section', 'card small svelte-1v51y3z bdr sticky')
    data = []
    for changes in returns:
        perf_div = changes.find('div', class_=['perf positive svelte-12wncuy', 'perf negative svelte-12wncuy'])
        if perf_div:
            sign = '+' if 'positive' in perf_div['class'] else '-'
            data.append(sign + perf_div.text.strip())

    ytd_return = data[0] if len(data) > 0 else None
    year_return = data[1] if len(data) > 1 else None
    three_year_return = data[2] if len(data) > 2 else None
    five_year_return = data[3] if len(data) > 3 else None

    return Quote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        after_hours_price=post_price,
        change=regular_change,
        percent_change=regular_percent_change,
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
        earnings_date=earnings_date,
        dividend=dividend,
        dividend_yield=yield_percent,
        ex_dividend=ex_dividend,
        net_assets=net_assets,
        nav=nav,
        expense_ratio=expense_ratio,
        sector=sector,
        industry=industry,
        about=about,
        ytd_return=ytd_return,
        year_return=year_return,
        three_year_return=three_year_return,
        five_year_return=five_year_return,
        logo=logo
    )


@cache(30, after_market_expire=600)
async def scrape_quotes(symbols: List[str]):
    async with AsyncClient(http2=True, max_redirects=5) as client:
        tasks = [scrape_quote(symbol, client) for symbol in symbols]
        quotes = await asyncio.gather(*tasks)
    return quotes


async def scrape_simple_quote(symbol: str, client: AsyncClient):
    url = 'https://finance.yahoo.com/quote/' + symbol + "/"
    html = await fetch(url, client)

    parse_only = SoupStrainer(['h1', 'fin-streamer'])
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    symbol_name_element = soup.select_one('h1.svelte-3a2v0c')
    if not symbol_name_element:
        raise HTTPException(status_code=404, detail="Symbol not found")

    name = symbol_name_element.text.split('(')[0].strip()

    regular_price = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price"})["data-value"]), 2)
    regular_change_value = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change"})["data-value"]),
                                 2)
    regular_percent_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change-percent"})["data-value"]), 2)

    # Add + or - sign and % for percent_change
    regular_change = '+' + str(regular_change_value) if regular_change_value >= 0 else str(regular_change_value)
    regular_percent_change = '+' + str(
        regular_percent_change_value) + '%' if regular_percent_change_value >= 0 else str(
        regular_percent_change_value) + '%'

    return SimpleQuote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        change=regular_change,
        percent_change=regular_percent_change
    )


@cache(30, after_market_expire=600)
async def scrape_simple_quotes(symbols: List[str]):
    async with AsyncClient(http2=True, max_redirects=5) as client:
        quotes = await asyncio.gather(*(scrape_simple_quote(symbol, client) for symbol in symbols))
        return [quote for quote in quotes if not isinstance(quote, Exception)]
