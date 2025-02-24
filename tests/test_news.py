from unittest.mock import patch, AsyncMock

import pytest
from fastapi import HTTPException

from src.models import News
from src.services.news.get_news import (
    parse_symbol_exchange,
    scrape_news_for_quote,
    scrape_general_news,
)
from tests.test_utils import timeout, bypass_cache, mock_context

# Test data
MOCK_NEWS_HTML = """
<div class="news-item">
    <a href="https://example.com/news/1" target="_blank">
        <img src="https://example.com/image.jpg" alt="News Image">
    </a>
    <div class="news-content">
        <h3>
            <a href="https://example.com/news/1" target="_blank">Test News Title</a>
        </h3>
        <p>Test news description.</p>
        <div class="news-meta" title="Feb 24, 2025, 10:40 AM EST">37 minutes ago - Test Source</div>
    </div>
</div>
"""

MOCK_NEWS_RESPONSE = [
    {
        "title": "Test News Title",
        "link": "https://example.com/news/1",
        "source": "Test Source",
        "img": "https://example.com/image.jpg",
        "time": "12:00 PM"
    }
]

NEWS_TEST_TIMEOUT = 1


def test_parse_symbol_exchange():
    """Test symbol and exchange parsing"""
    # Test US symbol without exchange
    symbol, exchange = parse_symbol_exchange("AAPL")
    assert symbol == "AAPL"
    assert exchange is None

    # Test symbol with known exchange
    symbol, exchange = parse_symbol_exchange("SONY.T")
    assert symbol == "SONY"
    assert exchange == "TYO"

    # Test symbol with unknown exchange
    symbol, exchange = parse_symbol_exchange("TEST.UNKNOWN")
    assert symbol == "TEST"
    assert exchange is None

    # Test malformed symbol
    symbol, exchange = parse_symbol_exchange("")
    assert symbol == ""
    assert exchange is None


@timeout(NEWS_TEST_TIMEOUT)
async def test_scrape_news_for_quote_success(mock_context):
    """Test successful news scraping for a quote"""
    with bypass_cache('src.services.news.get_news.cache'):
        with patch('src.services.news.get_news.fetch', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.news.get_news._parse_news', new_callable=AsyncMock) as mock_parse:
                mock_fetch.return_value = MOCK_NEWS_HTML
                mock_parse.return_value = [News(**item) for item in MOCK_NEWS_RESPONSE]

                result = await scrape_news_for_quote("AAPL")

                assert len(result) == 1
                assert all(isinstance(item, News) for item in result)
                assert result[0].title == "Test News Title"
                assert result[0].source == "Test Source"
                mock_fetch.assert_awaited_once_with(
                    url='https://stockanalysis.com/stocks/AAPL'
                )


@timeout(NEWS_TEST_TIMEOUT)
async def test_scrape_news_for_quote_with_exchange(mock_context):
    """Test news scraping for a quote with exchange code"""
    with bypass_cache('src.services.news.get_news.cache'):
        with patch('src.services.news.get_news.fetch', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.news.get_news._parse_news', new_callable=AsyncMock) as mock_parse:
                mock_fetch.return_value = MOCK_NEWS_HTML
                mock_parse.return_value = [News(**item) for item in MOCK_NEWS_RESPONSE]

                result = await scrape_news_for_quote("SONY.T")

                assert len(result) == 1
                mock_fetch.assert_awaited_once_with(
                    url='https://stockanalysis.com/quote/tyo/SONY'
                )


@timeout(NEWS_TEST_TIMEOUT)
async def test_scrape_news_for_quote_failure(mock_context):
    """Test handling of failed news scraping for a quote"""
    with bypass_cache('src.services.news.get_news.cache'):
        with patch('src.services.news.get_news.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = Exception("Failed to fetch")

            with pytest.raises(HTTPException) as exc_info:
                await scrape_news_for_quote("INVALID")

            assert exc_info.value.status_code == 404
            assert "Could not find news" in str(exc_info.value.detail)


@timeout(NEWS_TEST_TIMEOUT)
async def test_scrape_general_news_success(mock_context):
    """Test successful general news scraping"""
    with bypass_cache('src.services.news.get_news.cache'):
        with patch('src.services.news.get_news.fetch', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.news.get_news._parse_news', new_callable=AsyncMock) as mock_parse:
                mock_fetch.return_value = MOCK_NEWS_HTML
                mock_parse.return_value = [News(**item) for item in MOCK_NEWS_RESPONSE]

                result = await scrape_general_news()

                assert len(result) == 1
                assert all(isinstance(item, News) for item in result)
                mock_fetch.assert_awaited_once_with(
                    url='https://stockanalysis.com/news/'
                )


@timeout(NEWS_TEST_TIMEOUT)
async def test_scrape_general_news_failure(mock_context):
    """Test handling of failed general news scraping"""
    with bypass_cache('src.services.news.get_news.cache'):
        with patch('src.services.news.get_news.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = ""  # Empty response
            with patch('src.services.news.get_news._parse_news', return_value=[]):
                with pytest.raises(HTTPException) as exc_info:
                    await scrape_general_news()

                assert exc_info.value.status_code == 404
                assert "Error fetching news" in str(exc_info.value.detail)


