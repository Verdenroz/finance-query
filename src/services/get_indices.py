import asyncio
from decimal import Decimal

from aiohttp import ClientSession, TCPConnector
from bs4 import BeautifulSoup, SoupStrainer
from fastapi.responses import JSONResponse

from ..constants import headers
from ..schemas.index import Index
from ..utils import cache


async def fetch_and_parse(session, url, semaphore):
    async with semaphore, session.get(url, headers=headers) as response:
        html = await response.text()
        return await parse_html(html)


async def parse_html(html):
    parse_only = SoupStrainer('table', {'id': 'indice_table_1'})
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)  # Use 'lxml' as the parser
    table = soup.find('table', {'id': 'indice_table_1'})
    indices = []
    if table:
        rows = table.find_all('tr')
        for row in rows:
            cells = row.find_all('td')
            if len(cells) > 5:
                index_data = Index(
                    name=cells[1].text,
                    value=Decimal(cells[2].text.replace(',', '')),
                    change=cells[5].text,
                    percent_change=cells[6].text,
                )
                indices.append(index_data)
    return indices


@cache(expire=15, after_market_expire=3600)
async def scrape_indices():
    urls = ['https://www.investing.com/indices/americas-indices']
    semaphore = asyncio.Semaphore(25)  # Limit to 10 concurrent requests

    try:
        async with ClientSession(connector=TCPConnector(limit=25)) as session:
            tasks = [fetch_and_parse(session, url, semaphore) for url in urls]
            all_indices = await asyncio.gather(*tasks)
            return [index for indices in all_indices for index in indices]
    except Exception as e:
        return JSONResponse(status_code=500, content={"message": str(e)})
