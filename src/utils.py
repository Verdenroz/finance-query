import os
from datetime import datetime
from typing import Optional

import pytz
from aiohttp import ClientSession
from dotenv import load_dotenv

from src.constants import headers
from src.proxy import proxy, proxy_auth

load_dotenv()

global_session = ClientSession(max_field_size=20000, headers=headers)

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


async def fetch(
        url: str,
        session: ClientSession = global_session,
        use_proxy: bool = os.getenv('USE_PROXY', 'False') == 'True') -> str:
    """
    Fetch the data from the given URL with proxy if enabled
    :param url: the URL to fetch data from
    :param session: the global aiohttp ClientSession
    :param use_proxy: whether to use a proxy or not (requires proxy vars to be set in .env)
    :return: the html content of the page

    :raises ValueError: if proxy is enabled but proxy URL and/or Proxy Auth not set in .env
    """
    if use_proxy:
        if proxy is None or proxy_auth is None or os.getenv('PROXY_TOKEN') is None:
            raise ValueError("Proxy URL/Auth/Token not set in .env")

        async with session.get("https://api.ipify.org/") as ip_response:
            # Get current IP address
            ip = await ip_response.text()

            api_url = "https://api.brightdata.com/zone/whitelist"
            proxy_header_token = {
                "Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}",
                "Content-Type": "application/json"
            }
            payload = {
                "ip": ip
            }
            # Whitelists current IP address
            await session.post(api_url, headers=proxy_header_token, json=payload)

            try:
                async with session.get(url, proxy=proxy, proxy_auth=proxy_auth) as response:
                    html = await response.text()
            finally:
                # Deletes the IP address from the whitelist
                await session.delete(api_url, headers=proxy_header_token, json=payload)

            return html

    else:
        async with session.get(url) as response:
            return await response.text()


async def get_logo(url: str, session: ClientSession = global_session) -> Optional[str]:
    """
    Get the logo of the company from the given URL
    :param url: the URL of the company
    :param session: the global aiohttp ClientSession
    :return: the URL of the logo as a string
    """
    async with session.get(f"https://logo.clearbit.com/{url}") as response:
        if response.status == 200:
            return str(response.url)
        else:
            return None
