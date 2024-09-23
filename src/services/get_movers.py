from decimal import Decimal

from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException

from ..redis import cache
from ..schemas.marketmover import MarketMover
from ..utils import fetch


@cache(expire=15, after_market_expire=3600)
async def scrape_actives():
    url = 'https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50'
    return await _scrape_movers(url)


@cache(expire=15, after_market_expire=3600)
async def scrape_gainers():
    url = 'https://finance.yahoo.com/markets/stocks/gainers/?start=0&count=50'
    return await _scrape_movers(url)


@cache(expire=15, after_market_expire=3600)
async def scrape_losers():
    url = 'https://finance.yahoo.com/markets/stocks/losers/?start=0&count=50'
    return await _scrape_movers(url)


async def _scrape_movers(url: str) -> list[MarketMover]:
    """
    Scrape the most active, gainers, or losers from Yahoo Finance
    :param url: the Yahoo Finance URL to scrape
    :return: a list of MarketMover objects

    :raises: HTTPException with status code 500 if an error occurs while scraping
    """
    html = await fetch(url)
    parse_only = SoupStrainer('tr', attrs={'class': 'row false  yf-42jv6g'})
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)
    movers = []

    for row in soup.find_all('tr'):
        cells = row.find_all('td', limit=4)

        symbol = cells[0].find('span', class_='symbol').text.strip()
        name = cells[0].find('span', class_='longName').text.strip()
        price = cells[1].find('fin-streamer', {'data-field': 'regularMarketPrice'}).text.strip()
        change = cells[2].find('fin-streamer', {'data-field': 'regularMarketChange'}).text.strip()
        percent_change = cells[3].find('fin-streamer', {'data-field': 'regularMarketChangePercent'}).text.strip()

        mover = MarketMover(
            symbol=symbol,
            name=name,
            price=Decimal(price.replace(',', '')),
            change=change,
            percent_change=percent_change
        )
        movers.append(mover)

    # If no movers are found, raise an HTTPException
    if not movers:
        raise HTTPException(status_code=500, detail='Error scraping data from Yahoo Finance')

    return movers
