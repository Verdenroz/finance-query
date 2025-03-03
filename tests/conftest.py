from unittest.mock import MagicMock, AsyncMock, patch

import pytest
from fastapi.testclient import TestClient

from src.main import app

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
    with patch('src.dependencies.request_context', MagicMock()) as mock:
        mock.get.return_value.app.state.cookies = "mock_cookies"
        mock.get.return_value.app.state.crumb = "mock_crumb"
        yield mock


@pytest.fixture(scope="session")
def mock_yahoo_auth(mock_request_context):
    """Mock Yahoo authentication data"""
    with patch('src.dependencies.get_auth_data', new_callable=AsyncMock) as mock:
        mock.return_value = ("mock_cookies", "mock_crumb")
        yield mock