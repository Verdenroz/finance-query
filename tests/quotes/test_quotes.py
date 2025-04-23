import hashlib
from pathlib import Path
from unittest.mock import AsyncMock, patch

import orjson
import pytest
import requests
from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from src.services.quotes.fetchers import fetch_quotes, scrape_quotes, fetch_simple_quotes, scrape_simple_quotes
from tests.conftest import VERSION

# Mock response data for quotes
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
                with open(cache_file, 'r', encoding='utf-8') as f:
                    html_content = f.read()
            else:
                response = requests.get(url, headers={"User-Agent": "Mozilla/5.0"})
                html_content = response.text
                with open(cache_file, 'w', encoding='utf-8') as f:
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
        """
        Fixture that provides mock Yahoo Finance API responses for quotes.
        """
        def get_mock_response():
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
                                "postMarketPrice": {"fmt": "138.46"}
                            },
                            "summaryDetail": {
                                "open": {"fmt": "136.41"},
                                "dayHigh": {"fmt": "139.20"},
                                "dayLow": {"fmt": "135.50"},
                                "fiftyTwoWeekHigh": {"fmt": "153.13"},
                                "fiftyTwoWeekLow": {"fmt": "66.25"},
                                "volume": {"raw": 194448420},
                                "averageVolume": {"raw": 244250595},
                                "marketCap": {"fmt": "3.4T"},
                                "beta": {"fmt": "1.62"},
                                "trailingPE": {"fmt": "54.88"},
                                "dividendRate": {"fmt": "0.04"},
                                "dividendYield": {"fmt": "0.03%"},
                                "exDividendDate": {"fmt": "Dec 05, 2024"}
                            },
                            "calendarEvents": {
                                "earnings": {
                                    "earningsDate": [{"fmt": "2025-04-23"}]
                                }
                            },
                            "defaultKeyStatistics": {
                                "trailingEps": {"fmt": "2.53"},
                                "annualReportExpenseRatio": {"fmt": "0.04"},
                                "morningStarOverallRating": {"raw": 5},
                                "morningStarRiskRating": {"raw": 3},
                                "annualHoldingsTurnover": {"fmt": "10%"},
                                "lastCapGain": {"fmt": "0.01"},
                                "fundInceptionDate": {"raw": 946684800}
                            },
                            "assetProfile": {
                                "sector": "Technology",
                                "industry": "Semiconductors",
                                "longBusinessSummary": "NVIDIA Corporation provides graphics...",
                                "fullTimeEmployees": 29600,
                                "website": "https://www.nvidia.com"
                            },
                            "quoteUnadjustedPerformanceOverview": {
                                "performanceOverview": {
                                    "fiveDaysReturn": {"fmt": "1.5%"},
                                    "oneMonthReturn": {"fmt": "3.0%"},
                                    "threeMonthReturn": {"fmt": "5.0%"},
                                    "sixMonthReturn": {"fmt": "10.0%"},
                                    "ytdReturnPct": {"fmt": "15.0%"},
                                    "oneYearTotalReturn": {"fmt": "20.0%"},
                                    "threeYearTotalReturn": {"fmt": "30.0%"},
                                    "fiveYearTotalReturn": {"fmt": "50.0%"},
                                    "tenYearTotalReturn": {"fmt": "100.0%"},
                                    "maxReturn": {"fmt": "200.0%"}
                                }
                            }
                        }
                    ]
                }
            }

        return get_mock_response

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_fetch_quotes(self, mock_api_response, symbols, bypass_cache):
        """Test fetch_quotes function with mocked API response"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"

        mock_response = mock_api_response()

        class MockResponse:
            def __init__(self, json_data, status_code):
                self._json_data = json_data
                self.status = status_code

            async def text(self):
                return orjson.dumps(self._json_data).decode('utf-8')

        with patch('src.services.quotes.fetchers.quote_api.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = MockResponse(mock_response, 200)

            result = await fetch_quotes(symbols, test_cookies, test_crumb)

            assert isinstance(result, list)
            assert all(isinstance(quote, Quote) for quote in result)

            mock_fetch.assert_called()

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_scrape_quotes(self, quote_html, symbols, bypass_cache):
        """Test scrape_quotes function with cached HTML content"""
        test_url = f"https://finance.yahoo.com/quote/{symbols[0]}/"

        html_content = quote_html(test_url)

        with patch('src.services.quotes.fetchers.quote_scraper.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content

            result = await scrape_quotes(symbols)

            assert isinstance(result, list)
            assert all(isinstance(quote, Quote) for quote in result)

            mock_fetch.assert_called()

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_fetch_simple_quotes(self, mock_api_response, symbols, bypass_cache):
        """Test fetch_simple_quotes function with mocked API response"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"

        mock_response = mock_api_response()

        class MockResponse:
            def __init__(self, json_data, status_code):
                self._json_data = json_data
                self.status = status_code

            async def text(self):
                return orjson.dumps(self._json_data).decode('utf-8')

        with patch('src.services.quotes.fetchers.quote_api.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = MockResponse(mock_response, 200)

            result = await fetch_simple_quotes(symbols, test_cookies, test_crumb)

            assert isinstance(result, list)
            assert all(isinstance(quote, SimpleQuote) for quote in result)

            mock_fetch.assert_called()

    @pytest.mark.parametrize("symbols", [["NVDA"], ["AAPL", "MSFT"]])
    async def test_scrape_simple_quotes(self, quote_html, symbols, bypass_cache):
        """Test scrape_simple_quotes function with cached HTML content"""
        test_url = f"https://finance.yahoo.com/quote/{symbols[0]}/"

        html_content = quote_html(test_url)

        with patch('src.services.quotes.fetchers.quote_scraper.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content

            result = await scrape_simple_quotes(symbols)

            assert isinstance(result, list)
            assert all(isinstance(quote, SimpleQuote) for quote in result)

            mock_fetch.assert_called()

    def test_get_quotes_success(self, test_client, mock_api_response, mock_yahoo_auth, monkeypatch):
        """Test successful quote retrieval"""
        mock_service = AsyncMock(return_value=[Quote(**MOCK_QUOTE_RESPONSE)])
        monkeypatch.setattr("src.routes.quotes.get_quotes", mock_service)

        response = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0] == MOCK_QUOTE_RESPONSE

        mock_service.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")

    def test_get_simple_quotes_success(self, test_client, mock_api_response, mock_yahoo_auth, monkeypatch):
        """Test successful simple quote retrieval"""
        mock_service = AsyncMock(return_value=[SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)])
        monkeypatch.setattr("src.routes.quotes.get_simple_quotes", mock_service)

        response = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0] == MOCK_SIMPLE_QUOTE_RESPONSE

        mock_service.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")

    def test_scrape_quotes_fallback(self, test_client, mock_yahoo_auth):
        """Test failure case when quotes cannot be fetched and fallback to scraping"""
        with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as fetch_mock, \
                patch('src.services.quotes.get_quotes.scrape_quotes', new_callable=AsyncMock) as scrape_mock:
            fetch_mock.side_effect = ValueError("Error with Yahoo Finance credentials")
            scrape_mock.return_value = [Quote(**MOCK_QUOTE_RESPONSE)]

            response = test_client.get(f"{VERSION}/quotes?symbols=NVDA")
            data = response.json()

            assert response.status_code == 200
            assert len(data) == 1
            assert data[0] == MOCK_QUOTE_RESPONSE

            fetch_mock.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")
            scrape_mock.assert_awaited_once_with(["NVDA"])

    def test_scrape_simple_quotes_fallback(self, test_client, mock_yahoo_auth):
        """Test failure case when simple quotes cannot be fetched and fallback to scraping"""
        with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as fetch_mock, \
                patch('src.services.quotes.get_quotes.scrape_simple_quotes', new_callable=AsyncMock) as scrape_mock:
            fetch_mock.side_effect = ValueError("Error with Yahoo Finance credentials")
            scrape_mock.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]

            response = test_client.get(f"{VERSION}/simple-quotes?symbols=NVDA")
            data = response.json()

            assert response.status_code == 200
            assert len(data) == 1
            assert data[0] == MOCK_SIMPLE_QUOTE_RESPONSE

            fetch_mock.assert_awaited_once_with(["NVDA"], "mock_cookies", "mock_crumb")
            scrape_mock.assert_awaited_once_with(["NVDA"])

    def test_invalid_symbol(self, test_client, mock_yahoo_auth):
        """Test invalid symbol"""
        with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as fetch_mock:
            fetch_mock.side_effect = HTTPException(404, "Symbol not found")
            response = test_client.get(f"{VERSION}/quotes?symbols=INVALID")
            assert response.status_code == 404

        with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as fetch_mock:
            fetch_mock.side_effect = HTTPException(404, "Symbol not found")
            response = test_client.get(f"{VERSION}/simple-quotes?symbols=INVALID")
            assert response.status_code == 404
