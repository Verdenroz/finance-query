from decimal import Decimal
from fastapi.responses import JSONResponse
from ..constants import headers
from ..schemas.news import News
from ..schemas.quote import Quote
from ..schemas.stock import Stock
from bs4 import BeautifulSoup
from typing import List
import requests
import re

# Compile a regular expression pattern that matches a number, optionally followed by a decimal point and more numbers
number_pattern = re.compile(r'\d+\.?\d*')


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

    return news_list


async def scrape_similar_stocks(soup: BeautifulSoup, symbol: str) -> List[Stock]:
    similar_stocks = soup.find_all("div", class_="main-div svelte-15b2o7n")
    stocks = []

    for div in similar_stocks:
        symbol_element = div.find("span")
        if not symbol_element:
            continue
        div_symbol = symbol_element.text
        if div_symbol == symbol:
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
    return stocks


async def scrape_quote(symbol: str):
    url = 'https://finance.yahoo.com/quote/' + symbol
    response = requests.get(url, headers=headers)
    if response.status_code == 404:
        return JSONResponse(status_code=404, content={"detail": "Symbol not found"})
    soup = BeautifulSoup(response.text, 'lxml')
    symbolName = soup.find('h1', class_='svelte-ufs8hf').text
    name = symbolName.split(' (')[0]
    symbol = symbolName.split(' (')[1].replace(')', '')
    priceNumbers = soup.find('div', class_='container svelte-mgkamr').text
    price = priceNumbers.split(' ')[0]
    change = priceNumbers.split(' ')[1]
    percent_change = priceNumbers.split(' ')[2].replace('(', '').strip('(').strip(')')

    # Find all list items
    list_items = soup.find_all("li", class_="svelte-tx3nkj")

    data = {}

    for item in list_items:
        label = item.find("span", class_="label").text.strip()
        value = item.find("span", class_="value").text.strip()
        data[label] = value

    open_price = Decimal(data.get("Open"))
    market_cap = data.get("Market Cap (intraday)")
    beta = Decimal(data.get("Beta (5Y Monthly)"))
    pe = Decimal(data.get("PE Ratio (TTM)"))
    eps = Decimal(data.get("EPS (TTM)"))
    earnings_date = data.get("Earnings Date")
    ex_dividend = data.get("Ex-Dividend Date")

    # Extract high and low from "Day's Range"
    days_range = data.get("Day's Range")
    if not days_range:
        return JSONResponse(status_code=500, content={"detail": "Error parsing days range"})
    low, high = [Decimal(x) for x in days_range.split(' - ')]

    # Extract year_high and year_low from "52 Week Range"
    fifty_two_week_range = data.get("52 Week Range")
    year_low, year_high = [Decimal(x) for x in fifty_two_week_range.split(' - ')] if fifty_two_week_range else (
        None, None)

    # Convert volume and avg_volume to integers
    volume = int(data.get("Volume").replace(',', '')) if data.get("Volume") else None
    avg_volume = int(data.get("Avg. Volume").replace(',', '')) if data.get("Avg. Volume") else None

    about = soup.find('p', class_='svelte-1xu2f9r').text
    sector, industry = await extract_sector_and_industry(soup)

    news = await scrape_news_for_quote(symbol)

    stocks = await scrape_similar_stocks(soup, symbol)

    # print(f"Creating Quote with:\n"
    #       f"Symbol: {symbol}\n"
    #       f"Name: {name}\n"
    #       f"Price: {price}\n"
    #       f"Change: {change}\n"
    #       f"Percent Change: {percent_change}\n"
    #       f"Open: {open}\n"
    #       f"High: {high}\n"
    #       f"Low: {low}\n"
    #       f"Year High: {year_high}\n"
    #       f"Year Low: {year_low}\n"
    #       f"Volume: {volume}\n"
    #       f"Avg Volume: {avg_volume}\n"
    #       f"Market Cap: {market_cap}\n"
    #       f"Beta: {beta}\n"
    #       f"PE: {pe}\n"
    #       f"EPS: {eps}\n"
    #       f"Earnings Date: {earnings_date}\n"
    #       f"Ex Dividend: {ex_dividend}\n"
    #       f"About: {about}\n"
    #       f"Sector: {sector}\n"
    #       f"Industry: {industry}\n"
    #       f"News: {news}\n"
    #       f"Similar Stocks: {stocks}")

    quote = Quote(
        symbol=symbol,
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

    return quote
