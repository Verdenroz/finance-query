import asyncio
import datetime
import os
import time
from typing import Optional, Annotated, Union
from urllib.parse import urlparse

import requests
from aiohttp import ClientSession, ClientResponse, ClientPayloadError, ClientError
from fastapi import Depends, Request, HTTPException, FastAPI
from fastapi_injectable import injectable
from redis import Redis
from starlette.websockets import WebSocket

from src.connections import RedisConnectionManager
from src.constants import proxy, proxy_auth
from src.context import request_context


async def get_request_context() -> Request | WebSocket:
    """Get request context from FastAPI app"""
    return request_context.get()


async def get_connection_manager(websocket: WebSocket) -> RedisConnectionManager:
    """
    Get connection manager instance from app state using WebSocket
    """
    return websocket.app.state.connection_manager


async def get_session(request=Depends(get_request_context)) -> ClientSession:
    """
    Creates and yields an aiohttp ClientSession with proper cleanup.
    Headers can be customized as needed.
    """
    return request.app.state.session


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

    return None


@injectable
async def get_logo(
        session: Annotated[ClientSession, Depends(get_session)],
        symbol: str = "",
        url: Optional[str] = None,
) -> Optional[str]:
    """
    Get logo URL from logo.dev
    """
    if not url and not symbol:
        return None

    async with session.get(f"https://img.logo.dev/ticker/{symbol}?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true") as response:
        if response.status == 200:
            return str(response.url)

    # Fallback to using the domain if the symbol request fails
    if url:
        parsed_url = urlparse(url)
        domain = parsed_url.netloc.replace('www.', '')
        # The token is my personal public key, but feel free to use your own
        async with session.get(f"https://img.logo.dev/{domain}?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true") as response:
            if response.status == 200:
                return str(response.url)

    return None


async def get_auth_data() -> tuple[str, str] | None:
    """
    Get Yahoo Finance authentication data (cookies and crumb)

    Raises:
        HTTPException: If unable to get cookies or crumb
    """
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Connection': 'keep-alive',
        }
        response = requests.get('https://finance.yahoo.com', headers=headers)
        cookies_dict = response.cookies.get_dict()
        cookies_str = '; '.join([f'{k}={v}' for k, v in cookies_dict.items()])

        if cookies_str:
            headers['Cookie'] = cookies_str
            crumb = get_crumb(headers)
            return cookies_str, crumb

    except Exception as e:
        print(f"finance.yahoo.com auth failed: {e}")
        raise HTTPException(status_code=500, detail="Failed to authenticate with Yahoo Finance")


def get_crumb(headers: dict[str, str]) -> str:
    try:
        response = requests.get('https://query1.finance.yahoo.com/v1/test/getcrumb', headers=headers)
        crumb = response.text.strip('"')
        if crumb:
            return crumb
    except Exception as e:
        print(f"Crumb retrieval failed: {e}")

    raise ValueError("Failed to get valid crumb")


async def refresh_yahoo_auth(app: FastAPI) -> None:
    """Background task to refresh Yahoo Finance authentication"""
    while True:
        try:
            current_time = time.time()

            # If auth_expiry doesn't exist or time has passed the expiry
            if not app.state.auth_expiry or current_time > app.state.auth_expiry:
                # Get new auth data
                cookies, crumb = await get_auth_data()

                # Update app state
                app.state.cookies = cookies
                app.state.crumb = crumb
                app.state.auth_expiry = current_time + app.state.auth_refresh_interval

                refresh_time = datetime.datetime.now().isoformat()
                print(f"Yahoo Finance auth refreshed at {refresh_time}, next refresh at "
                      f"{datetime.datetime.fromtimestamp(app.state.auth_expiry).isoformat()}")
        except Exception as e:
            print(f"Auth refresh error: {e}")

        # Sleep until 5 minutes before expiry or check every hour if something went wrong
        sleep_time = 3600  # Default 1 hour
        if app.state.auth_expiry:
            time_to_expiry = max(0, app.state.auth_expiry - time.time() - 300)  # 5 minutes before expiry
            sleep_time = min(time_to_expiry, 3600)  # Don't wait more than an hour

        await asyncio.sleep(sleep_time)


async def setup_proxy_whitelist() -> dict | None:
    """
    Setup proxy whitelist for BrightData or similar proxy services
    Returns the configuration data needed for cleanup
    """
    if not os.getenv('PROXY_TOKEN') or os.getenv('USE_PROXY', 'False') != 'True':
        raise ValueError("Proxy configuration is missing")

    ip_response = requests.get("https://api.ipify.org/")
    ip = ip_response.text
    api_url = "https://api.brightdata.com/zone/whitelist"
    proxy_header_token = {
        "Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}",
        "Content-Type": "application/json"
    }
    payload = {"ip": ip}

    response = requests.post(api_url, headers=proxy_header_token, json=payload)
    if response.status_code != 200:
        print(f"Proxy whitelist setup failed: {response.text}")
        return None

    return {
        "api_url": api_url,
        "headers": proxy_header_token,
        "payload": payload
    }


async def remove_proxy_whitelist(proxy_data: dict) -> None:
    """
    Remove IP from proxy whitelist when application is shutting down
    """
    if not proxy_data:
        return
    requests.delete(
        proxy_data["api_url"],
        headers=proxy_data["headers"],
        json=proxy_data["payload"]
    )
