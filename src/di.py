from typing import Optional, AsyncGenerator

from aiohttp import ClientSession

from src.constants import headers
from src.security import RateLimitManager

global_rate_limit_manager: Optional[RateLimitManager] = None


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


def get_global_rate_limit_manager() -> RateLimitManager:
    global global_rate_limit_manager
    if global_rate_limit_manager is None:
        global_rate_limit_manager = RateLimitManager()
    return global_rate_limit_manager
