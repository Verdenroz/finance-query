from pathlib import Path
from unittest.mock import patch, AsyncMock

import pytest
import requests
from aiohttp import ClientResponse
from fastapi import HTTPException
from orjson import orjson

from src.models import MarketSector, Sector, MarketSectorDetails
from src.services.sectors.get_sectors import get_sector_for_symbol, urls, get_yahoo_sector, get_sector_details
from tests.conftest import VERSION


class TestSectors:
    @pytest.fixture
    def sector_html(self):
        """
        Fixture that provides a function to get cached HTML content for URLs.
        If the HTML is not cached, it will fetch and cache it.
        """
        # Path for storing cached HTML responses
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "sectors"
        cache_dir.mkdir(parents=True, exist_ok=True)

        # Create a dictionary to store HTML content by URL
        html_cache = {}

        def get_cached_html(url):
            # Check if we already have this URL in our in-memory cache
            if url in html_cache:
                return html_cache[url]

            # Extract sector path from URL for filename
            sector_path = url.split('sectors/')[1].strip('/')
            cache_file = cache_dir / f"{sector_path}.html"

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
    def yahoo_sectors(self):
        """
        Fixture that provides a function to get cached Yahoo API data for symbols.
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
            cache_file = cache_dir / f"{symbol}_yahoo_data.json"

            # Check if we have cached data
            if cache_file.exists():
                import json
                with open(cache_file, 'r') as f:
                    yahoo_data = json.load(f)
            else:
                # Create mock data if no cache exists
                sectors = {
                    'AAPL': 'Technology',
                    'MSFT': 'Technology',
                    'JPM': 'Financial Services',
                    'PFE': 'Healthcare',
                    'XOM': 'Energy',
                    'KO': 'Consumer Defensive',
                    'HD': 'Consumer Cyclical',
                    'VZ': 'Communication Services',
                    'NEE': 'Utilities',
                    'AMT': 'Real Estate',
                    'BHP': 'Basic Materials',
                    'BA': 'Industrials'
                }

                # Get sector for the symbol or use a default
                sector = sectors.get(symbol, 'Technology')

                yahoo_data = {
                    'quoteSummary': {
                        'result': [
                            {
                                'assetProfile': {
                                    'sector': sector
                                }
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

    async def test_sectors_endpoint(self, test_client, monkeypatch):
        """Test the /sectors endpoint"""

        async def mock_get_sectors():
            return [
                {
                    "sector": "Technology",
                    "dayReturn": "+0.45%",
                    "ytdReturn": "+12.34%",
                    "yearReturn": "+23.45%",
                    "threeYearReturn": "+34.56%",
                    "fiveYearReturn": "+45.67%"
                }
            ]

        monkeypatch.setattr('src.routes.sectors.get_sectors', mock_get_sectors)

        response = test_client.get(f"/{VERSION}/sectors")

        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) == 1
        assert data[0]["sector"] == "Technology"
        assert data[0]["dayReturn"] == "+0.45%"

    async def test_sector_by_symbol_endpoint(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test the /sectors/symbol/{symbol} endpoint"""

        async def mock_get_sector_for_symbol(symbol, cookies, crumb):
            if symbol == "AAPL":
                return {
                    "sector": "Technology",
                    "dayReturn": "+0.45%",
                    "ytdReturn": "+12.34%",
                    "yearReturn": "+23.45%",
                    "threeYearReturn": "+34.56%",
                    "fiveYearReturn": "+45.67%"
                }
            raise HTTPException(status_code=404, detail=f"Sector for {symbol} not found.")

        monkeypatch.setattr('src.routes.sectors.get_sector_for_symbol', mock_get_sector_for_symbol)

        # Test successful request
        response = test_client.get(f"/{VERSION}/sectors/symbol/AAPL")

        assert response.status_code == 200
        data = response.json()
        assert data["sector"] == "Technology"

        # Test with unknown symbol
        response = test_client.get(f"/{VERSION}/sectors/symbol/UNKNOWN")

        assert response.status_code == 404
        assert "Sector for UNKNOWN not found" in response.json()["detail"]

    async def test_sector_details_endpoint(self, test_client, monkeypatch):
        """Test the /sectors/details/{sector} endpoint"""

        async def mock_get_sector_details(sector):
            return {
                "sector": sector.value,
                "dayReturn": "+0.45%",
                "ytdReturn": "+12.34%",
                "yearReturn": "+23.45%",
                "threeYearReturn": "+34.56%",
                "fiveYearReturn": "+45.67%",
                "marketCap": "20.196T",
                "market_weight": "29.28%",
                "industries": 12,
                "companies": 815,
                "topIndustries": ["Semiconductors: 29.04%", "Software - Infrastructure: 26.44%"],
                "topCompanies": ["NVDA", "AAPL"]
            }

        monkeypatch.setattr('src.routes.sectors.get_sector_details', mock_get_sector_details)

        response = test_client.get(f"/{VERSION}/sectors/details/Technology")

        assert response.status_code == 200
        data = response.json()
        assert data["sector"] == "Technology"
        assert data["marketCap"] == "20.196T"
        assert len(data["topIndustries"]) == 2
        assert len(data["topCompanies"]) == 2

        # Test with invalid sector
        response = test_client.get(f"/{VERSION}/sectors/details/invalid")
        assert response.status_code == 422  # Validation error

    async def test_get_sectors(self, sector_html, bypass_cache):
        """Test the sector scraping service with real cached HTML responses"""
        # Create a dictionary to store HTML content by URL
        html_cache = {}

        # Prepare HTML cache for each sector URL
        for sector, url in urls.items():
            html_content = sector_html(url)
            html_cache[url] = html_content

        # Create a fetch mock that returns cached HTML directly
        async def mock_fetch(url):
            return html_cache[url]

        # Patch the fetch function
        with patch('src.services.sectors.get_sectors.fetch', mock_fetch):
            # Run the function with our patched fetch
            from src.services.sectors.get_sectors import get_sectors
            result = await get_sectors()

        # Verify the results
        assert result is not None
        assert len(result) == len(urls)

        # Check that each sector has the expected values
        for sector_data in result:
            assert isinstance(sector_data, MarketSector)
            assert isinstance(sector_data.sector, str)

            # Verify that the sector string matches one of the Sector enum values
            assert sector_data.sector in [s.value for s in Sector], f"{sector_data.sector} is not a valid Sector enum value"

            assert sector_data.day_return.startswith("+") or sector_data.day_return.startswith("-")
            assert sector_data.ytd_return.startswith("+") or sector_data.ytd_return.startswith("-")
            assert sector_data.year_return.startswith("+") or sector_data.year_return.startswith("-")
            assert sector_data.three_year_return.startswith("+") or sector_data.three_year_return.startswith("-")
            assert sector_data.five_year_return.startswith("+") or sector_data.five_year_return.startswith("-")

            assert sector_data.day_return.endswith("%")
            assert sector_data.ytd_return.endswith("%")
            assert sector_data.year_return.endswith("%")
            assert sector_data.three_year_return.endswith("%")
            assert sector_data.five_year_return.endswith("%")

    async def test_get_yahoo_sector(self, yahoo_sectors, bypass_cache):
        """Test the get_yahoo_sector function with cached Yahoo API data"""
        # Get test symbols
        test_symbols = ['AAPL', 'MSFT', 'JPM', 'PFE']

        for symbol in test_symbols:
            # Get cached data for this symbol
            yahoo_data = yahoo_sectors(symbol)

            # Expected sector from the cached data
            expected_sector = yahoo_data['quoteSummary']['result'][0]['assetProfile']['sector']

            # Mock the _fetch_yahoo_data function to return our cached data
            with patch('src.services.sectors.utils._fetch_yahoo_data', new_callable=AsyncMock) as mock_fetch:
                mock_fetch.return_value = yahoo_data

                # Call the function with test parameters
                result = await get_yahoo_sector(symbol, 'test_cookies', 'test_crumb')

                # Verify the result
                assert result == expected_sector

                # Verify the fetch function was called with correct parameters
                mock_fetch.assert_called_once_with(symbol, 'test_cookies', 'test_crumb')

    async def test_get_sector_for_symbol(self, yahoo_sectors, sector_html, bypass_cache):
        """Test the get_sector_for_symbol function with cached data"""
        # Set up test symbols
        test_symbols = ['AAPL', 'MSFT', 'JPM', 'PFE']

        for symbol in test_symbols:
            # Get cached Yahoo data for this symbol
            yahoo_data = yahoo_sectors(symbol)
            expected_sector = yahoo_data['quoteSummary']['result'][0]['assetProfile']['sector']

            # Get the sector URL
            sector_url = urls[Sector(expected_sector)]

            # Get cached HTML for this sector
            html_content = sector_html(sector_url)

            # Mock the necessary functions
            with patch('src.services.sectors.get_sectors.get_yahoo_sector', new_callable=AsyncMock) as mock_get_sector, \
                 patch('src.services.sectors.get_sectors.fetch', new_callable=AsyncMock) as mock_fetch:
                # Set up the mocks to return our cached data
                mock_get_sector.return_value = expected_sector
                mock_fetch.return_value = html_content

                # Call the function
                result = await get_sector_for_symbol(symbol, 'test_cookies', 'test_crumb')

                # Verify the result
                assert isinstance(result, MarketSector)
                assert result.sector == expected_sector

                # Verify that the return values are properly formatted
                assert result.day_return.endswith("%")
                assert result.ytd_return.endswith("%")
                assert result.year_return.endswith("%")
                assert result.three_year_return.endswith("%")
                assert result.five_year_return.endswith("%")

                assert result.day_return.startswith("+") or result.day_return.startswith("-")
                assert result.ytd_return.startswith("+") or result.ytd_return.startswith("-")
                assert result.year_return.startswith("+") or result.year_return.startswith("-")
                assert result.three_year_return.startswith("+") or result.three_year_return.startswith("-")
                assert result.five_year_return.startswith("+") or result.five_year_return.startswith("-")

                # Verify the mocks were called correctly
                mock_get_sector.assert_called_once_with(symbol, 'test_cookies', 'test_crumb')
                mock_fetch.assert_called_once_with(url=sector_url)

    async def test_get_sector_for_symbol_not_found(self, bypass_cache):
        """Test the get_sector_for_symbol function when sector is not found"""
        # Mock get_yahoo_sector to return None
        with patch('src.services.sectors.get_sectors.get_yahoo_sector', new_callable=AsyncMock) as mock_get_sector:
            mock_get_sector.return_value = None

            # Verify that HTTPException is raised
            with pytest.raises(HTTPException) as excinfo:
                await get_sector_for_symbol('UNKNOWN', 'test_cookies', 'test_crumb')

            # Verify the exception details
            assert excinfo.value.status_code == 404
            assert "Sector for UNKNOWN not found" in excinfo.value.detail

    async def test_get_sector_details(self, sector_html, bypass_cache):
        """Test the get_sector_details function with cached HTML content"""
        # Test with all sectors
        test_sectors = list(Sector)

        for sector in test_sectors:
            # Get cached HTML for this sector
            url = urls[sector]
            html_content = sector_html(url)

            # Mock the fetch function
            with patch('src.services.sectors.get_sectors.fetch', new_callable=AsyncMock) as mock_fetch:
                mock_fetch.return_value = html_content

                # Call the function
                result = await get_sector_details(sector)

                # Verify the result
                assert isinstance(result, MarketSectorDetails)
                assert result.sector == sector.value

                # Verify structure of the result
                assert result.day_return.endswith("%")
                assert result.ytd_return.endswith("%")
                assert result.year_return.endswith("%")
                assert result.three_year_return.endswith("%")
                assert result.five_year_return.endswith("%")

                assert result.day_return.startswith("+") or result.day_return.startswith("-")
                assert result.ytd_return.startswith("+") or result.ytd_return.startswith("-")
                assert result.year_return.startswith("+") or result.year_return.startswith("-")
                assert result.three_year_return.startswith("+") or result.three_year_return.startswith("-")
                assert result.five_year_return.startswith("+") or result.five_year_return.startswith("-")

                # Verify numeric fields
                assert isinstance(result.industries, int)
                assert isinstance(result.companies, int)

                # Verify lists
                assert isinstance(result.top_industries, list)
                assert isinstance(result.top_companies, list)

                # Verify non-empty lists
                assert len(result.top_industries) > 0
                assert len(result.top_companies) > 0

                # Mock was called with the correct URL
                mock_fetch.assert_called_once_with(url=url)

    async def test_fetch_yahoo_data(self):
        """Test the _fetch_yahoo_data function with mocked responses"""
        from src.services.sectors.utils import _fetch_yahoo_data

        # Test successful response
        mock_successful_data = {
            "quoteSummary": {
                "result": [{
                    "assetProfile": {
                        "sector": "Technology"
                    }
                }]
            }
        }

        mock_response = AsyncMock(spec=ClientResponse)
        mock_response.status = 200
        mock_response.text.return_value = orjson.dumps(mock_successful_data).decode()

        with patch('src.services.sectors.utils.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = mock_response

            result = await _fetch_yahoo_data("AAPL", "test_cookies", "test_crumb")

            assert result == mock_successful_data
            mock_fetch.assert_called_once_with(
                url="https://query2.finance.yahoo.com/v10/finance/quoteSummary/AAPL",
                params={"modules": "assetProfile", "crumb": "test_crumb"},
                headers={
                    'Cookie': 'test_cookies',
                    'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
                    'Accept': 'application/json'
                },
                return_response=True
            )

        # Test 404 response
        mock_error_response = AsyncMock(spec=ClientResponse)
        mock_error_response.status = 404

        with patch('src.services.sectors.utils.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = mock_error_response

            with pytest.raises(HTTPException) as excinfo:
                await _fetch_yahoo_data("INVALID", "test_cookies", "test_crumb")

            assert excinfo.value.status_code == 404
            assert "Symbol not found: INVALID" in excinfo.value.detail
