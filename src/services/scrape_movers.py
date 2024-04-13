from src.constants import headers
from decimal import Decimal
from bs4 import BeautifulSoup
from fastapi.responses import JSONResponse
import requests
from src import schemas


def create_market_mover(mover):
    symbol = mover.find('div', class_='COaKTb').text
    name = mover.find('div', class_='ZvmM7').text
    price = Decimal(mover.find('div', class_='YMlKec').text.replace('$', ''))
    change = mover.find('div', class_='SEGxAb').text
    percent_change = mover.find('div', class_='JwB6zf').text
    # Prepend '+' or '-' to percentChange based on whether change is positive or negative
    if change[0] != '-':
        percent_change = '+' + percent_change
    else:
        percent_change = '-' + percent_change

    mover_data = schemas.MarketMover(
        symbol=symbol,
        name=name,
        price=price,
        change=change,
        percent_change=percent_change,
    )
    return mover_data


async def scrape_actives():
    url = 'https://www.google.com/finance/markets/most-active'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        actives = []
        for active in soup.find_all('div', class_='SxcTic'):
            mover_data = create_market_mover(active)
            actives.append(mover_data)
        return actives
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})


async def scrape_gainers():
    url = 'https://www.google.com/finance/markets/gainers'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        gainers = []
        for gainer in soup.find_all('div', class_='SxcTic'):
            mover_data = create_market_mover(gainer)
            gainers.append(mover_data)
        return gainers
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})


async def scrape_losers():
    url = 'https://www.google.com/finance/markets/losers'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        losers = []
        for loser in soup.find_all('div', class_='SxcTic'):
            mover_data = create_market_mover(loser)
            losers.append(mover_data)
        return losers
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})
