import hashlib
import os
import threading
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import requests
from fastapi.testclient import TestClient
from orjson import orjson
from starlette.websockets import WebSocket

from src.connections import ConnectionManager, RedisConnectionManager
from src.main import app
from src.models import HistoricalData
from src.utils.dependencies import FinanceClient
from src.utils.yahoo_auth import YahooAuthManager

VERSION = "v1"


class ThreadSafeHTMLCacheManager:
    """Thread-safe HTML cache manager for caching real web responses."""

    def __init__(self, cache_dir: Path):
        self.cache_dir = cache_dir
        self.locks = {}
        self.locks_lock = threading.Lock()

    def _get_lock(self, cache_key: str) -> threading.Lock:
        """Get or create a lock for a specific cache key."""
        with self.locks_lock:
            if cache_key not in self.locks:
                self.locks[cache_key] = threading.Lock()
            return self.locks[cache_key]

    def _get_worker_id(self) -> str:
        """Get pytest worker ID for parallel execution isolation."""
        worker_id = os.environ.get("PYTEST_XDIST_WORKER", "master")
        return worker_id

    def _get_cache_file_path(self, url: str, context: str = "") -> Path:
        """Generate cache file path with worker isolation."""
        worker_id = self._get_worker_id()
        self.cache_dir.mkdir(parents=True, exist_ok=True)

        # Create a safe filename from URL and context
        url_hash = hashlib.md5(url.encode()).hexdigest()
        if context:
            context_safe = "".join(c for c in context if c.isalnum() or c in "._-")
            filename = f"{worker_id}_{context_safe}_{url_hash}.html"
        else:
            filename = f"{worker_id}_{url_hash}.html"

        return self.cache_dir / filename

    def get_cached_html(self, url: str, context: str = "", headers: dict = None) -> str:
        """
        Get cached HTML content or fetch from URL if not available.
        Thread-safe with file locking.
        """
        cache_file = self._get_cache_file_path(url, context)
        lock = self._get_lock(str(cache_file))

        with lock:
            # Check if cache exists
            if cache_file.exists():
                try:
                    with open(cache_file, encoding="utf-8") as f:
                        return f.read()
                except (OSError, UnicodeDecodeError):
                    # If cache is corrupted, remove it and fetch fresh
                    cache_file.unlink(missing_ok=True)

            # Fetch fresh HTML content
            if headers is None:
                headers = {"User-Agent": "Mozilla/5.0"}

            response = requests.get(url, headers=headers)
            response.raise_for_status()
            html_content = response.text

            # Save to cache atomically
            temp_file = cache_file.with_suffix(f"{cache_file.suffix}.tmp")
            try:
                with open(temp_file, "w", encoding="utf-8") as f:
                    f.write(html_content)
                temp_file.replace(cache_file)
            except Exception:
                temp_file.unlink(missing_ok=True)
                raise

            return html_content


# Global HTML cache manager instance
_html_cache_manager = None


def get_html_cache_manager() -> ThreadSafeHTMLCacheManager:
    """Get the global HTML cache manager instance."""
    global _html_cache_manager
    if _html_cache_manager is None:
        cache_dir = Path(__file__).parent / "data" / "cache"
        _html_cache_manager = ThreadSafeHTMLCacheManager(cache_dir)
    return _html_cache_manager


@pytest.fixture(scope="session")
def html_cache_setup():
    """Set up HTML cache directory for the test session."""
    cache_manager = get_html_cache_manager()
    cache_manager.cache_dir.mkdir(parents=True, exist_ok=True)

    yield cache_manager

    # Cleanup: Only remove files for this worker to avoid conflicts
    worker_id = cache_manager._get_worker_id()
    for cache_file in cache_manager.cache_dir.glob(f"{worker_id}_*.html"):
        try:
            cache_file.unlink()
        except FileNotFoundError:
            pass


@pytest.fixture(scope="session")
def html_cache_manager(html_cache_setup):
    """Provides HTML caching functionality for real web requests."""
    cache_manager = html_cache_setup

    def get_cached_html(url: str, context: str = "", headers: dict = None) -> str:
        """Get cached HTML content or fetch from URL."""
        return cache_manager.get_cached_html(url, context, headers)

    return get_cached_html


@pytest.fixture(scope="session")
def mock_redis():
    """Mock Redis client"""
    redis_mock = MagicMock()
    redis_mock.get.return_value = None  # Simulate no cached data
    return redis_mock


@pytest.fixture(scope="session")
def mock_session():
    """Mock aiohttp ClientSession"""
    session_mock = AsyncMock()
    return session_mock


@pytest.fixture(scope="session")
def yahoo_auth_manager():
    """
    Real instance (so internals are exercised) but patched so that
    .get_or_refresh() never reaches out to Yahoo.
    """
    mgr = YahooAuthManager()

    # Use patch as a context manager inside the fixture
    original_get_or_refresh = mgr.get_or_refresh

    # Create the mock function
    async def _fake_get_or_refresh(*_, **__):
        return {"B": "fake_cookie"}, "fake_crumb"

    # Replace the real method with the mock
    mgr.get_or_refresh = AsyncMock(side_effect=_fake_get_or_refresh)

    yield mgr

    # Restore the original after tests
    mgr.get_or_refresh = original_get_or_refresh


@pytest.fixture(autouse=True)
def mock_finance_client():
    """
    The object FastAPI will get back from utils.dependencies.get_yahoo_finance_client.
    You can preset return values (or side_effects) per-test.

    This fixture uses FastAPI's dependency_overrides to properly replace the dependency.
    """
    client = AsyncMock(name="FinanceClient")
    client.get_quote = AsyncMock()
    client.get_simple_quotes = AsyncMock()
    client.get_chart = AsyncMock()
    client.search = AsyncMock()
    client.get_similar_quotes = AsyncMock()

    # Store the original overrides
    original_overrides = app.dependency_overrides.copy()

    # Set the override for the dependency
    app.dependency_overrides[FinanceClient] = client

    yield client

    # Restore original overrides after the test
    app.dependency_overrides = original_overrides


@pytest.fixture(scope="session")
def test_client(yahoo_auth_manager):
    """
    Starts the app with a working YahooAuthManager and a dummy Redis / aiohttp
    session that the rest of the suite already expects.
    """
    app.state.yahoo_auth_manager = yahoo_auth_manager
    app.state.redis = MagicMock(name="redis")  # if anything still touches app.state.redis
    app.state.session = MagicMock(name="curl_session")
    with TestClient(app) as c:
        yield c


@pytest.fixture(scope="session")
def mock_request_context():
    """Mock request context"""
    with patch("src.dependencies.request_context", MagicMock()) as mock:
        mock.get.return_value.app.state.cookies = "mock_cookies"
        mock.get.return_value.app.state.crumb = "mock_crumb"
        yield mock


@pytest.fixture
def historical_quotes():
    """Load historical price data from a JSON file for testing and convert to HistoricalData objects"""
    data_path = Path(__file__).parent / "data" / "historical_quotes.json"
    with open(data_path) as file:
        raw_data = orjson.loads(file.read())

    # Convert each date entry to a HistoricalData object
    return {date: HistoricalData(**quote_data) for date, quote_data in raw_data.items()}


@pytest.fixture
def mock_websocket():
    """Fixture for mocking a WebSocket connection."""
    websocket = AsyncMock(spec=WebSocket)
    websocket.accept = AsyncMock()
    websocket.send_text = AsyncMock()
    websocket.send_json = AsyncMock()
    websocket.receive_text = AsyncMock()
    websocket.receive_json = AsyncMock()
    websocket.close = AsyncMock()
    websocket.client = MagicMock()
    websocket.client.host = "127.0.0.1"
    return websocket


@pytest.fixture
async def redis_connection_manager(mock_redis):
    with patch("redis.Redis", return_value=mock_redis):
        mock_redis.publish = MagicMock(return_value=None)
        manager = RedisConnectionManager(mock_redis)
        return manager


@pytest.fixture
def connection_manager():
    """Fixture for a ConnectionManager instance."""
    return ConnectionManager()


@pytest.fixture
def bypass_cache(monkeypatch):
    """
    Bypass the cache decorator for testing.
    """
    os.environ["BYPASS_CACHE"] = "true"
    yield
    del os.environ["BYPASS_CACHE"]
