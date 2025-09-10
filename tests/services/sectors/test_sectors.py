from unittest.mock import AsyncMock, patch

import pytest
from fastapi import HTTPException

from src.models import MarketSector, MarketSectorDetails, Sector
from src.services.sectors.get_sectors import get_sector_details, get_sector_for_symbol, urls
from tests.conftest import VERSION


class TestSectors:
    @pytest.fixture
    def yahoo_sectors(self):
        """
        Fixture that provides mock Yahoo API data for symbols.
        This is not cached since it's just mock data generation.
        """

        def get_mock_data(symbol):
            # Mock sector data mapping
            sectors = {
                "AAPL": "Technology",
                "MSFT": "Technology",
                "JPM": "Financial Services",
                "PFE": "Healthcare",
                "XOM": "Energy",
                "KO": "Consumer Defensive",
                "HD": "Consumer Cyclical",
                "VZ": "Communication Services",
                "NEE": "Utilities",
                "AMT": "Real Estate",
                "BHP": "Basic Materials",
                "BA": "Industrials",
            }

            sector = sectors.get(symbol, "Technology")
            return {"quoteSummary": {"result": [{"assetProfile": {"sector": sector}}]}}

        return get_mock_data

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
                    "fiveYearReturn": "+45.67%",
                }
            ]

        monkeypatch.setattr("src.routes.sectors.get_sectors", mock_get_sectors)

        response = test_client.get(f"/{VERSION}/sectors")

        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) == 1
        assert data[0]["sector"] == "Technology"
        assert data[0]["dayReturn"] == "+0.45%"

    async def test_sector_by_symbol_endpoint(self, test_client, mock_finance_client, monkeypatch):
        """Test the /sectors/symbol/{symbol} endpoint"""

        async def mock_get_sector_for_symbol(finance_client, symbol):
            if symbol == "AAPL":
                return {
                    "sector": "Technology",
                    "dayReturn": "+0.45%",
                    "ytdReturn": "+12.34%",
                    "yearReturn": "+23.45%",
                    "threeYearReturn": "+34.56%",
                    "fiveYearReturn": "+45.67%",
                }
            raise HTTPException(status_code=404, detail=f"Sector for {symbol} not found.")

        monkeypatch.setattr("src.routes.sectors.get_sector_for_symbol", mock_get_sector_for_symbol)

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
                "topCompanies": ["NVDA", "AAPL"],
            }

        monkeypatch.setattr("src.routes.sectors.get_sector_details", mock_get_sector_details)

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

    async def test_get_sectors(self, html_cache_manager, bypass_cache):
        """Test the sector scraping service with real cached HTML responses"""
        # Create a dictionary to store HTML content by URL
        html_cache = {}

        # Prepare HTML cache for each sector URL
        for sector, url in urls.items():
            context = f"sectors_{sector.value.lower().replace(' ', '_')}"
            html_content = html_cache_manager(url, context=context)
            html_cache[url] = html_content

        # Create a fetch mock that returns cached HTML directly
        async def mock_fetch(url):
            return html_cache[url]

        # Patch the fetch function
        with patch("src.services.sectors.get_sectors.fetch", mock_fetch):
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

    async def test_get_sector_for_symbol(self, yahoo_sectors, html_cache_manager, mock_finance_client, bypass_cache):
        """Test the get_sector_for_symbol function with cached data"""
        # Set up test symbols
        test_symbols = ["AAPL", "MSFT", "JPM", "PFE"]

        for symbol in test_symbols:
            # Get mock Yahoo data for this symbol
            yahoo_data = yahoo_sectors(symbol)
            expected_sector = yahoo_data["quoteSummary"]["result"][0]["assetProfile"]["sector"]

            # Get the sector URL
            sector_url = urls[Sector(expected_sector)]

            # Get cached HTML for this sector
            context = f"sectors_{expected_sector.lower().replace(' ', '_')}"
            html_content = html_cache_manager(sector_url, context=context)

            # Mock the necessary functions
            with (
                patch("src.services.sectors.get_sectors.get_yahoo_sector", new_callable=AsyncMock) as mock_get_sector,
                patch("src.services.sectors.get_sectors.fetch", new_callable=AsyncMock) as mock_fetch,
            ):
                # Set up the mocks to return our cached data
                mock_get_sector.return_value = expected_sector
                mock_fetch.return_value = html_content

                # Call the function
                result = await get_sector_for_symbol(mock_finance_client, symbol)

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
                mock_get_sector.assert_awaited_once()
                mock_fetch.assert_called_once_with(url=sector_url)

    async def test_get_sector_for_symbol_not_found(self, bypass_cache, mock_finance_client):
        """Test the get_sector_for_symbol function when sector is not found"""
        # Mock get_yahoo_sector to return None
        with patch("src.services.sectors.get_sectors.get_yahoo_sector", new_callable=AsyncMock) as mock_get_sector:
            mock_get_sector.return_value = None

            # Verify that HTTPException is raised
            with pytest.raises(HTTPException) as excinfo:
                await get_sector_for_symbol(mock_finance_client, "UNKNOWN")

            # Verify the exception details
            assert excinfo.value.status_code == 404
            assert "Sector for UNKNOWN not found" in excinfo.value.detail

    async def test_get_sector_details(self, html_cache_manager, bypass_cache):
        """Test the get_sector_details function with cached HTML content"""
        # Test with all sectors
        test_sectors = list(Sector)

        for sector in test_sectors:
            # Get cached HTML for this sector
            url = urls[sector]
            context = f"sectors_{sector.value.lower().replace(' ', '_')}_details"
            html_content = html_cache_manager(url, context=context)

            # Mock the fetch function
            with patch("src.services.sectors.get_sectors.fetch", new_callable=AsyncMock) as mock_fetch:
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
