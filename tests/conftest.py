import functools
from pathlib import Path
from unittest.mock import MagicMock, AsyncMock, patch

import pytest
from fastapi.testclient import TestClient
from orjson import orjson
from starlette.websockets import WebSocket

from src.connections import RedisConnectionManager, ConnectionManager
from src.context import request_context
from src.main import app
from src.models import HistoricalData

VERSION = "v1"


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
def test_client(mock_redis, mock_session):
    """Create TestClient with mocked dependencies"""
    app.state.redis = mock_redis
    app.state.session = mock_session
    with TestClient(app) as client:
        yield client


@pytest.fixture(scope="session")
def mock_request_context():
    """Mock request context"""
    with patch("src.dependencies.request_context", MagicMock()) as mock:
        mock.get.return_value.app.state.cookies = "mock_cookies"
        mock.get.return_value.app.state.crumb = "mock_crumb"
        yield mock


@pytest.fixture(scope="session")
def mock_yahoo_auth(mock_request_context):
    """Mock Yahoo authentication data"""
    with patch("src.dependencies.get_auth_data", new_callable=AsyncMock) as mock:
        mock.return_value = ("mock_cookies", "mock_crumb")
        yield mock


@pytest.fixture
def historical_quotes():
    """Load historical price data from a JSON file for testing and convert to HistoricalData objects"""
    data_path = Path(__file__).parent / "data" / "historical_quotes.json"
    with open(data_path, "r") as file:
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
    Fixture to bypass both Redis and in-memory caching.
    Works regardless of whether REDIS_URL is set or not.
    """
    mock_request = MagicMock()
    mock_redis = MagicMock()
    mock_redis.exists.return_value = False  # Always return cache miss
    mock_request.app.state.redis = mock_redis

    # Set the context variable
    token = request_context.set(mock_request)

    # Define a pass-through decorator to replace cache
    def bypass_decorator(*args, **kwargs):
        def decorator(func):
            @functools.wraps(func)
            async def wrapper(*fn_args, **fn_kwargs):
                return await func(*fn_args, **fn_kwargs)

            return wrapper

        return decorator

    # Patch the cache decorator in the src.cache module
    monkeypatch.setattr("src.cache.cache", bypass_decorator)

    # Clean up the context variable after the test
    yield

    try:
        request_context.reset(token)
    except ValueError:
        # Context may already be reset
        pass
