from pathlib import Path
from unittest.mock import AsyncMock, patch

import pytest
import requests
from fastapi import HTTPException

from src.models import News
from src.services.news.get_news import parse_symbol_exchange, scrape_general_news, scrape_news_for_quote
from tests.conftest import VERSION

# Mock response data for news
MOCK_NEWS_RESPONSE = {
    "title": "Test News 1",
    "link": "https://example.com/news1",
    "source": "Test Source",
    "img": "https://example.com/image1.jpg",
    "time": "1 hour ago",
}

MOCK_SYMBOL_NEWS_RESPONSE = [
    News(
        title="Test News 1",
        link="https://example.com/news1",
        source="Test Source",
        img="https://example.com/image1.jpg",
        time="1 hour ago",
    ),
    News(
        title="Test News 2",
        link="https://example.com/news2",
        source="Test Source",
        img="https://example.com/image2.jpg",
        time="2 hours ago",
    ),
]


class TestNews:
    @pytest.fixture
    def news_html(self):
        """
        Fixture that provides a function to get cached HTML content for URLs.
        If the HTML is not cached, it will fetch and cache it from the real URL.
        """
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "news"
        cache_dir.mkdir(parents=True, exist_ok=True)

        html_cache = {}

        def get_cached_html(url, symbol="general"):
            if url in html_cache:
                return html_cache[url]

            cache_file = cache_dir / f"{symbol}.html"

            if cache_file.exists():
                with open(cache_file, encoding="utf-8") as f:
                    html_content = f.read()
            else:
                response = requests.get(url, headers={"User-Agent": "Mozilla/5.0"})
                html_content = response.text
                with open(cache_file, "w", encoding="utf-8") as f:
                    f.write(html_content)

            html_cache[url] = html_content
            return html_content

        yield get_cached_html
        # Cleanup on teardown
        for file in cache_dir.glob("*.html"):
            file.unlink()
        if cache_dir.exists():
            cache_dir.rmdir()

    @pytest.mark.parametrize(
        "symbol,expected_base,expected_exchange",
        [
            ("AAPL", "AAPL", None),
            ("AAPL.US", "AAPL", None),
            ("VOD.L", "VOD", "LON"),
            ("INVALID.XX", "INVALID", None),
            ("NVDA.TO", "NVDA", "TSX"),
        ],
    )
    def test_parse_symbol_exchange(self, symbol, expected_base, expected_exchange):
        """Test parse_symbol_exchange function with different symbols"""
        base_symbol, exchange = parse_symbol_exchange(symbol)
        assert base_symbol == expected_base
        assert exchange == expected_exchange

    @pytest.mark.parametrize(
        "symbol,test_url",
        [
            ("AAPL", "https://stockanalysis.com/stocks/AAPL"),
            ("MSFT", "https://stockanalysis.com/stocks/MSFT"),
            ("QQQ", "https://stockanalysis.com/etf/QQQ"),
            ("TQQQ", "https://stockanalysis.com/etf/TQQQ"),
            ("NVDA.TO", "https://stockanalysis.com/quote/tsx/NVDA"),  # should be verified from last test
        ],
    )
    async def test_scrape_news_for_quote(self, news_html, symbol, test_url, bypass_cache):
        """Test scrape_news_for_quote function with cached HTML content"""
        html_content = news_html(test_url, symbol=symbol)

        with patch("src.services.news.get_news.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content

            result = await scrape_news_for_quote(symbol)

            assert isinstance(result, list)
            assert all(isinstance(news, News) for news in result)
            assert len(result) > 0

            mock_fetch.assert_called()

    async def test_scrape_general_news(self, news_html, bypass_cache):
        """Test scrape_general_news function with cached HTML content"""
        test_url = "https://stockanalysis.com/news/"
        html_content = news_html(test_url)

        with patch("src.services.news.get_news.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content

            result = await scrape_general_news()

            assert isinstance(result, list)
            assert all(isinstance(news, News) for news in result)
            assert len(result) > 0

            mock_fetch.assert_called_once_with(url=test_url)

    async def test_scrape_news_invalid_symbol(self, bypass_cache):
        """Test scrape_news_for_quote with invalid symbol"""
        with pytest.raises(HTTPException) as exc_info:
            await scrape_news_for_quote("INVALID")
        assert exc_info.value.status_code == 404
        assert "Could not find news for the provided symbol" in str(exc_info.value.detail)

    def test_get_news_success(self, test_client, monkeypatch):
        """Test successful news retrieval"""
        mock_service = AsyncMock(return_value=MOCK_SYMBOL_NEWS_RESPONSE)
        monkeypatch.setattr("src.routes.finance_news.scrape_general_news", mock_service)

        response = test_client.get(f"{VERSION}/news")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == len(MOCK_SYMBOL_NEWS_RESPONSE)
        assert data[0]["title"] == MOCK_SYMBOL_NEWS_RESPONSE[0].title

        mock_service.assert_awaited_once()

    def test_get_symbol_news_success(self, test_client, monkeypatch):
        """Test successful symbol news retrieval"""
        mock_service = AsyncMock(return_value=MOCK_SYMBOL_NEWS_RESPONSE)
        monkeypatch.setattr("src.routes.finance_news.scrape_news_for_quote", mock_service)

        response = test_client.get(f"{VERSION}/news?symbol=AAPL")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == len(MOCK_SYMBOL_NEWS_RESPONSE)
        assert data[0]["title"] == MOCK_SYMBOL_NEWS_RESPONSE[0].title

        mock_service.assert_awaited_once_with("AAPL")

    def test_get_news_failure(self, test_client, monkeypatch):
        """Test failure case when news cannot be fetched"""
        mock_service = AsyncMock(side_effect=HTTPException(status_code=404, detail="Error fetching news"))
        monkeypatch.setattr("src.routes.finance_news.scrape_general_news", mock_service)

        response = test_client.get(f"{VERSION}/news")
        assert response.status_code == 404
