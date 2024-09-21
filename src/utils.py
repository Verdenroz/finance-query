from datetime import datetime
from typing import Optional

import pytz
from aiohttp import ClientSession

from src.constants import headers
from src.proxy import proxy, proxy_auth


def is_market_open() -> bool:
    """
    Check if the market is open
    :return: True if the market is open, False otherwise
    """
    now = datetime.now(pytz.timezone('US/Eastern'))
    open_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=9, minute=30, second=0)
    close_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=16, minute=0, second=0)
    # Check if current time is within market hours and it's a weekday
    return open_time <= now <= close_time and 0 <= now.weekday() < 5


async def fetch(url: str, session: ClientSession) -> str:
    """
    Fetch the data from the given URL
    :param url: the URL to fetch data from
    :param session: the aiohttp ClientSession
    :return: the html content of the page
    """
    async with session.get(url, headers=headers, proxy=proxy, proxy_auth=proxy_auth) as response:
        return await response.text()


async def get_logo(url: str, session: ClientSession) -> Optional[str]:
    """
    Get the logo of the company from the given URL
    :param url: the URL of the company
    :param session: the aiohttp ClientSession
    :return: the URL of the logo as a string
    """
    async with session.get(f"https://logo.clearbit.com/{url}") as response:
        if response.status == 200:
            return str(response.url)
        else:
            return None
