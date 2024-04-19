import asyncio
from concurrent.futures import ThreadPoolExecutor

import requests
from decimal import Decimal
from typing import List

from aiohttp import ClientSession
from bs4 import BeautifulSoup, SoupStrainer
from fastapi.responses import JSONResponse

from ..constants import headers
from ..schemas.news import News
from ..schemas.quote import Quote
from ..schemas.stock import Stock


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


async def scrape_news_for_quote(symbol: str):
    link = 'https://stockanalysis.com/stocks/' + symbol
    response = requests.get(link, headers=headers)
    if response.status_code == 404:
        return JSONResponse(status_code=404, content={"detail": "Symbol not found"})
    soup = BeautifulSoup(response.text, 'lxml')

    news = soup.find_all('div', class_='gap-4 border-gray-300 bg-white p-4 shadow last:pb-1 last:shadow-none '
                                       'dark:border-dark-600 dark:bg-dark-800 sm:border-b sm:px-0 sm:shadow-none '
                                       'sm:last:border-b-0 lg:gap-5 sm:grid sm:grid-cols-news sm:py-6')
    news_list = []
    for new in news:
        img_element = new.find("img", class_="rounded")
        img = img_element["src"]
        if not img:
            continue

        title_element = new.find("h3",
                                 class_="mb-2 mt-3 text-xl font-bold leading-snug sm:order-2 sm:mt-0 sm:leading-tight")
        title_link_element = title_element.find("a")
        title = title_link_element.text
        link = title_link_element["href"]

        source_date_element = new.find("div", class_="mt-1 text-sm text-faded sm:order-1 sm:mt-0")
        source_date = source_date_element.text
        time = source_date.split(" - ")[0]
        source = source_date.split(" - ")[1]

        news_item = News(title=title, link=link, source=source, img=img, time=time)
        news_list.append(news_item)
        if len(news_list) == 5:
            break

    return news_list


async def scrape_similar_stocks(soup: BeautifulSoup, symbol: str) -> List[Stock]:
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


async def scrape_quote(symbol: str):
    url = 'https://finance.yahoo.com/quote/' + symbol
    response = requests.get(url, headers=headers)
    if response.status_code == 404:
        return JSONResponse(status_code=404, content={"detail": "Symbol not found"})

    parse_only = SoupStrainer(['h1', 'div'])
    soup = BeautifulSoup(response.text, 'lxml', parse_only=parse_only)

    symbol_name_element = soup.select_one('h1.svelte-ufs8hf')
    price_numbers_element = soup.select_one('div.container.svelte-mgkamr')

    name = symbol_name_element.text.split('(')[0].strip()
    price_numbers = price_numbers_element.text
    price = price_numbers.split(' ')[0]
    change = price_numbers.split(' ')[1]
    percent_change = price_numbers.split(' ')[2].replace('(', '').strip('(').strip(')')

    list_items = soup.select('li.svelte-tx3nkj')

    data = {}

    for item in list_items:
        label = item.find("span", class_="label").text.strip()
        value = item.find("span", class_="value").text.strip()
        data[label] = value

    open_price = Decimal(data.get("Open"))
    market_cap = data.get("Market Cap (intraday)")
    beta = Decimal(data.get("Beta (5Y Monthly)")) if data.get("Beta (5Y Monthly)").replace('.', '', 1).isdigit() else None
    pe = Decimal(data.get("PE Ratio (TTM)"))
    eps = Decimal(data.get("EPS (TTM)"))
    earnings_date = data.get("Earnings Date")
    ex_dividend = data.get("Ex-Dividend Date")

    days_range = data.get("Day's Range")
    if not days_range:
        return JSONResponse(status_code=500, content={"detail": "Error parsing days range"})
    low, high = [Decimal(x) for x in days_range.split(' - ')]

    fifty_two_week_range = data.get("52 Week Range")
    year_low, year_high = [Decimal(x) for x in fifty_two_week_range.split(' - ')] if fifty_two_week_range else (
        None, None)

    volume = int(data.get("Volume").replace(',', '')) if data.get("Volume") else None
    avg_volume = int(data.get("Avg. Volume").replace(',', '')) if data.get("Avg. Volume") else None

    about = soup.find('p', class_='svelte-1xu2f9r').text
    sector, industry = await extract_sector_and_industry(soup)

    news = await scrape_news_for_quote(symbol)

    stocks = await scrape_similar_stocks(soup, symbol)

    return Quote(
        symbol=symbol.upper(),
        name=name,
        price=Decimal(price),
        change=change,
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
        earnings_date=earnings_date,
        ex_dividend_date=ex_dividend,
        sector=sector,
        industry=industry,
        about=about,
        news=news,
        similar_stocks=stocks
    )


async def scrape_quotes(symbols: List[str]):
    quotes = await asyncio.gather(*(scrape_quote(symbol) for symbol in symbols))
    return [quote for quote in quotes if not isinstance(quote, Exception)]
