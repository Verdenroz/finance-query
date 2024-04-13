from constants import headers
from decimal import Decimal
from bs4 import BeautifulSoup
from fastapi.responses import JSONResponse
import requests
import models


async def scrape_actives():
    url = 'https://www.google.com/finance/markets/most-active'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        actives = []
        for active in soup.find_all('div', class_='SxcTic'):
            symbol = active.find('div', class_='COaKTb').text
            name = active.find('div', class_='ZvmM7').text
            price = Decimal(active.find('div', class_='YMlKec').text.replace('$', ''))
            change = active.find('div', class_='SEGxAb').text
            percentChange = active.find('div', class_='JwB6zf').text
            # Prepend '+' or '-' to percentChange based on whether change is positive or negative
            if change[0] != '-':
                percentChange = '+' + percentChange
            else:
                percentChange = '-' + percentChange

            mover_data = models.MarketMover(
                symbol=symbol,
                name=name,
                price=price,
                change=change,
                percentChange=percentChange,
            )

            actives.append(mover_data)
        return actives
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})
