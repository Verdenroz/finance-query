from unittest.mock import AsyncMock, patch

import pytest
from fastapi import HTTPException
from orjson import orjson

from src.models import INDEX_REGIONS, Region, Index, MarketIndex
from src.services import get_indices
from src.services.indices.fetchers import fetch_index
from tests.conftest import VERSION

# Mock indices response data
MOCK_INDICES_RESPONSE = [
    {
        "name": "S&P 500",
        "value": 5234.32,
        "change": "+15.29",
        "percentChange": "+0.29%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None
    },
    {
        "name": "Dow Jones Industrial Average",
        "value": 39127.14,
        "change": "+62.39",
        "percentChange": "+0.16%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None
    },
    {
        "name": "NASDAQ Composite",
        "value": 16439.22,
        "change": "+83.95",
        "percentChange": "+0.51%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None
    },
    {
        "name": "Russell 2000",
        "value": 2042.60,
        "change": "-3.71",
        "percentChange": "-0.18%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None
    }
]


class TestIndices:
    @pytest.fixture
    def mock_api_response(self):
        """
        Fixture that provides a function to get a mock API response for URLs.
        """
        response_cache = {}

        def get_mock_response(url):
            if url in response_cache:
                return response_cache[url]

            # Define mock response data based on the URL
            mock_responses = {
                "https://query2.finance.yahoo.com/v10/finance/quoteSummary/GSPC": {
                    "quoteSummary": {
                        "result": [{
                            "price": {
                                "regularMarketPrice": {"raw": 5234.32},
                                "regularMarketChange": {"fmt": "+15.29"},
                                "regularMarketChangePercent": {"fmt": "+0.29%"},
                                "longName": "S&P 500",
                                "shortName": "S&P 500"
                            },
                            "quoteUnadjustedPerformanceOverview": {
                                "performanceOverview": {
                                    "fiveDaysReturn": {"fmt": "+1.23%"},
                                    "oneMonthReturn": {"fmt": "-2.34%"},
                                    "threeMonthReturn": {"fmt": "+3.45%"},
                                    "sixMonthReturn": {"fmt": "-4.56%"},
                                    "ytdReturnPct": {"fmt": "+5.67%"},
                                    "oneYearTotalReturn": {"fmt": "-6.78%"},
                                    "threeYearTotalReturn": {"fmt": "+7.89%"},
                                    "fiveYearTotalReturn": {"fmt": "-8.90%"},
                                    "tenYearTotalReturn": {"fmt": "+9.01%"},
                                    "maxReturn": {"fmt": "-10.12%"}
                                }
                            }
                        }],
                        "error": None
                    }
                },
                "https://query2.finance.yahoo.com/v10/finance/quoteSummary/DJI": {
                    "quoteSummary": {
                        "result": [{
                            "price": {
                                "regularMarketPrice": {"raw": 39127.14},
                                "regularMarketChange": {"fmt": "+62.39"},
                                "regularMarketChangePercent": {"fmt": "+0.16%"},
                                "longName": "Dow Jones Industrial Average",
                                "shortName": "Dow Jones Industrial Average"
                            },
                            "quoteUnadjustedPerformanceOverview": {
                                "performanceOverview": {
                                    "fiveDaysReturn": {"fmt": "+1.23%"},
                                    "oneMonthReturn": {"fmt": "-2.34%"},
                                    "threeMonthReturn": {"fmt": "+3.45%"},
                                    "sixMonthReturn": {"fmt": "-4.56%"},
                                    "ytdReturnPct": {"fmt": "+5.67%"},
                                    "oneYearTotalReturn": {"fmt": "-6.78%"},
                                    "threeYearTotalReturn": {"fmt": "+7.89%"},
                                    "fiveYearTotalReturn": {"fmt": "-8.90%"},
                                    "tenYearTotalReturn": {"fmt": "+9.01%"},
                                    "maxReturn": {"fmt": "-10.12%"}
                                }
                            }
                        }],
                        "error": None
                    }
                }
            }

            response_content = orjson.dumps(mock_responses[url]).decode('utf-8')
            response_cache[url] = response_content
            return response_content

        return get_mock_response

    def test_get_indices_success(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test successful indices retrieval"""
        mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == len(MOCK_INDICES_RESPONSE)
        assert data == MOCK_INDICES_RESPONSE

        mock_scrape_indices.assert_awaited_once()

    def test_get_indices_by_specific_index(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test indices retrieval with specific index filter"""
        mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0]])
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?index=snp")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0]["name"] == "S&P 500"

        mock_scrape_indices.assert_awaited_once()

    def test_get_indices_by_region(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test indices retrieval filtered by region"""
        mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?region=US")
        data = response.json()

        assert response.status_code == 200
        assert len(data) > 0

        mock_scrape_indices.assert_awaited_once()

        args = mock_scrape_indices.call_args[0]
        us_indices = args[2]
        for idx in us_indices:
            assert INDEX_REGIONS.get(idx) == Region.UNITED_STATES
        assert len(us_indices) == 7
        us_enum_values = {Index.GSPC, Index.DJI, Index.IXIC, Index.NYA, Index.XAX, Index.RUT, Index.VIX}
        assert all(idx in us_enum_values for idx in us_indices)

    def test_get_indices_multiple_filters(self, test_client, mock_yahoo_auth, monkeypatch):
        """Test indices retrieval with multiple filters"""
        mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0], MOCK_INDICES_RESPONSE[1]])
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?index=dax&region=US")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 2
        assert {item["name"] for item in data} == {"S&P 500", "Dow Jones Industrial Average"}

        mock_scrape_indices.assert_awaited_once()

        args = mock_scrape_indices.call_args[0]
        ordered_indices = args[2]
        assert Index.GDAXI in ordered_indices
        us_indices = [idx for idx in ordered_indices if INDEX_REGIONS.get(idx) == Region.UNITED_STATES]
        assert len(us_indices) == 7
        assert len(ordered_indices) == 8

    def test_get_indices_invalid_region(self, test_client, mock_yahoo_auth):
        """Test indices retrieval with invalid region parameter"""
        response = test_client.get(f"{VERSION}/indices?region=INVALID_REGION")
        assert response.status_code == 422
        error_detail = response.json()["errors"]
        assert "region" in error_detail

    def test_get_indices_invalid_index(self, test_client, mock_yahoo_auth):
        """Test indices retrieval with invalid index parameter"""
        response = test_client.get(f"{VERSION}/indices?index=INVALID_INDEX")
        assert response.status_code == 422
        error_detail = response.json()["errors"]
        assert "index.0" in error_detail

    async def test_get_indices(self, bypass_cache):
        """Test get_indices function with all indices"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"

        mock_index_data = MarketIndex(
            name="Test Index",
            value=1000.0,
            change="+10.0",
            percent_change="+1.0%",
        )

        with patch('src.services.indices.get_indices.fetch_index', new_callable=AsyncMock) as mock_fetch_index:
            mock_fetch_index.return_value = mock_index_data
            indices = list(Index)
            result = await get_indices(test_cookies, test_crumb, None)

            assert len(result) == len(indices)
            assert all(isinstance(index, MarketIndex) for index in result)
            assert mock_fetch_index.call_count == len(indices)

    async def test_get_indices_error_handling(self, bypass_cache):
        """Test get_indices handles errors correctly"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"

        mock_index_data = MarketIndex(
            name="Test Index",
            value=1000.0,
            change="+10.0",
            percent_change="+1.0%"
        )

        async def mock_fetch_side_effect(index, *args, **kwargs):
            if index == Index.GDAXI:
                return Exception("Failed to fetch index")
            return mock_index_data

        with patch('src.services.indices.get_indices.fetch_index', new_callable=AsyncMock) as mock_fetch_index:
            mock_fetch_index.side_effect = mock_fetch_side_effect
            test_indices = [Index.GSPC, Index.DJI, Index.GDAXI, Index.IXIC]
            result = await get_indices(test_cookies, test_crumb, test_indices)
            assert len(result) == 3

    async def test_get_indices_missing_credentials(self, bypass_cache):
        """Test get_indices raises error with missing credentials"""
        with pytest.raises(ValueError):
            await get_indices("", "")
        with pytest.raises(ValueError):
            await get_indices("cookies", "")

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index(self, mock_api_response, index, bypass_cache):
        """Test fetch_index function with mocked API response"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"
        test_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{index.name}"
        response_content = mock_api_response(test_url)
        mock_response_data = orjson.loads(response_content)

        class MockResponse:
            def __init__(self, json_data, status_code):
                self._json_data = json_data
                self.status = status_code

            async def text(self):
                return orjson.dumps(self._json_data).decode('utf-8')

        with patch('src.services.indices.fetchers.fetch_index.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = MockResponse(mock_response_data, 200)
            result = await fetch_index(index, test_cookies, test_crumb)
            assert isinstance(result, MarketIndex)
            assert result.change.startswith(('+', '-'))
            assert result.percent_change.startswith(('+', '-'))

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index_failure(self, mock_api_response, index):
        """Test fetch_index function with mocked API response failure"""
        test_cookies = "mock_cookies"
        test_crumb = "mock_crumb"
        error_response_data = {"quoteSummary": {"error": {"description": "Mocked error message"}}}

        class MockResponse:
            def __init__(self, json_data, status_code):
                self._json_data = json_data
                self.status = status_code

            async def text(self):
                return orjson.dumps(self._json_data).decode('utf-8')

        with patch('src.services.indices.fetchers.fetch_index.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = MockResponse(error_response_data, 500)
            with pytest.raises(HTTPException) as exc_info:
                await fetch_index(index, test_cookies, test_crumb)
            assert exc_info.value.status_code == 500
            assert exc_info.value.detail == "Mocked error message"
