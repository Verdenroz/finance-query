import asyncio
import os
from typing import Optional, Annotated, AsyncGenerator, Union

from aiohttp import ClientSession, ClientResponse, ClientPayloadError, ClientError
from fastapi import Depends, Request, HTTPException
from fastapi_injectable import injectable
from redis import Redis
from starlette.websockets import WebSocket

from src.connections import RedisConnectionManager
from src.constants import proxy, proxy_auth, headers
from src.context import request_context


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


async def get_request_context() -> Request | WebSocket:
    """Get request context from FastAPI app"""
    return request_context.get()


async def get_connection_manager(websocket: WebSocket) -> RedisConnectionManager:
    """
    Get connection manager instance from app state using WebSocket
    """
    return websocket.app.state.connection_manager


@injectable
async def get_redis(request=Depends(get_request_context)) -> Redis:
    """Get Redis client from registered app state"""
    return request.app.state.redis


@injectable
async def get_yahoo_cookies(request=Depends(get_request_context)) -> dict:
    """Get Yahoo cookies from app state"""
    return request.app.state.cookies


@injectable
async def get_yahoo_crumb(request=Depends(get_request_context)) -> str:
    """Get Yahoo crumb from app state"""
    return request.app.state.crumb


@injectable
async def fetch(
        session: Annotated[ClientSession, Depends(get_session)],
        url: str = "",
        method: str = "GET",
        params: dict = None,
        headers: dict = None,
        return_response: bool = False,
        use_proxy: bool = os.getenv('USE_PROXY', 'False') == 'True',
        max_retries: int = 3,
        retry_delay: float = 1.0
) -> Optional[Union[str, ClientResponse]]:
    """
    Fetch URL content with retry logic and optional proxy support
    """
    if not url:
        return None

    if use_proxy:
        if not proxy or not proxy_auth:
            raise HTTPException(status_code=500, detail="Proxy configuration is missing")

    for attempt in range(max_retries):
        try:
            async with session.request(
                    method=method,
                    url=url,
                    params=params,
                    headers=headers,
                    proxy=proxy if use_proxy else None,
                    proxy_auth=proxy_auth if use_proxy else None,
                    timeout=5
            ) as response:
                if return_response:
                    # Create a new response object with the content already read
                    content = await response.read()
                    response._body = content
                    return response
                return await response.text()

        except (ClientPayloadError, asyncio.TimeoutError, ClientError) as e:
            if attempt == max_retries - 1:
                raise HTTPException(
                    status_code=500,
                    detail=f"Request failed after {max_retries} attempts: {str(e)}"
                )
            await asyncio.sleep(retry_delay)


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


async def _get_auth_data(redis: Redis = None) -> tuple[str, str]:
    if redis:
        cookies = redis.get('yahoo_cookies')
        crumb = redis.get('yahoo_crumb')
        if cookies and crumb:
            return cookies.decode('utf-8'), crumb.decode('utf-8')

    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Connection': 'keep-alive',
        }
        response = await fetch(url='https://finance.yahoo.com', headers=headers, return_response=True)
        cookies = response.headers.get('Set-Cookie', '')
        if cookies:
            headers['Cookie'] = cookies
            crumb = await _get_crumb(headers)
            if redis:
                redis.set('yahoo_cookies', cookies, ex=90 * 24 * 60 * 60)  # 90 days in seconds
                redis.set('yahoo_crumb', crumb, ex=90 * 24 * 60 * 60)  # 90 days in seconds
            return cookies, crumb
    except Exception as e:
        print(f"finance.yahoo.com auth failed: {e}")
        raise HTTPException(status_code=500, detail="Failed to authenticate with Yahoo Finance")


async def _get_crumb(headers: dict[str, str]) -> str:
    try:
        response = await fetch(url='https://query1.finance.yahoo.com/v1/test/getcrumb', headers=headers)
        crumb = response.strip('"')
        if crumb:
            return crumb
    except Exception as e:
        print(f"Crumb retrieval failed: {e}")

    raise ValueError("Failed to get valid crumb")
