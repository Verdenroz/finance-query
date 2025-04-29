from typing import AsyncGenerator, Annotated

from fastapi import Depends
from fastapi_injectable import injectable
from starlette.websockets import WebSocket

from src.security.rate_limit_manager import RateLimitManager


async def get_rate_limit_manager() -> AsyncGenerator[RateLimitManager, None]:
    """
    Generator dependency that provides the RateLimitManager instance.
    """
    manager = RateLimitManager()
    yield manager
    await manager.cleanup()


@injectable
async def increment_and_check(
    rate_limit_manager: Annotated[RateLimitManager, Depends(get_rate_limit_manager)],
    ip: str = "",
    api_key: str | None = None,
) -> tuple[bool, dict]:
    """Returns (is_allowed, rate_limit_info)"""
    return await rate_limit_manager.increment_and_check(ip, api_key)


@injectable
async def check_health_rate_limit(
    rate_limit_manager: Annotated[RateLimitManager, Depends(get_rate_limit_manager)],
    ip: str = "",
    api_key: str | None = None,
) -> tuple[bool, dict]:
    """Returns (is_allowed, rate_limit_info) for health check endpoint"""
    return await rate_limit_manager.check_health_rate_limit(ip, api_key)


@injectable
async def validate_websocket(
    rate_limit_manager: Annotated[RateLimitManager, Depends(get_rate_limit_manager)],
    websocket: WebSocket,
) -> tuple[bool, dict]:
    """
    Backwards compatible wrapper for websocket validation
    """
    return await rate_limit_manager.validate_websocket(websocket)
