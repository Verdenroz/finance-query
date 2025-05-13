import asyncio
import os
from typing import Annotated, Any, Literal, Optional, Union, cast
from urllib.parse import urlparse

from clients.fetch_client import CurlFetchClient
from clients.yahoo_client import YahooFinanceClient
from connections import ConnectionManager
from curl_cffi import requests
from fastapi import Depends, HTTPException, Request
from fastapi_injectable import injectable
from redis import Redis
from starlette.websockets import WebSocket
from utils.constants import default_headers
from utils.market import MarketSchedule
from utils.yahoo_auth import YahooAuthManager

from src.connections import RedisConnectionManager
from src.context import request_context

Schedule = Annotated[MarketSchedule, Depends(MarketSchedule)]


@injectable
async def get_request_context() -> Request | WebSocket:
    """Return whichever object (Request / WebSocket) is active on this task."""
    return request_context.get()


RequestContext = Annotated[Request | WebSocket, Depends(get_request_context)]


@injectable
async def get_connection_manager(
    websocket: WebSocket,
) -> RedisConnectionManager | ConnectionManager:
    """Return the current connection manager based on if redis is enabled"""
    return websocket.app.state.connection_manager


WebsocketConnectionManager = Annotated[RedisConnectionManager, ConnectionManager, Depends(get_connection_manager)]


@injectable
async def get_redis(request: RequestContext) -> Redis:
    """Return shared redis client"""
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
async def get_proxy() -> str | None:
    """
    Return the proxy URL if set, otherwise None.
    """
    return os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False") == "True" else None


Proxy = Annotated[str | None, Depends(get_proxy)]


@injectable
async def get_fetch_client(
    session: Session,
    proxy: Proxy,
) -> CurlFetchClient:
    """Returns a fetch client from the shared session"""
    return CurlFetchClient(session=session, proxy=proxy, default_headers=default_headers)


FetchClient = Annotated[CurlFetchClient, Depends(get_fetch_client)]


@injectable
async def _get_auth_manager(req: RequestContext) -> YahooAuthManager:
    """Return the auth manager saved in lifespan"""
    mgr = getattr(req.app.state, "yahoo_auth_manager", None)
    if mgr is None:
        raise HTTPException(500, "Yahoo auth manager not initialised")
    return cast(YahooAuthManager, mgr)


AuthManager = Annotated[YahooAuthManager, Depends(_get_auth_manager)]


@injectable
async def get_yahoo_auth(mgr: AuthManager) -> tuple[dict, str]:
    """
    Returns ``(cookies, crumb)``.  Serialises refreshes so that at most
    one coroutine hits Yahoo at a time; subsequent concurrent calls get
    the freshly cached pair.
    """
    cookies, crumb = await mgr.get_or_refresh(proxy=os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False") == "True" else None)
    return cookies, crumb


YahooAuth = Annotated[tuple[dict, str], Depends(get_yahoo_auth)]

YahooCookies = Annotated[dict, Depends(get_yahoo_auth)]
YahooCrumb = Annotated[str, Depends(get_yahoo_auth)]


@injectable
async def get_yahoo_finance_client(auth: YahooAuth, proxy: Proxy) -> CurlFetchClient:
    """
    Returns a YahooFinanceClient with the given auth and fetch client.
    """
    cookies, crumb = auth
    return YahooFinanceClient(
        cookies=cookies,
        crumb=crumb,
        proxy=proxy,
    )


FinanceClient = Annotated[YahooFinanceClient, Depends(get_yahoo_finance_client)]


@injectable
async def fetch(
    client: FetchClient,
    url: str = "",
    method: Literal["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "PATCH", "QUERY"] = "GET",
    params: Optional[dict[str, Any]] = None,
    data: Optional[dict[str, Any]] = None,
    headers: Optional[dict[str, str]] = None,
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
        merged_headers["Referer"] = "https://finance.yahoo.com/" if "yahoo.com" in url else "https://www.google.com/"

    # Basic retry loop (exponential back-off)
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
            if attempt < max_retries - 1:
                await asyncio.sleep(retry_delay * 2**attempt)
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
