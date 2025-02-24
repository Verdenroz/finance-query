from unittest.mock import AsyncMock, patch

import pytest
from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from tests.conftest import VERSION

# Test data
MOCK_QUOTE_RESPONSE = {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "138.85",
    "afterHoursPrice": "138.46",
    "change": "+3.56",
    "percentChange": "+2.63%",
    "open": "136.41",
    "high": "139.20",
    "low": "135.50",
    "yearHigh": "153.13",
    "yearLow": "66.25",
    "volume": 194448420,
    "avgVolume": 244250595,
    "marketCap": "3.4T",
    "beta": "1.62",
    "pe": "54.88",
    "dividend": "0.04",
    "yield": "0.03%",
    "exDividend": "Dec 05, 2024",
    "earningsDate": "Feb 26, 2025",
    "lastDividend": "0.01",
    "sector": "Technology",
    "industry": "Semiconductors",
    "about": "NVIDIA Corporation provides graphics...",
    "employees": "29600",
    "logo": "https://logo.clearbit.com/nvidia.com"
}

MOCK_SIMPLE_QUOTE_RESPONSE = {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "138.85",
    "afterHoursPrice": "138.46",
    "change": "+3.56",
    "percentChange": "+2.63%",
    "logo": "https://logo.clearbit.com/nvidia.com"
}


@pytest.fixture
def mock_get_quotes():
    with patch('src.routes.quotes.get_quotes', new_callable=AsyncMock) as mock:
        mock.return_value = [Quote(**MOCK_QUOTE_RESPONSE)]
        yield mock


@pytest.fixture
def mock_get_simple_quotes():
    with patch('src.routes.quotes.get_simple_quotes', new_callable=AsyncMock) as mock:
        mock.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]
        yield mock


def test_get_quotes_success(test_client, mock_get_quotes, mock_yahoo_auth):
    """Test successful quote retrieval"""
    response = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
    data = response.json()

    assert response.status_code == 200
    assert len(data) == 1
    assert data[0] == MOCK_QUOTE_RESPONSE

    # Verify mock was called with correct arguments
    mock_get_quotes.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")


def test_get_quotes_multiple_symbols(test_client, mock_get_quotes, mock_yahoo_auth):
    """Test quote retrieval for multiple symbols"""
    response = test_client.get(f"{VERSION}/quotes?symbols=NVDA, NVDA, nvda, AAPL, MSFT, msft")

    assert response.status_code == 200
    mock_get_quotes.assert_awaited_once()
    args = mock_get_quotes.await_args[0]
    assert set(args[0]) == {"NVDA", "AAPL", "MSFT"}  # symbols should be unique and uppercase


def test_get_simple_quotes_success(test_client, mock_get_simple_quotes, mock_yahoo_auth):
    """Test successful simple quote retrieval"""
    response = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
    data = response.json()

    assert response.status_code == 200
    assert len(data) == 1
    assert data[0] == MOCK_SIMPLE_QUOTE_RESPONSE

    # Verify mock was called with correct arguments
    mock_get_simple_quotes.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")


def test_get_simple_quotes_multiple_symbols(test_client, mock_get_simple_quotes, mock_yahoo_auth):
    """Test simple quote retrieval for multiple symbols"""
    response = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA, NVDA, nvda, AAPL, MSFT, msft")

    assert response.status_code == 200
    mock_get_simple_quotes.assert_awaited_once()
    args = mock_get_simple_quotes.await_args[0]
    assert set(args[0]) == {"NVDA", "AAPL", "MSFT"}  # symbols should be unique and uppercase


def test_quotes_symbol_validation(test_client):
    """Test symbol validation"""
    # Test empty symbol
    response = test_client.get(f"{VERSION}/quotes")
    assert response.status_code == 422

    # Test simple quotes with empty symbol
    response = test_client.get(f"{VERSION}/simple-quotes")
    assert response.status_code == 422


def test_scrape_quotes_fallback(test_client, mock_yahoo_auth):
    """Test failure case when quotes cannot be fetched and fallback to scraping"""
    with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as fetch_mock, \
            patch('src.services.quotes.get_quotes.scrape_quotes', new_callable=AsyncMock) as scrape_mock:
        fetch_mock.side_effect = Exception("Error fetching quotes")
        scrape_mock.return_value = [Quote(**MOCK_QUOTE_RESPONSE)]

        response = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0] == MOCK_QUOTE_RESPONSE

        # Verify fetch_quotes was called and then scrape_quotes was called as a fallback
        fetch_mock.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")
        scrape_mock.assert_awaited_once_with(["NVDA"])


def test_scrape_simple_quotes_fallback(test_client, mock_yahoo_auth):
    """Test failure case when simple quotes cannot be fetched and fallback to scraping"""
    with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as fetch_mock, \
            patch('src.services.quotes.get_quotes.scrape_simple_quotes', new_callable=AsyncMock) as scrape_mock:
        fetch_mock.side_effect = Exception("Error fetching quotes")
        scrape_mock.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]

        response = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0] == MOCK_SIMPLE_QUOTE_RESPONSE

        # Verify fetch_simple_quotes was called and then scrape_simple_quotes was called as a fallback
        fetch_mock.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")
        scrape_mock.assert_awaited_once_with(["NVDA"])


def test_invalid_symbol(test_client, mock_yahoo_auth):
    """Test invalid symbol"""
    with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as fetch_mock:
        fetch_mock.side_effect = HTTPException(404, "Symbol not found")
        response = test_client.get(f"{VERSION}/quotes?symbols=INVALID")
        assert response.status_code == 404

    with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as fetch_mock:
        fetch_mock.side_effect = HTTPException(404, "Symbol not found")
        response = test_client.get(f"{VERSION}/simple-quotes?symbols=INVALID")
        assert response.status_code == 404
