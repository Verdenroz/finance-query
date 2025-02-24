from unittest.mock import patch

import pytest

from src.models import News
from tests.conftest import VERSION

# Test data
sample_news = [
    News(
        title="Test News 1",
        link="https://example.com/news1",
        source="Test Source",
        img="https://example.com/image1.jpg",
        time="1 hour ago"
    ),
    News(
        title="Test News 2",
        link="https://example.com/news2",
        source="Test Source",
        img="https://example.com/image2.jpg",
        time="2 hours ago"
    )
]


@pytest.fixture
def mock_scrape_general_news():
    with patch('src.routes.finance_news.scrape_general_news') as mock:
        mock.return_value = sample_news
        yield mock


@pytest.fixture
def mock_scrape_news_for_quote():
    with patch('src.routes.finance_news.scrape_news_for_quote') as mock:
        mock.return_value = sample_news
        yield mock


def test_get_general_news_success(test_client, mock_scrape_general_news):
    """Test successful retrieval of general news"""
    response = test_client.get(f"{VERSION}/news")

    mock_scrape_general_news.assert_awaited_once()
    assert response.status_code == 200
    assert len(response.json()) == 2

    news_items = response.json()
    assert news_items[0]["title"] == "Test News 1"
    assert news_items[1]["title"] == "Test News 2"

    # Verify the response matches our News model structure
    for item in news_items:
        assert all(key in item for key in ["title", "link", "source", "img", "time"])

    # Verify the data types of the response
    for item in news_items:
        assert isinstance(item["title"], str)
        assert isinstance(item["link"], str)
        assert isinstance(item["source"], str)
        assert isinstance(item["img"], str)
        assert isinstance(item["time"], str)


def test_get_general_news_failure(test_client, mock_scrape_general_news):
    """Test failure case when general news cannot be fetched"""
    mock_scrape_general_news.side_effect = Exception("Failed to fetch news")
    response = test_client.get(f"{VERSION}/news")
    assert response.status_code == 404


def test_get_symbol_news_success(test_client, mock_scrape_news_for_quote):
    """Test successful retrieval of news for a specific symbol"""
    response = test_client.get(f"{VERSION}/news?symbol=AAPL")

    mock_scrape_news_for_quote.assert_awaited_once_with("AAPL")
    assert response.status_code == 200
    assert len(response.json()) == 2

    news_items = response.json()
    assert news_items[0]["title"] == "Test News 1"
    assert news_items[1]["title"] == "Test News 2"


def test_get_symbol_news_not_found(test_client, mock_scrape_news_for_quote):
    """Test case when no news is found for a symbol"""
    mock_scrape_news_for_quote.side_effect = Exception("Could not find news for the provided symbol")
    response = test_client.get(f"{VERSION}/news?symbol=INVALID")
    assert response.status_code == 404
    assert response.json()["detail"] == "Could not find news for the provided symbol"


def test_get_symbol_news_with_exchange(test_client, mock_scrape_news_for_quote):
    """Test news retrieval for a symbol with exchange code"""
    with patch('src.services.news.get_news.parse_symbol_exchange') as mock_parse:
        mock_parse.return_value = ('300750', 'SHE')
        with patch('src.services.news.get_news.scrape_news_for_quote') as mock_scrape:
            mock_scrape.return_value = sample_news
            response = test_client.get(f"{VERSION}/news?symbol=300750.SZ")

            mock_scrape_news_for_quote.assert_called_once_with('300750.SZ')
            assert response.status_code == 200
            assert len(response.json()) == 2


def test_invalid_symbol_format(test_client, mock_scrape_general_news):
    """Test with invalid symbol format"""
    with patch('src.services.news.get_news.scrape_news_for_quote') as mock_scrape:
        response = test_client.get(f"{VERSION}/news?symbol=")

        # Assert general news was called instead of symbol news
        mock_scrape_general_news.assert_awaited_once()
        mock_scrape.assert_not_awaited()

        assert response.status_code == 200
        assert len(response.json()) == 2
