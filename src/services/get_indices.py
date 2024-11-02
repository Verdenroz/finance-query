from decimal import Decimal

from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException

from ..redis import cache
from ..schemas.index import Index
from ..utils import fetch


@cache(expire=15, market_closed_expire=3600)
async def scrape_indices() -> list[Index]:
    """
    Scrape the Americas indices from investing.com
    :return: a list of Index objects

    :raises: HTTPException with status code 500 if an error occurs while scraping
    """
    url = 'https://www.investing.com/indices/americas-indices'

    try:
        html = await fetch(url)
        return await get_indices(html)
    except Exception as e:
        raise HTTPException(status_code=500, detail={str(e)})


async def get_indices(html) -> list[Index]:
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