import asyncio

from fastapi import HTTPException

from src.models import MarketSector, MarketSectorDetails, Sector
from src.services.sectors.fetchers import parse_sector, parse_sector_details
from src.services.sectors.utils import get_yahoo_sector
from src.utils.cache import cache
from src.utils.dependencies import FinanceClient, fetch

urls = {
    Sector.TECHNOLOGY: "https://finance.yahoo.com/sectors/technology/",
    Sector.HEALTHCARE: "https://finance.yahoo.com/sectors/healthcare/",
    Sector.FINANCIAL_SERVICES: "https://finance.yahoo.com/sectors/financial-services/",
    Sector.CONSUMER_CYCLICAL: "https://finance.yahoo.com/sectors/consumer-cyclical/",
    Sector.INDUSTRIALS: "https://finance.yahoo.com/sectors/industrials/",
    Sector.CONSUMER_DEFENSIVE: "https://finance.yahoo.com/sectors/consumer-defensive/",
    Sector.ENERGY: "https://finance.yahoo.com/sectors/energy/",
    Sector.REAL_ESTATE: "https://finance.yahoo.com/sectors/real-estate/",
    Sector.UTILITIES: "https://finance.yahoo.com/sectors/utilities/",
    Sector.BASIC_MATERIALS: "https://finance.yahoo.com/sectors/basic-materials/",
    Sector.COMMUNICATION: "https://finance.yahoo.com/sectors/communication-services/",
}


@cache(expire=300, market_closed_expire=3600)
async def get_sectors() -> list[MarketSector]:
    """
    Fetches and parses sector data for all sectors.
    :return: a list of MarketSector objects
    """
    tasks = []
    # Fetch sector data concurrently
    for sector, url in urls.items():
        tasks.append((sector.value, fetch(url=url)))
    responses = await asyncio.gather(*[task for _, task in tasks])

    sectors = []
    for (sector, _), html in zip(tasks, responses, strict=False):
        sector_data = await parse_sector(html, sector)
        sectors.append(sector_data)
    return sectors


@cache(expire=60, market_closed_expire=600)
async def get_sector_for_symbol(finance_client: FinanceClient, symbol: str) -> MarketSector:
    """
    Fetches and parses sector data for a specific stock symbol.
    :param finance_client: The Yahoo Finance client to use for API requests
    :param symbol: the stock symbol
    :return: a single MarketSector object

    :raises HTTPException: with code 404 if the sector for the symbol is not found
    """
    sector = await get_yahoo_sector(finance_client, symbol)
    if not sector:
        raise HTTPException(status_code=404, detail=f"Sector for {symbol} not found.")

    url = urls[Sector(sector)]
    html = await fetch(url=url)

    sector = await parse_sector(html, sector)
    return sector


@cache(expire=300, market_closed_expire=3600)
async def get_sector_details(sector: Sector) -> MarketSectorDetails:
    """
    Fetches and parses detailed sector data for a specific sector.
    :param sector: the sector to get detailed data for
    :return: a MarketSectorDetails object
    """
    url = urls[sector]
    html = await fetch(url=url)
    sector = await parse_sector_details(html, sector.value)

    return sector
