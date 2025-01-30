import os
from typing import Optional, Annotated

from aiohttp import ClientSession
from dotenv import load_dotenv
from fastapi import Depends
from fastapi_injectable import injectable

from src.constants import proxy, proxy_auth
from src.di import get_session

load_dotenv()


@injectable
async def fetch(
        session: Annotated[ClientSession, Depends(get_session)],
        url: str = "",
        use_proxy: bool = os.getenv('USE_PROXY', 'False') == 'True',
) -> str:
    """
    Fetch URL content with optional proxy support
    """
    if not url:
        return ""

    # Check if proxy is enabled and required values are set
    if use_proxy:
        if proxy is None or proxy_auth is None:
            raise ValueError("Proxy URL/Auth/Token not set in .env")

        async with session.get(url, proxy=proxy, proxy_auth=proxy_auth) as response:
            return await response.text()

    async with session.get(url) as response:
        return await response.text()


@injectable
async def get_logo(
        session: Annotated[ClientSession, Depends(get_session)],
        url: str = "",
) -> Optional[str]:
    """
    Get logo URL from Clearbit
    """
    if not url:
        return None

    async with session.get(f"https://logo.clearbit.com/{url}") as response:
        if response.status == 200:
            return str(response.url)
        return None
