import asyncio
import re
from aiohttp import ClientSession, TCPConnector
from bs4 import BeautifulSoup, SoupStrainer
from decimal import Decimal
from fastapi.responses import JSONResponse
from ..constants import headers
from ..schemas.marketmover import MarketMover

# Compile a regular expression pattern that matches a number, optionally followed by a decimal point and more numbers
number_pattern = re.compile(r'\d+\.?\d*')


async def create_market_mover(mover):
    symbol = mover.find('div', class_='COaKTb').text
    name = mover.find('div', class_='ZvmM7').text

    price_text = mover.find('div', class_='YMlKec').text
    price_match = number_pattern.search(price_text)
    price = Decimal(price_match.group()) if price_match else None

    change_text = mover.find('div', class_='SEGxAb').text
    change_match = number_pattern.search(change_text)
    change = change_match.group() if change_match else None

    percent_change_text = mover.find('div', class_='JwB6zf').text
    percent_change_match = number_pattern.search(percent_change_text)
    percent_change = percent_change_match.group() if percent_change_match else None

    # Prepend '+' or '-' to percentChange based on whether change is positive or negative
    if change and change[0] != '-':
        percent_change = '+' + percent_change
    else:
        percent_change = '-' + percent_change

    mover_data = MarketMover(
        symbol=symbol,
        name=name,
        price=price,
        change=change,
        percent_change=percent_change,
    )
    return mover_data


async def fetch_and_parse_movers(session, url, semaphore):
    async with semaphore, session.get(url, headers=headers) as response:
        html = await response.text()
        parse_only = SoupStrainer('ul', class_='sbnBtf')
        soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)
        movers = []
        for mover in soup.find_all('div', class_='SxcTic'):
            mover_data = await create_market_mover(mover)
            movers.append(mover_data)
        return movers


async def scrape_movers(url):
    semaphore = asyncio.Semaphore(25)  # Limit to 25 concurrent requests
    try:
        async with ClientSession(connector=TCPConnector(limit=25)) as session:
            movers = await fetch_and_parse_movers(session, url, semaphore)
            return movers
    except Exception as e:
        return JSONResponse(status_code=500, content={"message": str(e)})


async def scrape_actives():
    url = 'https://www.google.com/finance/markets/most-active'
    return await scrape_movers(url)


async def scrape_gainers():
    url = 'https://www.google.com/finance/markets/gainers'
    return await scrape_movers(url)


async def scrape_losers():
    url = 'https://www.google.com/finance/markets/losers'
    return await scrape_movers(url)
