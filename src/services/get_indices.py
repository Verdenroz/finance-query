import asyncio
from decimal import Decimal

from aiohttp import ClientSession, TCPConnector
from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException

from ..constants import headers
from ..proxy import proxy, proxy_auth
from ..redis import cache
from ..schemas.index import Index


@cache(expire=15, after_market_expire=3600)
async def scrape_indices() -> list[Index]:
    """
    Scrape the Americas indices from investing.com
    :return: a list of Index objects

    :raises: HTTPException with status code 500 if an error occurs while scraping
    """
    urls = ['https://www.investing.com/indices/americas-indices']
    semaphore = asyncio.Semaphore(25)  # Limit to 25 concurrent requests

    try:
        async with ClientSession(connector=TCPConnector(limit=25)) as session:
            tasks = [_fetch_and_parse(url, session, semaphore) for url in urls]
            all_indices = await asyncio.gather(*tasks)
            return [index for indices in all_indices for index in indices]
    except Exception as e:
        raise HTTPException(status_code=500, detail={str(e)})


async def _fetch_and_parse(url: str, session: ClientSession, semaphore: asyncio.Semaphore):
    """
    Custom fetch and parse function to limit the number of concurrent requests
    :param url: the URL to fetch data from
    :param session: the aiohttp ClientSession
    :param semaphore: the semaphore to limit the number of concurrent requests
    :return: 
    """
    async with semaphore, session.get(url, headers=headers, proxy=proxy, proxy_auth=proxy_auth) as response:
        html = await response.text()
        return await parse_html(html)


async def parse_html(html) -> list[Index]:
    """
    Parse the HTML content and return a list of Index objects
    :param html: the HTML content
    :return: a list of Index objects
    """
    parse_only = SoupStrainer('table', {'id': 'indice_table_1'})
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)
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
