import asyncio
import os
import time
from datetime import datetime, timedelta
from typing import Annotated, Any, Literal, Optional, Union, cast
from urllib.parse import urlparse

from curl_cffi import requests
from fastapi import Depends, HTTPException, Request
from fastapi_injectable import injectable
from redis import Redis
from starlette.websockets import WebSocket

from src.clients.fetch_client import CurlFetchClient
from src.clients.yahoo_client import YahooFinanceClient
from src.connections import ConnectionManager, RedisConnectionManager
from src.context import request_context
from src.utils.cache import cache
from src.utils.logging import get_logger, log_external_api_call
from src.utils.market import MarketSchedule
from src.utils.proxy_rotator import ProxyRotator
from src.utils.yahoo_auth import YahooAuthManager

Schedule = Annotated[MarketSchedule, Depends(MarketSchedule)]
logger = get_logger(__name__)


class CircuitBreaker:
    """Simple circuit breaker for external API calls."""

    def __init__(self, failure_threshold: int = 5, timeout_duration: int = 300):
        self.failure_threshold = failure_threshold
        self.timeout_duration = timeout_duration  # seconds
        self.failure_count = 0
        self.last_failure_time: Optional[datetime] = None
        self.state = "CLOSED"  # CLOSED, OPEN, HALF_OPEN

    def is_available(self) -> bool:
        """Check if the circuit breaker allows calls."""
        if self.state == "CLOSED":
            return True

        if self.state == "OPEN":
            if self.last_failure_time and datetime.now() - self.last_failure_time > timedelta(seconds=self.timeout_duration):
                self.state = "HALF_OPEN"
                return True
            return False

        return True  # HALF_OPEN allows one call

    def record_success(self):
        """Record a successful call."""
        self.failure_count = 0
        self.state = "CLOSED"
        self.last_failure_time = None

    def record_failure(self):
        """Record a failed call."""
        self.failure_count += 1
        self.last_failure_time = datetime.now()

        if self.failure_count >= self.failure_threshold:
            self.state = "OPEN"
            logger.warning(f"Circuit breaker OPEN after {self.failure_count} failures")


# Global circuit breaker for Logo.dev API
logo_circuit_breaker = CircuitBreaker(
    failure_threshold=int(os.getenv("LOGO_CIRCUIT_BREAKER_THRESHOLD", "5")), timeout_duration=int(os.getenv("LOGO_CIRCUIT_BREAKER_TIMEOUT", "300"))
)


async def get_request_context() -> Union[Request, WebSocket]:
    """Return whichever object (Request / WebSocket) is active on this task."""
    return request_context.get()


RequestContext = Annotated[Union[Request, WebSocket], Depends(get_request_context)]


async def get_connection_manager(
    websocket: WebSocket,
) -> Union[RedisConnectionManager, ConnectionManager]:
    """Return the current connection manager based on if redis is enabled"""
    return websocket.app.state.connection_manager


WebsocketConnectionManager = Annotated[RedisConnectionManager, ConnectionManager, Depends(get_connection_manager)]


async def get_redis(request: RequestContext) -> Redis:
    """Return shared redis client"""
    return cast(Redis, request.app.state.redis)


RedisClient = Annotated[Redis, Depends(get_redis)]


async def get_session(req: RequestContext) -> requests.Session:
    """
    Return the *shared* curl_cffi session placed on `app.state.session`
    by `lifespan(...)`
    """
    return cast(requests.Session, req.app.state.session)


Session = Annotated[requests.Session, Depends(get_session)]


async def get_proxy_rotator(req: RequestContext) -> Optional[ProxyRotator]:
    """
    Return the ProxyRotator instance from app state if available.
    """
    return getattr(req.app.state, "proxy_rotator", None)


ProxyRotatorDep = Annotated[Optional[ProxyRotator], Depends(get_proxy_rotator)]


@injectable
async def get_proxy() -> Optional[str]:
    """
    Return the proxy URL if set, otherwise None.
    Backward compatibility function - for single proxy URL access.
    """
    return os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False").lower() == "true" else None


Proxy = Annotated[Optional[str], Depends(get_proxy)]


async def get_fetch_client(
    session: Session,
    proxy_rotator: ProxyRotatorDep,
) -> CurlFetchClient:
    """
    Returns a fetch client from the shared session.
    Works with ProxyRotator for per-request proxy selection.
    """
    # For backward compatibility, if no rotator, try to get single proxy URL
    proxy_url = None
    if proxy_rotator is None:
        proxy_url = os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False").lower() == "true" else None

    return CurlFetchClient(session=session, proxy=proxy_url)


FetchClient = Annotated[CurlFetchClient, Depends(get_fetch_client)]


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
    cookies, crumb = await mgr.get_or_refresh(proxy=os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False").lower() == "true" else None)
    return cookies, crumb


YahooAuth = Annotated[tuple[dict, str], Depends(get_yahoo_auth)]


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
    proxy_rotator: ProxyRotatorDep = None,
) -> Optional[Union[str, requests.Response]]:
    """
    A thin async wrapper that delegates to CurlFetchClient.fetch
    with automatic proxy rotation support.
    """
    # Manually resolve proxy_rotator if not injected (e.g., when called directly from services)
    if proxy_rotator is None:
        try:
            req = request_context.get()
            proxy_rotator = getattr(req.app.state, "proxy_rotator", None)
        except (LookupError, AttributeError, RuntimeError):
            # No request context available (e.g., background task, startup/shutdown)
            proxy_rotator = None

    # Default headers for the request
    default_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.9",
        "Accept-Encoding": "gzip, deflate, br",
        "sec-ch-ua": '"Chromium";v="122", "Google Chrome";v="122"',
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": '"Windows"',
    }

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

    # Retry loop with proxy rotation
    for attempt in range(max_retries):
        # Get proxy for this attempt (rotate on retries)
        proxy = None
        if proxy_rotator:
            proxy = proxy_rotator.get_proxy()
            if proxy:
                logger.debug(f"Using proxy for request", extra={"proxy": proxy, "attempt": attempt + 1, "url": url})
            else:
                logger.warning("ProxyRotator returned None, proceeding without proxy")

        try:
            result = await client.fetch(
                url,
                method=method,
                params=params,
                data=data,
                headers=merged_headers,
                return_response=return_response,
                proxy=proxy,
            )
            # Mark proxy as successful if rotator is available
            if proxy_rotator and proxy:
                proxy_rotator.mark_success(proxy)
                logger.debug(f"Request succeeded", extra={"proxy": proxy, "attempt": attempt + 1})
            return result

        except Exception as exc:
            # Mark proxy as failed if rotator is available
            if proxy_rotator and proxy:
                proxy_rotator.mark_failure(proxy)
                logger.warning(
                    "Request failed with proxy",
                    extra={
                        "proxy": proxy,
                        "attempt": attempt + 1,
                        "max_retries": max_retries,
                        "error": str(exc),
                        "error_type": type(exc).__name__,
                    },
                )
            else:
                logger.warning(
                    "Request attempt failed",
                    extra={"attempt": attempt + 1, "max_retries": max_retries, "error": str(exc), "error_type": type(exc).__name__},
                )

            if attempt < max_retries - 1:
                delay = retry_delay * 2**attempt
                logger.debug(f"Retrying request after {delay}s with different proxy", extra={"attempt": attempt + 1})
                await asyncio.sleep(delay)
            else:
                logger.error("All request attempts failed", extra={"max_retries": max_retries, "final_error": str(exc)}, exc_info=True)
                raise HTTPException(
                    500,
                    f"Request failed after {max_retries} attempts: {exc}",
                ) from exc

    return None  # (never reached)


@cache(expire=86400)  # Cache for 24 hours
@injectable
async def get_logo(
    client: FetchClient,
    *,
    symbol: str = "",
    url: str | None = None,
) -> str | None:
    """
    Try logo.dev with the ticker first, fall back to domain icon.
    Cached for 24 hours to prevent repeated expensive calls.
    Uses circuit breaker to prevent cascading failures.
    """
    # Check if logo fetching is disabled
    if os.getenv("DISABLE_LOGO_FETCHING", "false").lower() == "true":
        return None

    # Check circuit breaker
    if not logo_circuit_breaker.is_available():
        logger.debug(f"Logo.dev circuit breaker is OPEN, skipping logo fetch for {symbol or 'domain'}")
        return None

    token = "pk_Xd1Cdye3QYmCOXzcvxhxyw"  # personal public key
    if not symbol and not url:
        return None

    # Set timeout for logo requests (default 1 seconds)
    logo_timeout = float(os.getenv("LOGO_TIMEOUT_SECONDS", "1"))

    if symbol:
        start_time = time.perf_counter()
        try:
            maybe = await asyncio.wait_for(
                client.fetch(
                    f"https://img.logo.dev/ticker/{symbol}?token={token}&retina=true",
                    return_response=True,
                ),
                timeout=logo_timeout,
            )
            duration_ms = (time.perf_counter() - start_time) * 1000
            success = isinstance(maybe, requests.Response) and maybe.status_code == 200
            log_external_api_call(logger, "Logo.dev", "ticker", duration_ms, success=success)

            if success:
                logo_circuit_breaker.record_success()
                return str(maybe.url)
            else:
                logo_circuit_breaker.record_failure()
        except Exception:
            duration_ms = (time.perf_counter() - start_time) * 1000
            log_external_api_call(logger, "Logo.dev", "ticker", duration_ms, success=False)
            logo_circuit_breaker.record_failure()
            # Don't re-raise, fall through to domain lookup

    if url:
        domain = urlparse(url).netloc.replace("www.", "")
        start_time = time.perf_counter()
        try:
            maybe = await asyncio.wait_for(
                client.fetch(
                    f"https://img.logo.dev/{domain}?token={token}&retina=true",
                    return_response=True,
                ),
                timeout=logo_timeout,
            )
            duration_ms = (time.perf_counter() - start_time) * 1000
            success = isinstance(maybe, requests.Response) and maybe.status_code == 200
            log_external_api_call(logger, "Logo.dev", "domain", duration_ms, success=success)

            if success:
                logo_circuit_breaker.record_success()
                return str(maybe.url)
            else:
                logo_circuit_breaker.record_failure()

        except TimeoutError:
            duration_ms = (time.perf_counter() - start_time) * 1000
            log_external_api_call(logger, "Logo.dev", "domain", duration_ms, success=False)
            logger.warning(f"Logo fetch timeout for domain {domain} after {logo_timeout}s")
            logo_circuit_breaker.record_failure()
            # Don't re-raise, return None
        except Exception:
            duration_ms = (time.perf_counter() - start_time) * 1000
            log_external_api_call(logger, "Logo.dev", "domain", duration_ms, success=False)
            logo_circuit_breaker.record_failure()
            # Don't re-raise, return None

    return None


async def setup_proxy_whitelist() -> dict | None:
    """
    Setup proxy whitelist for BrightData or similar proxy services
    Returns the configuration data needed for cleanup
    """
    if not os.getenv("PROXY_TOKEN") or os.getenv("USE_PROXY", "False").lower() != "true":
        raise ValueError("Proxy configuration is missing")

    # Get IP address
    start_time = time.perf_counter()
    try:
        ip_response = requests.get("https://api.ipify.org/")
        duration_ms = (time.perf_counter() - start_time) * 1000
        success = ip_response.status_code == 200
        log_external_api_call(logger, "ipify", "get_ip", duration_ms, success=success)
        if not success:
            return None
        ip = ip_response.text
    except Exception:
        duration_ms = (time.perf_counter() - start_time) * 1000
        log_external_api_call(logger, "ipify", "get_ip", duration_ms, success=False)
        return None

    # Setup proxy whitelist
    api_url = "https://api.brightdata.com/zone/whitelist"
    proxy_header_token = {"Authorization": f"Bearer {os.getenv('PROXY_TOKEN')}", "Content-Type": "application/json"}
    payload = {"ip": ip}

    start_time = time.perf_counter()
    try:
        response = requests.post(api_url, headers=proxy_header_token, json=payload)
        duration_ms = (time.perf_counter() - start_time) * 1000
        success = 200 <= response.status_code < 300
        log_external_api_call(logger, "BrightData", "whitelist_add", duration_ms, success=success)
    except Exception:
        duration_ms = (time.perf_counter() - start_time) * 1000
        log_external_api_call(logger, "BrightData", "whitelist_add", duration_ms, success=False)
        return None
    if not success:
        logger.error("Proxy whitelist setup failed", extra={"status_code": response.status_code, "response_text": response.text})
        return None

    return {"api_url": api_url, "headers": proxy_header_token, "payload": payload}


async def remove_proxy_whitelist(proxy_data: dict) -> None:
    """
    Remove IP from proxy whitelist when application is shutting down
    """
    if not proxy_data:
        return

    start_time = time.perf_counter()
    try:
        response = requests.delete(proxy_data["api_url"], headers=proxy_data["headers"], json=proxy_data["payload"])
        duration_ms = (time.perf_counter() - start_time) * 1000
        success = 200 <= response.status_code < 300
        log_external_api_call(logger, "BrightData", "whitelist_remove", duration_ms, success=success)
    except Exception:
        duration_ms = (time.perf_counter() - start_time) * 1000
        log_external_api_call(logger, "BrightData", "whitelist_remove", duration_ms, success=False)
