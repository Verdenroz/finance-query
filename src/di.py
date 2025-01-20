from typing import Optional

from aiohttp import ClientSession

from src.constants import headers
from src.security import RateLimitManager

global_session: Optional[ClientSession] = None
global_rate_limit_manager: Optional[RateLimitManager] = None


async def get_global_session() -> ClientSession:
    global global_session
    if global_session is None:
        global_session = ClientSession(max_field_size=30000, headers=headers)
    return global_session


async def close_global_session():
    global global_session
    if global_session is not None:
        await global_session.close()
        global_session = None


def get_global_rate_limit_manager() -> RateLimitManager:
    global global_rate_limit_manager
    if global_rate_limit_manager is None:
        global_rate_limit_manager = RateLimitManager()
    return global_rate_limit_manager
