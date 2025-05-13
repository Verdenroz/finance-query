import asyncio

from utils.cache import cache
from utils.dependencies import FinanceClient

from src.models import MarketIndex
from src.models.index import Index
from src.services.indices.fetchers import fetch_index
from src.services.quotes import get_adaptive_chunk_size


@cache(expire=15, market_closed_expire=180)
async def get_indices(finance_client: FinanceClient, indices: list[Index] = None) -> list[MarketIndex]:
    """
    Gets an aggregated performance of major world market indices or specific indices.

    :param finance_client: The Yahoo Finance client to use for API requests
    :param indices: A list of indices to fetch. If None, fetches all indices.

    :raises HTTPException: with status code 500 if an error occurs while scraping
    """
    # Get all indices by default
    if not indices:
        indices = list(Index)

    chunk_size = get_adaptive_chunk_size()
    chunks = [indices[i : i + chunk_size] for i in range(0, len(indices), chunk_size)]

    async def fetch_index_data(index: Index) -> MarketIndex:
        return await fetch_index(finance_client, index)

    all_indices = await asyncio.gather(*(asyncio.gather(*(fetch_index_data(index) for index in chunk)) for chunk in chunks))

    return [index for indices in all_indices for index in indices if index is not None]
