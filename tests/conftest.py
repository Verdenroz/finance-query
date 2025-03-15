from pathlib import Path
from unittest.mock import MagicMock, AsyncMock, patch

import pytest
from fastapi.testclient import TestClient
from orjson import orjson

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


@pytest.fixture
def historical_quotes():
    """Load historical price data from a JSON file for testing and convert to HistoricalData objects"""
    data_path = Path(__file__).parent / "data" / "historical_quotes.json"
    with open(data_path, "r") as file:
        raw_data = orjson.loads(file.read())

    # Convert each date entry to a HistoricalData object
    return {
        date: HistoricalData(**quote_data)
        for date, quote_data in raw_data.items()
    }


@pytest.fixture
def epoch_quotes():
    """Load historical price data with epoch timestamps from a JSON file for testing"""
    data_path = Path(__file__).parent / "data" / "epoch_quotes.json"
    with open(data_path, "r") as file:
        raw_data = orjson.loads(file.read())

    # Convert each date entry to a HistoricalData object
    return {
        date: HistoricalData(**quote_data)
        for date, quote_data in raw_data.items()
    }
