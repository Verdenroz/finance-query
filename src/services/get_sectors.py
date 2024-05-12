import asyncio

from bs4 import BeautifulSoup, SoupStrainer
from httpx import AsyncClient

from src.constants import headers
from src.schemas import Sector


async def parse_sector(html: str, sector: str) -> Sector:
    soup = BeautifulSoup(html, 'lxml', parse_only=SoupStrainer('section'))
    returns = soup.find_all('section', 'card small svelte-1v51y3z bdr sticky')
    data = []
    for changes in returns:
        perf_div = changes.find('div', class_=['perf positive svelte-12wncuy', 'perf negative svelte-12wncuy'])
        sign = '+' if 'positive' in perf_div['class'] else '-'
        data.append(sign + perf_div.text)
    return Sector(
        sector=sector,
        day_return=data[0],
        ytd_return=data[1],
        year_return=data[2],
        three_year_return=data[3],
        five_year_return=data[4]
    )


async def get_sectors():
    urls = {
        'Technology': 'https://finance.yahoo.com/sectors/technology/',
        'Healthcare': 'https://finance.yahoo.com/sectors/healthcare/',
        'Financial Services': 'https://finance.yahoo.com/sectors/financial-services/',
        'Consumer Cyclical': 'https://finance.yahoo.com/sectors/consumer-cyclical/',
        'Industrials': 'https://finance.yahoo.com/sectors/industrials/',
        'Consumer Defensive': 'https://finance.yahoo.com/sectors/consumer-defensive/',
        'Energy': 'https://finance.yahoo.com/sectors/energy/',
        'Real Estate': 'https://finance.yahoo.com/sectors/real-estate/',
        'Utilities': 'https://finance.yahoo.com/sectors/utilities/',
        'Basic Materials': 'https://finance.yahoo.com/sectors/basic-materials/',
        'Communication Services': 'https://finance.yahoo.com/sectors/communication-services/'
    }

    async with AsyncClient(http2=True) as client:
        tasks = []
        for sector, url in urls.items():
            tasks.append((sector, client.get(url, headers=headers)))
        responses = await asyncio.gather(*[task for _, task in tasks])

    sectors = []
    for (sector, _), response in zip(tasks, responses):
        html = response.text
        sector_data = await parse_sector(html, sector)
        sectors.append(sector_data)

    return sectors
