import asyncio
import re
from datetime import UTC, datetime
from typing import Optional

from curl_cffi import requests


class YahooAuthError(RuntimeError):
    """Raised when we fail to obtain a valid cookie + crumb pair."""

    pass


class YahooAuthManager:
    """
    Fetches and caches Yahoo Finance auth (cookie jar + crumb).

    A single instance is created at app start and stored on ``app.state``.
    Every request that needs Yahoo data calls :func:`get_yahoo_auth` in
    ``dependencies.py`` which, under an *asyncio* lock, either returns the
    still-fresh pair or calls :meth:`refresh`.
    """

    # How many seconds we keep using an existing pair before refreshing
    _MIN_REFRESH_INTERVAL = 30

    def __init__(self) -> None:
        self._crumb: Optional[str] = None
        self._cookie: Optional[dict[str, str]] = None
        self._last_update: Optional[datetime] = None
        self._lock = asyncio.Lock()

    @property
    def crumb(self) -> str | None:
        return self._crumb

    @property
    def cookie(self) -> dict[str, str] | None:
        return self._cookie

    @property
    def last_update(self) -> datetime | None:
        return self._last_update

    async def refresh(self, proxy: str | None = None) -> None:
        """
        Always fetch a *new* cookie/crumb pair and cache it.

        Must be executed under ``self._lock`` by the caller.
        Raises :class:`YahooAuthError` on failure.
        """
        session = requests.Session(impersonate="chrome")
        if proxy:
            session.proxies = {"http": proxy, "https": proxy}

        session.get("https://fc.yahoo.com", timeout=10, allow_redirects=True)
        crumb = session.get(
            "https://query1.finance.yahoo.com/v1/test/getcrumb",
            timeout=10,
            allow_redirects=True,
        ).text.strip()

        if not crumb or "<html" in crumb:
            # fallback to csrf token method
            csrf_token, session_id = self._extract_csrf(session.get("https://guce.yahoo.com/consent", timeout=10).text)
            if not csrf_token or not session_id:
                raise YahooAuthError("Failed to extract CSRF token and session id")

            data = {
                "agree": ["agree", "agree"],
                "consentUUID": "default",
                "sessionId": session_id,
                "csrfToken": csrf_token,
                "originalDoneUrl": "https://finance.yahoo.com/",
                "namespace": "yahoo",
            }

            session.post(
                f"https://consent.yahoo.com/v2/collectConsent?sessionId={session_id}",
                data=data,
                timeout=10,
            )
            session.get(
                f"https://guce.yahoo.com/copyConsent?sessionId={session_id}",
                data=data,
                timeout=10,
            )

            crumb = session.get("https://query2.finance.yahoo.com/v1/test/getcrumb", timeout=10).text.strip()

            if not crumb or "<html" in crumb:
                raise YahooAuthError("Yahoo returned an invalid crumb")

        # Successfully obtained a cookie/crumb pair
        self._crumb = crumb
        self._cookie = dict(session.cookies)
        self._last_update = datetime.now(UTC)

    @staticmethod
    def _extract_csrf(html: str) -> tuple[Optional[str], Optional[str]]:
        """Extract CSRF token and session ID from the HTML response."""
        csrf_match = re.search(r'name="csrfToken"[^>]*value="([^"]+)"', html)
        sess_match = re.search(r'name="sessionId"[^>]*value="([^"]+)"', html)
        return (
            csrf_match.group(1) if csrf_match else None,
            sess_match.group(1) if sess_match else None,
        )

    async def get_or_refresh(self, proxy: str | None = None) -> tuple[dict, str]:
        """
        Return a valid cookie/crumb pair, refreshing if necessary.

        Ensures at most one concurrent refresh and throttles to one refresh
        every ``_MIN_REFRESH_INTERVAL`` seconds.
        """
        async with self._lock:
            if (
                self._cookie is None
                or self._crumb is None
                or self._last_update is None
                or (datetime.now(UTC) - self._last_update).total_seconds() > self._MIN_REFRESH_INTERVAL
            ):
                await self.refresh(proxy)
            # guaranteed non-None here
            return self._cookie, self._crumb
