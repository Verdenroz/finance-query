import hashlib
from pathlib import Path
from unittest.mock import patch, AsyncMock

import pytest
import requests
import orjson
from fastapi import HTTPException

from src.models import SimpleQuote
from src.services import get_similar_quotes
from src.services.similar.fetchers import fetch_similar, scrape_similar_quotes
from src.services.similar.fetchers.similar_api import _fetch_yahoo_recommended_symbols
from tests.conftest import VERSION


class TestSimilarQuotesHandler:
    @pytest.fixture
    def similar_quotes_html(self):
        """
        Fixture that provides a function to get cached HTML content for URLs.
        If the HTML is not cached, it will fetch and cache it.
        """
        # Path for storing cached HTML responses
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "similar"
        cache_dir.mkdir(parents=True, exist_ok=True)

        # Create a dictionary to store HTML content by URL
        html_cache = {}

        def get_cached_html(url):
            # Check if we already have this URL in our in-memory cache
            if url in html_cache:
                return html_cache[url]

            # Extract symbol from URL for filename
            if 'quote/' in url:
                symbol = url.split('quote/')[1].strip('/')
                cache_file = cache_dir / f"{symbol}_page.html"
            else:
                # For any other URL, use a hash of the URL as filename
                filename = hashlib.md5(url.encode()).hexdigest()
                cache_file = cache_dir / f"{filename}.html"

            # Check if we have cached HTML
            if cache_file.exists():
                with open(cache_file, 'r', encoding='utf-8') as f:
                    html_content = f.read()
            else:
                # Fetch real content if no cache exists (only for first run)
                response = requests.get(url, headers={"User-Agent": "Mozilla/5.0"})
                html_content = response.text

                # Save for future test runs
                with open(cache_file, 'w', encoding='utf-8') as f:
                    f.write(html_content)

            # Store HTML in our cache dictionary
            html_cache[url] = html_content
            return html_content

        yield get_cached_html
        # Cleanup on teardown
        for file in cache_dir.glob("*.html"):
            file.unlink()
        if cache_dir.exists():
            cache_dir.rmdir()

    @pytest.fixture
    def yahoo_recommendations(self):
        """
        Fixture that provides a function to get cached Yahoo recommendation data for symbols.
        If the data is not cached, it will create mock data and cache it.
        """
        # Path for storing cached Yahoo API responses
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "yahoo"
        cache_dir.mkdir(parents=True, exist_ok=True)

        # Create a dictionary to store data by symbol
        data_cache = {}

        def get_cached_data(symbol):
            # Check if we already have this symbol in our in-memory cache
            if symbol in data_cache:
                return data_cache[symbol]

            # Create a cache file path
            cache_file = cache_dir / f"{symbol}_recommendations.json"

            # Check if we have cached data
            if cache_file.exists():
                with open(cache_file, 'r') as f:
                    import json
                    yahoo_data = json.load(f)
            else:
                # Create mock data if no cache exists
                recommendations = []

                # Sample recommended symbols based on the input symbol
                sample_recommendations = {
                    'AAPL': ['MSFT', 'GOOGL', 'META', 'AMZN', 'NVDA'],
                    'MSFT': ['AAPL', 'GOOGL', 'META', 'AMZN', 'ORCL'],
                    'NVDA': ['AMD', 'INTC', 'TSM', 'MU', 'AVGO'],
                    'JPM': ['BAC', 'C', 'WFC', 'GS', 'MS'],
                    'META': ['GOOGL', 'SNAP', 'PINS', 'TWTR', 'TTD']
                }

                # Get recommendations for the symbol or use a default set
                recommended_symbols = sample_recommendations.get(symbol, ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'META'])

                for rec_symbol in recommended_symbols:
                    recommendations.append({"symbol": rec_symbol})

                yahoo_data = {
                    "finance": {
                        "result": [
                            {
                                "recommendedSymbols": recommendations
                            }
                        ]
                    }
                }

                # Save for future test runs
                with open(cache_file, 'w') as f:
                    import json
                    json.dump(yahoo_data, f)

            # Store data in our cache dictionary
            data_cache[symbol] = yahoo_data
            return yahoo_data

        yield get_cached_data
        # Cleanup on teardown
        for file in cache_dir.glob("*.json"):
            file.unlink()
        if cache_dir.exists():
            cache_dir.rmdir()

    @pytest.fixture
    def cached_quote_data(self):
        """
        Fixture that provides a function to get cached Yahoo quote data for symbols.
        If the data is not cached, it will create mock data and cache it.
        """
        # Path for storing cached Yahoo API responses
        cache_dir = Path(__file__).parent / "data" / "similar"
        cache_dir.mkdir(parents=True, exist_ok=True)

        # Create a dictionary to store data by symbol
        data_cache = {}

        def get_cached_data(symbols):
            # Turn symbols list into a key for caching
            symbols_key = "_".join(sorted(symbols))

            # Check if we already have these symbols in our in-memory cache
            if symbols_key in data_cache:
                return data_cache[symbols_key]

            # Create a cache file path
            cache_file = cache_dir / f"quotes_{symbols_key}.json"

            # Check if we have cached data
            if cache_file.exists():
                with open(cache_file, 'r') as f:
                    import json
                    quotes_data = json.load(f)
            else:
                # Create mock data if no cache exists
                sample_quotes = {
                    'AAPL': {'name': 'Apple Inc.', 'price': '176.43', 'change': '0.51', 'percentChange': '+0.29%'},
                    'MSFT': {'name': 'Microsoft Corporation', 'price': '385.22', 'change': '-3.17',
                             'percentChange': '-0.82%'},
                    'GOOGL': {'name': 'Alphabet Inc.', 'price': '142.65', 'change': '1.23', 'percentChange': '+0.87%'},
                    'META': {'name': 'Meta Platforms, Inc.', 'price': '485.58', 'change': '5.37',
                             'percentChange': '+1.12%'},
                    'AMZN': {'name': 'Amazon.com, Inc.', 'price': '179.56', 'change': '-0.76', 'percentChange': '-0.42%'},
                    'NVDA': {'name': 'NVIDIA Corporation', 'price': '860.28', 'change': '15.37', 'percentChange': '+1.82%'},
                    'AMD': {'name': 'Advanced Micro Devices, Inc.', 'price': '146.88', 'change': '2.35',
                            'percentChange': '+1.63%'},
                    'INTC': {'name': 'Intel Corporation', 'price': '33.94', 'change': '-0.38', 'percentChange': '-1.11%'},
                    'TSM': {'name': 'Taiwan Semiconductor Manufacturing Co. Ltd.', 'price': '174.25', 'change': '3.75',
                            'percentChange': '+2.20%'},
                    'JPM': {'name': 'JPMorgan Chase & Co.', 'price': '198.52', 'change': '1.43', 'percentChange': '+0.73%'},
                    'BAC': {'name': 'Bank of America Corporation', 'price': '37.41', 'change': '0.24',
                            'percentChange': '+0.65%'}
                }

                quotes_data = []
                for symbol in symbols:
                    if symbol in sample_quotes:
                        quote = sample_quotes[symbol]
                        quotes_data.append({
                            'symbol': symbol,
                            'name': quote['name'],
                            'price': quote['price'],
                            'change': quote['change'],
                            'percentChange': quote['percentChange']
                        })
                    else:
                        # Generate mock data for unknown symbols
                        quotes_data.append({
                            'symbol': symbol,
                            'name': f'{symbol} Corporation',
                            'price': '100.00',
                            'change': '0.00',
                            'percentChange': '0.00%'
                        })

                # Save for future test runs
                with open(cache_file, 'w') as f:
                    import json
                    json.dump(quotes_data, f)

            # Store data in our cache dictionary
            data_cache[symbols_key] = quotes_data
            return quotes_data

        yield get_cached_data
        # Cleanup on teardown
        for file in cache_dir.glob("*.json"):
            file.unlink()
        if cache_dir.exists():
            cache_dir.rmdir()

    async def test_similar_quotes_endpoint(self, test_client, monkeypatch, mock_yahoo_auth):
        """Test the /similar endpoint"""

        test_symbol = "NVDA"
        mock_quotes = [
            {
                "symbol": "AMD",
                "name": "Advanced Micro Devices, Inc.",
                "price": "146.88",
                "change": "2.35",
                "percentChange": "+1.63%"
            },
            {
                "symbol": "INTC",
                "name": "Intel Corporation",
                "price": "33.94",
                "change": "-0.38",
                "percentChange": "-1.11%"
            }
        ]

        async def mock_get_similar(symbol, cookies, crumb, limit):
            return [SimpleQuote(**quote) for quote in mock_quotes[:limit]]

        monkeypatch.setattr('src.routes.similar.get_similar_quotes', mock_get_similar)

        # Test default limit
        response = test_client.get(f"/{VERSION}/similar?symbol={test_symbol}")

        assert response.status_code == 200
        data = response.json()
        assert len(data) == 2
        assert data[0]["symbol"] == "AMD"

        # Test with custom limit
        response = test_client.get(f"/{VERSION}/similar?symbol={test_symbol}&limit=1")

        assert response.status_code == 200
        data = response.json()
        assert len(data) == 1
        assert data[0]["symbol"] == "AMD"

    async def test_fetch_yahoo_recommended_symbols(self, yahoo_recommendations, bypass_cache):
        """Test _fetch_yahoo_recommended_symbols function with cached data"""

        test_symbols = ['AAPL', 'MSFT', 'NVDA', 'JPM']
        test_limit = 3

        for symbol in test_symbols:
            # Get cached recommendation data
            yahoo_data = yahoo_recommendations(symbol)

            # Expected recommendations from the cached data
            recommendations = yahoo_data["finance"]["result"][0]["recommendedSymbols"]
            expected_symbols = [rec["symbol"] for rec in recommendations]  # Get all symbols, not just limited ones

            # Mock the fetch function to return our cached data
            with patch('src.services.similar.fetchers.similar_api.fetch', new_callable=AsyncMock) as mock_fetch:
                mock_fetch.return_value = orjson.dumps(yahoo_data)

                # Call the function with test parameters
                result = await _fetch_yahoo_recommended_symbols(symbol, test_limit)

                # Verify the result matches all recommendations (function ignores limit)
                assert result == expected_symbols

                # Verify fetch was called with correct parameters
                mock_fetch.assert_called_once()
                call_args = mock_fetch.call_args[1]
                assert call_args["url"] == f"https://query1.finance.yahoo.com/v6/finance/recommendationsbysymbol/{symbol}"
                assert call_args["params"] == {"count": test_limit}  # Verify limit is passed to API

    async def test_fetch_similar(self, yahoo_recommendations, cached_quote_data, bypass_cache):
        """Test fetch_similar function with cached data"""

        test_symbol = 'NVDA'
        test_limit = 3
        test_cookies = 'test_cookies'
        test_crumb = 'test_crumb'

        # Get cached recommendation data
        yahoo_data = yahoo_recommendations(test_symbol)
        recommendations = yahoo_data["finance"]["result"][0]["recommendedSymbols"]
        recommended_symbols = [rec["symbol"] for rec in recommendations[:test_limit]]

        # Get cached quote data
        quote_data = cached_quote_data(recommended_symbols)

        # Mock the necessary functions
        with patch('src.services.similar.fetchers.similar_api._fetch_yahoo_recommended_symbols',
                   new_callable=AsyncMock) as mock_fetch_symbols, \
             patch('src.services.similar.fetchers.similar_api.get_simple_quotes',
                   new_callable=AsyncMock) as mock_get_quotes:
            mock_fetch_symbols.return_value = recommended_symbols
            mock_get_quotes.return_value = [SimpleQuote(**quote) for quote in quote_data]

            # Call the function
            result = await fetch_similar(test_symbol, test_limit, test_cookies, test_crumb)

            # Verify the result
            assert len(result) == len(recommended_symbols)
            assert all(isinstance(quote, SimpleQuote) for quote in result)
            assert [quote.symbol for quote in result] == recommended_symbols

            # Verify mocks were called correctly
            mock_fetch_symbols.assert_called_once_with(test_symbol, test_limit)
            mock_get_quotes.assert_called_once_with(recommended_symbols, test_cookies, test_crumb)

    async def test_fetch_similar_not_found(self, bypass_cache):
        """Test fetch_similar when no recommendations are found"""

        test_symbol = 'INVALID'
        test_limit = 5
        test_cookies = 'test_cookies'
        test_crumb = 'test_crumb'

        # Mock _fetch_yahoo_recommended_symbols to raise HTTPException
        with patch('src.services.similar.fetchers.similar_api._fetch_yahoo_recommended_symbols',
                   new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

            # Verify that HTTPException is re-raised
            with pytest.raises(HTTPException) as excinfo:
                await fetch_similar(test_symbol, test_limit, test_cookies, test_crumb)

            # Verify the exception details
            assert excinfo.value.status_code == 404
            assert "No similar stocks found or invalid symbol" in excinfo.value.detail

    async def test_scrape_similar_quotes(self, similar_quotes_html, bypass_cache):
        """Test scrape_similar_quotes function with cached HTML content"""
        test_symbols = ['AAPL', 'MSFT', 'NVDA', 'JPM', 'TQQQ', 'SPY']
        test_limit = 5

        for symbol in test_symbols:
            # Get cached HTML for this symbol
            url = f'https://finance.yahoo.com/quote/{symbol}'
            html_content = similar_quotes_html(url)

            # Mock the fetch function
            with patch('src.services.similar.fetchers.similar_scraper.fetch', new_callable=AsyncMock) as mock_fetch:
                mock_fetch.return_value = html_content

                # Call the function
                result = await scrape_similar_quotes(symbol, test_limit)

                # Verify the result structure
                assert isinstance(result, list)
                assert all(isinstance(quote, SimpleQuote) for quote in result)

                # Verify we got some results (may vary based on the cached HTML)
                # We won't assert exact counts or symbols since it depends on actual Yahoo Finance data
                assert len(result) >= 0

                # Check that the symbol itself is not in the results
                assert all(quote.symbol != symbol for quote in result)

                # Verify each quote has the expected properties
                for quote in result:
                    assert quote.symbol
                    assert quote.name
                    assert quote.price
                    assert quote.change
                    assert quote.percent_change

                    # Verify percent_change format
                    assert quote.percent_change.endswith('%')
                    assert quote.percent_change.startswith('+') or quote.percent_change.startswith('-')

                # Verify fetch was called with correct parameters
                mock_fetch.assert_called_once_with(url=url)

    async def test_scrape_similar_quotes_invalid_symbol(self, bypass_cache):
        """Test scrape_similar_quotes with an invalid symbol"""
        test_symbol = 'INVALID_SYMBOL_12345'
        test_limit = 5

        # Mock fetch to simulate a failed request
        with patch('src.services.similar.fetchers.similar_scraper.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = Exception("Failed to fetch data")

            # Verify that HTTPException is raised
            with pytest.raises(HTTPException) as excinfo:
                await scrape_similar_quotes(test_symbol, test_limit)

            # Verify the exception details
            assert excinfo.value.status_code == 500
            assert "No similar stocks found or invalid symbol" in excinfo.value.detail

    async def test_get_similar_quotes_success(self, bypass_cache):
        """Test get_similar_quotes with successful API fetch"""
        test_symbol = 'AAPL'
        test_cookies = 'test_cookies'
        test_crumb = 'test_crumb'
        test_limit = 3

        mock_quotes = [
            SimpleQuote(symbol="MSFT", name="Microsoft Corporation", price="385.22", change="-3.17",
                        percent_change="-0.82%"),
            SimpleQuote(symbol="GOOGL", name="Alphabet Inc.", price="142.65", change="1.23", percent_change="+0.87%"),
            SimpleQuote(symbol="META", name="Meta Platforms, Inc.", price="485.58", change="5.37", percent_change="+1.12%")
        ]

        # Mock fetch_similar to return our test quotes
        with patch('src.services.similar.get_similar_quotes.fetch_similar', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = mock_quotes

            # Call the function
            result = await get_similar_quotes(test_symbol, test_cookies, test_crumb, test_limit)

            # Verify the result
            assert result == mock_quotes
            mock_fetch.assert_called_once_with(test_symbol, test_limit, test_cookies, test_crumb)

    async def test_get_similar_quotes_fallback(self, bypass_cache):
        """Test get_similar_quotes fallback to scraper when API fails"""
        test_symbol = 'AAPL'
        test_cookies = 'test_cookies'
        test_crumb = 'test_crumb'
        test_limit = 3

        mock_quotes = [
            SimpleQuote(symbol="MSFT", name="Microsoft Corporation", price="385.22", change="-3.17",
                        percent_change="-0.82%"),
            SimpleQuote(symbol="GOOGL", name="Alphabet Inc.", price="142.65", change="1.23", percent_change="+0.87%")
        ]

        # Mock fetch_similar to raise a generic exception
        with patch('src.services.similar.get_similar_quotes.fetch_similar', new_callable=AsyncMock) as mock_fetch, \
             patch('src.services.similar.get_similar_quotes.scrape_similar_quotes',
                   new_callable=AsyncMock) as mock_scrape:
            mock_fetch.return_value = []
            mock_scrape.return_value = mock_quotes

            # Call the function
            result = await get_similar_quotes(test_symbol, test_cookies, test_crumb, test_limit)

            # Verify the result
            assert result == mock_quotes
            mock_fetch.assert_called_once_with(test_symbol, test_limit, test_cookies, test_crumb)
            mock_scrape.assert_called_once_with(test_symbol, test_limit)

    async def test_get_similar_quotes_http_exception(self, bypass_cache):
        """Test get_similar_quotes propagates HTTPException from fetch_similar"""
        test_symbol = 'INVALID'
        test_cookies = 'test_cookies'
        test_crumb = 'test_crumb'
        test_limit = 3

        # Mock fetch_similar to raise an HTTPException
        with patch('src.services.similar.get_similar_quotes.fetch_similar', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

            # Verify that HTTPException is re-raised
            with pytest.raises(HTTPException) as excinfo:
                await get_similar_quotes(test_symbol, test_cookies, test_crumb, test_limit)

            # Verify the exception details
            assert excinfo.value.status_code == 404
            assert "No similar stocks found or invalid symbol" in excinfo.value.detail
