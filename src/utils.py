import os
from datetime import datetime
from typing import Optional

import pytz
from aiohttp import ClientSession
from dotenv import load_dotenv

from src.proxy import proxy, proxy_auth
from src.session_manager import get_global_session

load_dotenv()


def is_market_open() -> bool:
    now = datetime.now(pytz.timezone('US/Eastern'))
    open_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=9, minute=30, second=0)
    close_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=16, minute=0, second=0)
    # Check if current time is within market hours and it's a weekday
    return open_time <= now <= close_time and 0 <= now.weekday() < 5


async def fetch(url: str,session: Optional[ClientSession] = None, use_proxy: bool = os.getenv('USE_PROXY', 'False') == 'True') -> str:
    session = session or await get_global_session()
    if use_proxy:
        if proxy is None or proxy_auth is None:
            raise ValueError("Proxy URL/Auth/Token not set in .env")

        async with session.get(url, proxy=proxy, proxy_auth=proxy_auth) as response:
            return await response.text()

    else:
        async with session.get(url) as response:
            return await response.text()


async def get_logo(url: str, session: Optional[ClientSession] = None) -> Optional[str]:
    session = session or await get_global_session()
    async with session.get(f"https://logo.clearbit.com/{url}") as response:
        if response.status == 200:
            return str(response.url)
        else:
            return None
