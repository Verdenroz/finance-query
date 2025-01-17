import os
from typing import Optional

from aiohttp import ClientSession
from dotenv import load_dotenv

from src.constants import proxy, proxy_auth
from src.session_manager import get_global_session

load_dotenv()


async def fetch(url: str, session: Optional[ClientSession] = None,
                use_proxy: bool = os.getenv('USE_PROXY', 'False') == 'True') -> str:
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
    if not url:
        return None

    session = session or await get_global_session()
    async with session.get(f"https://logo.clearbit.com/{url}") as response:
        if response.status == 200:
            return str(response.url)
        else:
            return None
