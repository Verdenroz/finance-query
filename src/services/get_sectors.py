import asyncio

from bs4 import BeautifulSoup, SoupStrainer
from httpx import AsyncClient

from src.constants import headers
from src.schemas import Sector
from src.utils import cache


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
        day_return=data[0].strip(),
        ytd_return=data[1].strip(),
        year_return=data[2].strip(),
        three_year_return=data[3].strip(),
        five_year_return=data[4].strip()
    )


@cache(expire=300, after_market_expire=3600)
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

    async with AsyncClient(http2=True, max_redirects=5) as client:
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
