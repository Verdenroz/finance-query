import asyncio
import os
from typing import (
    Annotated, Any, Dict, Optional, Union, cast, Literal
)
from urllib.parse import urlparse

from curl_cffi import requests
from fastapi import Depends, HTTPException, Request
from fastapi_injectable import injectable
from redis import Redis
from starlette.websockets import WebSocket

from clients.fetch_client import CurlFetchClient
from connections import ConnectionManager
from src.connections import RedisConnectionManager
from src.context import request_context
from utils.constants import default_headers
from utils.market import MarketSchedule
from utils.yahoo_auth import YahooAuthManager


@injectable
async def get_request_context() -> Request | WebSocket:
    """Return whichever object (Request / WebSocket) is active on this task."""
    return request_context.get()


RequestContext = Annotated[Request | WebSocket, Depends(get_request_context)]


@injectable
async def get_connection_manager(
        websocket: WebSocket,
) -> RedisConnectionManager | ConnectionManager:
    """ Return the current connection manager based on if redis is enabled """
    return websocket.app.state.connection_manager


WebsocketConnectionManager = Annotated[RedisConnectionManager, ConnectionManager, Depends(get_connection_manager)]


@injectable
async def get_redis(request: RequestContext) -> Redis:
    """ Return shared redis client """
    return cast(Redis, request.app.state.redis)


RedisClient = Annotated[Redis, Depends(get_redis)]


@injectable
async def get_session(req: RequestContext) -> requests.Session:
    """
    Return the *shared* curl_cffi session placed on `app.state.session`
    by `lifespan(...)`
    """
    return cast(requests.Session, req.app.state.session)


Session = Annotated[requests.Session, Depends(get_session)]


@injectable
async def get_fetch_client(
        session: Session,
) -> CurlFetchClient:
    """ Returns a fetch client from the shared session """
    proxy = os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False") == "True" else None
    return CurlFetchClient(
        session=session, proxy=proxy, default_headers=default_headers
    )


FetchClient = Annotated[CurlFetchClient, Depends(get_fetch_client)]


@injectable
async def _get_auth_manager(req: RequestContext) -> YahooAuthManager:
    """ Return the auth manager saved in lifespan """
    mgr = getattr(req.app.state, "yahoo_auth_manager", None)
    if mgr is None:
        raise HTTPException(500, "Yahoo auth manager not initialised")
    return cast(YahooAuthManager, mgr)


AuthManager = Annotated[YahooAuthManager, Depends(_get_auth_manager)]


@injectable
async def get_yahoo_cookies(mgr: AuthManager) -> dict:
    """ Return current yahoo cookies, attempting refresh if not found """
    if mgr.is_expired() or mgr.cookie is None:
        await mgr.refresh_auth()
    if mgr.cookie is None:
        raise HTTPException(500, "Failed to obtain Yahoo cookies")
    return mgr.cookie


@injectable
async def get_yahoo_crumb(mgr: AuthManager) -> str:
    """ Return current yahoo crumb, attempting refresh if not found """
    if mgr.is_expired() or mgr.crumb is None:
        await mgr.refresh_auth()
    if mgr.crumb is None:
        raise HTTPException(500, "Failed to obtain Yahoo crumb")
    return mgr.crumb


YahooCookies = Annotated[dict, Depends(get_yahoo_cookies)]
YahooCrumb = Annotated[str, Depends(get_yahoo_crumb)]
Schedule = Annotated[MarketSchedule, Depends(MarketSchedule)]


@injectable
async def get_auth_data(
        cookies: YahooCookies, crumb: YahooCrumb
) -> dict:
    """ Return current pair of cookie and crumb """
    return {"cookies": cookies, "crumb": crumb}


@injectable
async def refresh_yahoo_auth(manager: AuthManager) -> bool:
    """ Quick util to force auth refresh """
    return await manager.refresh_auth()


@injectable
async def fetch(
        client: FetchClient,
        url: str = "",
        method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
        *,
        return_response: bool = False,
        max_retries: int = 3,
        retry_delay: float = 1.0,
) -> Optional[Union[str, requests.Response]]:
    """
    A thin async wrapper that delegates to CurlFetchClient.fetch
    """

    if not url:
        return None

    # Merge default ↔ custom headers (custom wins)
    merged_headers = {k: v for k, v in default_headers.items() if v is not None}
    if headers:
        # Make sure all header values are strings
        for k, v in headers.items():
            if v is not None:
                merged_headers[k] = str(v) if not isinstance(v, str) else v

    if "Referer" not in merged_headers:
        merged_headers["Referer"] = (
            "https://finance.yahoo.com/"
            if "yahoo.com" in url
            else "https://www.google.com/"
        )

    # Basic retry loop (exponential back-off)
    last_exc: Exception | None = None
    for attempt in range(max_retries):
        try:
            return await client.fetch(
                url,
                method=method,
                params=params,
                data=data,
                headers=merged_headers,
                return_response=return_response,
            )
        except Exception as exc:
            print(f"Attempt {attempt + 1} failed: {exc}, {str(exc)}")
            last_exc = exc
            if attempt < max_retries - 1:
                await asyncio.sleep(retry_delay * 2 ** attempt)
            else:
                raise HTTPException(
                    500,
                    f"Request failed after {max_retries} attempts: {exc}",
                ) from exc
    return None  # (never reached)


@injectable
async def get_logo(
        client: FetchClient,
        *,
        symbol: str = "",
        url: str | None = None,
) -> str | None:
    """
    Try logo.dev with the ticker first, fall back to domain icon.
    """
    token = "pk_Xd1Cdye3QYmCOXzcvxhxyw"  # personal public key
    if not symbol and not url:
        return None

    if symbol:
        maybe = await client.fetch(
            f"https://img.logo.dev/ticker/{symbol}?token={token}&retina=true",
            return_response=True,
        )
        if isinstance(maybe, requests.Response) and maybe.status_code == 200:
            return str(maybe.url)

    if url:
        domain = urlparse(url).netloc.replace("www.", "")
        maybe = await client.fetch(
            f"https://img.logo.dev/{domain}?token={token}&retina=true",
            return_response=True,
        )
        if isinstance(maybe, requests.Response) and maybe.status_code == 200:
            return str(maybe.url)

    return None


async def setup_proxy_whitelist() -> dict | None:
    """
    Setup proxy whitelist for BrightData or similar proxy services
    Returns the configuration data needed for cleanup
    """
    if not os.getenv("PROXY_TOKEN") or os.getenv("USE_PROXY", "False") != "True":
        raise ValueError("Proxy configuration is missing")

    ip_response = requests.get("https://api.ipify.org/")
    ip = ip_response.text
    api_url = "https://api.brightdata.com/zone/whitelist"
    proxy_header_token = {"Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}", "Content-Type": "application/json"}
    payload = {"ip": ip}
    response = requests.post(api_url, headers=proxy_header_token, json=payload)
    if 200 < response.status_code >= 300:
        print(f"Proxy whitelist setup failed: {response.text}")
        return None

    return {"api_url": api_url, "headers": proxy_header_token, "payload": payload}


async def remove_proxy_whitelist(proxy_data: dict) -> None:
    """
    Remove IP from proxy whitelist when application is shutting down
    """
    if not proxy_data:
        return
    requests.delete(proxy_data["api_url"], headers=proxy_data["headers"], json=proxy_data["payload"])
