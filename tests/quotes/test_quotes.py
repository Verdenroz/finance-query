import hashlib
from pathlib import Path
from unittest.mock import AsyncMock, patch, ANY

import pytest
import requests
from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from src.services.quotes.fetchers import (
    fetch_quotes,
    fetch_simple_quotes,
    scrape_quotes,
    scrape_simple_quotes,
)
from tests.conftest import VERSION

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
    "logo": "https://logo.clearbit.com/nvidia.com",
}

MOCK_SIMPLE_QUOTE_RESPONSE = {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "138.85",
    "afterHoursPrice": "138.46",
    "change": "+3.56",
    "percentChange": "+2.63%",
    "logo": "https://logo.clearbit.com/nvidia.com",
}


class TestQuotes:
    @pytest.fixture
    def quote_html(self):
        """
        Fixture that provides a function to get cached HTML content for URLs.
        If the HTML is not cached, it will fetch and cache it from the real URL.
        """
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "quotes"
        cache_dir.mkdir(parents=True, exist_ok=True)

        html_cache = {}

        def get_cached_html(url):
            if url in html_cache:
                return html_cache[url]

            filename = hashlib.md5(url.encode()).hexdigest()
            cache_file = cache_dir / f"{filename}.html"

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

    @pytest.fixture
    def mock_api_response(self):
        def _make():
            return {
                "quoteSummary": {
                    "result": [
                        {
                            "price": {
                                "symbol": "NVDA",
                                "longName": "NVIDIA Corporation",
                                "regularMarketPrice": {"fmt": "138.85"},
                                "regularMarketChange": {"fmt": "+3.56"},
                                "regularMarketChangePercent": {"raw": 0.0263},
                                "preMarketPrice": {"fmt": "138.46"},
                                "postMarketPrice": {"fmt": "138.46"},
                            },
                            "summaryDetail": {
                                "open": {"fmt": "136.41"},
                                "dayHigh": {"fmt": "139.20"},
                                "dayLow": {"fmt": "135.50"},
                                "fiftyTwoWeekHigh": {"fmt": "153.13"},
                                "fiftyTwoWeekLow": {"fmt": "66.25"},
                                "volume": {"raw": 194_448_420},
                                "averageVolume": {"raw": 244_250_595},
                                "marketCap": {"fmt": "3.4T"},
                                "beta": {"fmt": "1.62"},
                                "trailingPE": {"fmt": "54.88"},
                                "dividendRate": {"fmt": "0.04"},
                                "dividendYield": {"fmt": "0.03%"},
                                "exDividendDate": {"fmt": "Dec 05 2024"},
                            },
                            "calendarEvents": {
                                "earnings": {"earningsDate": [{"fmt": "2025-04-23"}]}
                            },
                            "assetProfile": {
                                "sector": "Technology",
                                "industry": "Semiconductors",
                                "longBusinessSummary": "NVIDIA Corporation provides graphics...",
                                "fullTimeEmployees": 29600,
                                "website": "https://www.nvidia.com",
                            },
                        }
                    ]
                }
            }

        return _make

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_fetch_quotes(self, mock_finance_client, mock_api_response, symbols, bypass_cache):
        """Test fetching quotes"""
        mock_finance_client.get_quote.return_value = mock_api_response()
        quotes = await fetch_quotes(mock_finance_client, symbols)
        assert all(isinstance(q, Quote) for q in quotes)
        assert mock_finance_client.get_quote.await_count == len(symbols)

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_scrape_quotes(self, quote_html, symbols, bypass_cache):
        """Test scraping quotes"""
        url = f"https://finance.yahoo.com/quote/{symbols[0]}/"
        html = quote_html(url)
        with patch("src.services.quotes.fetchers.quote_scraper.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html
            result = await scrape_quotes(symbols)
            assert isinstance(result, list)
            assert all(isinstance(q, Quote) for q in result)
            mock_fetch.assert_called()

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_fetch_simple_quotes(self, mock_finance_client, mock_api_response, symbols, bypass_cache):
        """Test fetching simple quotes"""
        batch = {"quoteResponse": {"result": []}}
        tpl = mock_api_response()["quoteSummary"]["result"][0]
        for s in symbols:
            batch["quoteResponse"]["result"].append(
                {
                    "symbol": s,
                    "longName": tpl["price"]["longName"],
                    "shortName": f"{s} Inc.",
                    "regularMarketPrice": tpl["price"]["regularMarketPrice"],
                    "regularMarketChange": tpl["price"]["regularMarketChange"],
                    "regularMarketChangePercent": tpl["price"]["regularMarketChangePercent"],
                    "preMarketPrice": tpl["price"]["preMarketPrice"],
                    "postMarketPrice": tpl["price"]["postMarketPrice"],
                }
            )
        mock_finance_client.get_simple_quotes.return_value = batch
        with patch("src.services.quotes.fetchers.quote_api.get_logo", new_callable=AsyncMock) as mock_logo:
            mock_logo.return_value = "https://logo.clearbit.com/example.com"
            quotes = await fetch_simple_quotes(mock_finance_client, symbols)
            assert all(isinstance(q, SimpleQuote) for q in quotes)
            assert len(quotes) == len(symbols)
            mock_finance_client.get_simple_quotes.assert_awaited_once_with(symbols)

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_scrape_simple_quotes(self, quote_html, symbols, bypass_cache):
        """Test scraping simple quotes"""
        url = f"https://finance.yahoo.com/quote/{symbols[0]}/"
        html = quote_html(url)
        with patch("src.services.quotes.fetchers.quote_scraper.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html
            result = await scrape_simple_quotes(symbols)
            assert isinstance(result, list)
            assert all(isinstance(q, SimpleQuote) for q in result)
            mock_fetch.assert_called()

    def test_get_quotes_success(self, test_client):
        """Test successful quote retrieval"""
        with patch("src.routes.quotes.get_quotes", new=AsyncMock(return_value=[Quote(**MOCK_QUOTE_RESPONSE)])) as svc:
            r = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
            assert r.status_code == 200
            assert r.json()[0]["symbol"] == "NVDA"
            svc.assert_awaited_once_with(ANY, ["NVDA"])

    def test_get_simple_quotes_success(self, test_client):
        """Test successful quote retrieval"""
        with patch("src.routes.quotes.get_simple_quotes", new=AsyncMock(return_value=[SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)])) as svc:
            r = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
            assert r.status_code == 200
            assert r.json()[0]["symbol"] == "NVDA"
            svc.assert_awaited_once_with(ANY, ["NVDA"])

    def test_scrape_quotes_fallback(self, test_client):
        """Test failure case when quotes cannot be fetched and fallback to scraping"""
        with patch("src.services.quotes.get_quotes.fetch_quotes", new=AsyncMock(side_effect=ValueError("bad creds"))) as fetch_mock, patch(
                "src.services.quotes.get_quotes.scrape_quotes", new=AsyncMock(return_value=[Quote(**MOCK_QUOTE_RESPONSE)])) as scrape_mock:
            r = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
            assert r.status_code == 200
            assert r.json()[0] == MOCK_QUOTE_RESPONSE
            fetch_mock.assert_awaited_once_with(ANY, ["NVDA"])
            scrape_mock.assert_awaited_once_with(["NVDA"])

    def test_scrape_simple_quotes_fallback(self, test_client):
        """Test failure case when simple quotes cannot be fetched and fallback to scraping"""
        with patch("src.services.quotes.get_quotes.fetch_simple_quotes", new=AsyncMock(side_effect=ValueError("bad creds"))) as fetch_mock, patch(
                "src.services.quotes.get_quotes.scrape_simple_quotes", new=AsyncMock(return_value=[SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)])) as scrape_mock:
            r = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
            assert r.status_code == 200
            assert r.json()[0] == MOCK_SIMPLE_QUOTE_RESPONSE
            fetch_mock.assert_awaited_once_with(ANY, ["NVDA"])
            scrape_mock.assert_awaited_once_with(["NVDA"])

    def test_unknown_symbol(self, test_client):
        """ Test for unknown symbols """
        with patch("src.services.quotes.get_quotes.fetch_quotes", new=AsyncMock(side_effect=HTTPException(404, "Symbol not found"))):
            r = test_client.get(f"{VERSION}/quotes?symbols=BAD")
            assert r.status_code == 404
        with patch("src.services.quotes.get_quotes.fetch_simple_quotes", new=AsyncMock(side_effect=HTTPException(404, "Symbol not found"))):
            r = test_client.get(f"{VERSION}/simple-quotes?symbols=BAD")
            assert r.status_code == 404
