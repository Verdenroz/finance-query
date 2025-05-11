import asyncio
import datetime
import os
import re
import threading
from typing import Dict, Optional, Tuple

from curl_cffi import requests


class YahooAuthManager:
    """
    Manager for Yahoo Finance authentication (cookies and crumb)
    """

    def __init__(self, refresh_interval: int = 86400):
        """
        Initialize Yahoo authentication manager

        Args:
            refresh_interval: Auth refresh interval in seconds (default: 24 hours)
        """
        self._crumb = None
        self._cookie = None
        self._last_update = None
        self._lock = threading.Lock()
        self._refresh_interval = refresh_interval
        self._refresh_task = None

    @property
    def crumb(self) -> Optional[str]:
        return self._crumb

    @property
    def cookie(self) -> Optional[Dict]:
        return self._cookie

    @property
    def last_update(self) -> Optional[datetime.datetime]:
        return self._last_update

    def update(self, crumb: str, cookie: Dict) -> None:
        """Update authentication data"""
        with self._lock:
            print(f"Updating Yahoo auth data: Crumb:{crumb}, Cookie:{cookie}")
            self._crumb = crumb
            self._cookie = cookie
            self._last_update = datetime.datetime.now()

    def is_expired(self) -> bool:
        """Check if the cached auth data is expired"""
        if self._last_update is None:
            return True
        delta = datetime.datetime.now() - self._last_update
        return delta.total_seconds() > self._refresh_interval

    async def start_refresh_task(self):
        """Start background task to refresh auth periodically"""
        if self._refresh_task is not None:
            # Cancel existing task if it's running
            self._refresh_task.cancel()
            try:
                await self._refresh_task
            except asyncio.CancelledError:
                pass

        # Start new refresh task
        self._refresh_task = asyncio.create_task(self._refresh_loop())

    async def _refresh_loop(self):
        """Background task to refresh auth periodically"""
        while True:
            try:
                await asyncio.sleep(self._refresh_interval)
                await self.refresh_auth()
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"Error refreshing Yahoo auth: {str(e)}")
                # Shorter retry interval on failure
                await asyncio.sleep(300)  # 5 minutes

    async def shutdown(self):
        """Shut down the refresh task"""
        if self._refresh_task:
            self._refresh_task.cancel()
            try:
                await self._refresh_task
            except asyncio.CancelledError:
                pass

    def _get_csrf_token_regex(self, html_content: str) -> Tuple[Optional[str], Optional[str]]:
        """Extract CSRF token and session ID using regex """
        csrf_match = re.search(r'<input[^>]*name="csrfToken"[^>]*value="([^"]*)"', html_content)
        session_match = re.search(r'<input[^>]*name="sessionId"[^>]*value="([^"]*)"', html_content)

        csrf_token = csrf_match.group(1) if csrf_match else None
        session_id = session_match.group(1) if session_match else None

        return csrf_token, session_id

    async def refresh_auth(self, proxy: Optional[str] = None) -> bool:
        """
        Refresh Yahoo Finance authentication

        Args:
            proxy: Optional proxy URL

        Returns:
            Success status (True/False)
        """
        # Don't use asyncio here as curl_cffi is already non-blocking
        try:
            # Use context manager to ensure proper cleanup
            session = requests.Session(impersonate="chrome")

            if proxy:
                session.proxies = {"http": proxy, "https": proxy}

            # Try basic strategy first (usually works and is simpler)
            # Get cookies
            session.get(
                url='https://fc.yahoo.com',
                timeout=10,
                allow_redirects=True
            )

            # Get crumb using cookies
            crumb_response = session.get(
                url="https://query1.finance.yahoo.com/v1/test/getcrumb",
                timeout=10,
                allow_redirects=True
            )

            crumb = crumb_response.text

            if crumb and '<html>' not in crumb and crumb.strip():
                self.update(crumb, dict(session.cookies))
                return True

            # Basic strategy failed, try CSRF strategy
            response = session.get(
                url='https://guce.yahoo.com/consent',
                timeout=10
            )

            # Extract CSRF token and session ID using regex
            csrf_token, session_id = self._get_csrf_token_regex(response.text)

            if not csrf_token or not session_id:
                return False

            data = {
                'agree': ['agree', 'agree'],
                'consentUUID': 'default',
                'sessionId': session_id,
                'csrfToken': csrf_token,
                'originalDoneUrl': 'https://finance.yahoo.com/',
                'namespace': 'yahoo',
            }

            session.post(
                url=f'https://consent.yahoo.com/v2/collectConsent?sessionId={session_id}',
                data=data,
                timeout=10
            )

            session.get(
                url=f'https://guce.yahoo.com/copyConsent?sessionId={session_id}',
                data=data,
                timeout=10
            )

            crumb_response = session.get(
                url='https://query2.finance.yahoo.com/v1/test/getcrumb',
                timeout=10
            )

            crumb = crumb_response.text

            if not crumb or '<html>' in crumb or crumb == '':
                return False

            self.update(crumb, dict(session.cookies))
            return True

        except Exception as e:
            print(f"Yahoo auth refresh error: {str(e)}")
            return False


async def setup_yahoo_auth(app) -> None:
    """Create `YahooAuthManager`, stash it on `app.state`, start auto‑refresh."""
    mgr = YahooAuthManager(refresh_interval=getattr(app.state, "auth_refresh_interval", 86_400))
    proxy = os.getenv("PROXY_URL") if os.getenv("USE_PROXY", "False") == "True" else None
    await mgr.refresh_auth(proxy)  # prime the cache
    app.state.yahoo_auth_manager = mgr
    await mgr.start_refresh_task()


async def cleanup_yahoo_auth(app) -> None:
    """Shut the background refresh loop down cleanly."""
    mgr: YahooAuthManager | None = getattr(app.state, "yahoo_auth_manager", None)
    if mgr:
        await mgr.shutdown()
