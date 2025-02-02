import os
from typing import Optional, Annotated, AsyncGenerator

from aiohttp import ClientSession
from dotenv import load_dotenv
from fastapi import Depends
from fastapi_injectable import injectable
from redis import asyncio as aioredis

from src.connections import RedisConnectionManager
from src.constants import proxy, proxy_auth, headers

load_dotenv()


async def get_session() -> AsyncGenerator[ClientSession, None]:
    """
    Creates and yields an aiohttp ClientSession with proper cleanup.
    Headers can be customized as needed.
    """
    session = ClientSession(headers=headers, max_field_size=30000)
    try:
        yield session
    finally:
        await session.close()


async def get_redis() -> AsyncGenerator[aioredis.Redis, None]:
    """
    Get Redis connection instance
    """
    if not os.getenv('REDIS_URL'):
        raise ValueError("REDIS_URL not set in .env")

    redis = aioredis.from_url(os.getenv('REDIS_URL'))
    try:
        yield redis
    finally:
        await redis.close()


@injectable
async def get_redis_connection_manager(redis: Annotated[aioredis.Redis, Depends(get_redis)]) -> RedisConnectionManager:
    """
    Get Redis connection manager instance
    """
    connection_manager = RedisConnectionManager(redis)
    try:
        return connection_manager
    finally:
        await connection_manager.close()


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
