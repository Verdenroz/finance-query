from constants import headers
from decimal import Decimal
from bs4 import BeautifulSoup
from fastapi.responses import JSONResponse
import requests
import models


async def scrape_gainers():
    url = 'https://www.google.com/finance/markets/gainers'
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        soup = BeautifulSoup(response.content, 'html.parser')
        gainers = []
        for gainer in soup.find_all('div', class_='SxcTic'):
            symbol = gainer.find('div', class_='COaKTb').text
            name = gainer.find('div', class_='ZvmM7').text
            price = Decimal(gainer.find('div', class_='YMlKec').text.replace('$', ''))
            change = gainer.find('div', class_='SEGxAb').text
            percentChange = gainer.find('div', class_='JwB6zf').text
            percentChange = '+' + percentChange

            mover_data = models.MarketMover(
                symbol=symbol,
                name=name,
                price=price,
                change=change,
                percentChange=percentChange,
            )
            gainers.append(mover_data)
        return gainers
    else:
        return JSONResponse(status_code=500, content={"message": "Failed to fetch data from the URL"})
