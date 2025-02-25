from unittest.mock import patch, AsyncMock

import pytest
from fastapi import HTTPException

from tests.conftest import VERSION

# Sample test data
MOCK_SYMBOL = "NVDA"
MOCK_SIMILAR_QUOTES = [
    {
        "symbol": "AMD",
        "name": "Advanced Micro Devices, Inc.",
        "price": "108.11",
        "afterHoursPrice": "108.35",
        "change": "-2.73",
        "percentChange": "-2.46%",
        "logo": "https://logo.clearbit.com/https://www.amd.com"
    },
    {
        "symbol": "TSLA",
        "name": "Tesla, Inc.",
        "price": "330.53",
        "afterHoursPrice": "326.85",
        "change": "-7.27",
        "percentChange": "-2.15%",
        "logo": "https://logo.clearbit.com/https://www.tesla.com"
    },
    {
        "symbol": "META",
        "name": "Meta Platforms, Inc.",
        "price": "668.13",
        "afterHoursPrice": "666.05",
        "change": "-15.42",
        "percentChange": "-2.26%"
    }
]


@pytest.fixture()
def mock_get_similar_quotes():
    with patch("src.routes.similar.get_similar_quotes", new_callable=AsyncMock) as mock:
        mock.return_value = MOCK_SIMILAR_QUOTES
        yield mock


def test_similar_quotes_success(test_client, mock_get_similar_quotes, mock_yahoo_auth):
    """Test successful retrieval of similar quotes."""
    response = test_client.get(f"{VERSION}/similar?symbol={MOCK_SYMBOL}")

    assert response.status_code == 200
    assert response.json() == MOCK_SIMILAR_QUOTES
    mock_get_similar_quotes.assert_called_once_with(MOCK_SYMBOL.upper(), "mock_cookies", "mock_crumb", 10)


def test_similar_quotes_with_limit(test_client, mock_get_similar_quotes, mock_yahoo_auth):
    """Test similar quotes with custom limit parameter."""
    mock_get_similar_quotes.return_value = MOCK_SIMILAR_QUOTES[:1]

    response = test_client.get(f"{VERSION}/similar?symbol={MOCK_SYMBOL}&limit=1")

    assert response.status_code == 200
    assert len(response.json()) == 1
    mock_get_similar_quotes.assert_called_once_with(MOCK_SYMBOL.upper(), "mock_cookies", "mock_crumb", 1)


def test_similar_quotes_invalid_limit(test_client, mock_yahoo_auth):
    """Test with invalid limit parameter."""
    # Test with limit below minimum
    response = test_client.get(f"{VERSION}/similar?symbol={MOCK_SYMBOL}&limit=0")
    assert response.status_code == 422

    # Test with limit above maximum
    response = test_client.get(f"{VERSION}/similar?symbol={MOCK_SYMBOL}&limit=21")
    assert response.status_code == 422


def test_similar_quotes_not_found(test_client, mock_yahoo_auth):
    """Test when no similar quotes are found."""
    with patch("src.services.similar.get_similar_quotes.fetch_similar", new_callable=AsyncMock) as mock_get_similar:
        mock_get_similar.side_effect = HTTPException(status_code=404,
                                                     detail="No similar stocks found or invalid symbol")

        response = test_client.get(f"{VERSION}/similar?symbol=INVALID")

        assert response.status_code == 404
        assert response.json() == {"detail": "No similar stocks found or invalid symbol"}
