from unittest.mock import AsyncMock, patch

import pytest
from fastapi import HTTPException
from orjson import orjson

from src.models import INDEX_REGIONS, Region, Index, MarketIndex
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


@pytest.fixture
def mock_api_response():
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


def test_get_indices_success(test_client, mock_yahoo_auth, monkeypatch):
    """Test successful indices retrieval"""
    # Mock the indices service function
    mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
    monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

    # Make the request
    response = test_client.get(f"{VERSION}/indices")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == len(MOCK_INDICES_RESPONSE)
    assert data == MOCK_INDICES_RESPONSE

    # Verify mock was called
    mock_scrape_indices.assert_awaited_once()


def test_get_indices_by_specific_index(test_client, mock_yahoo_auth, monkeypatch):
    """Test indices retrieval with specific index filter"""
    # Mock the indices service function
    mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0]])
    monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

    # Make the request with the S&P 500 index query parameter
    response = test_client.get(f"{VERSION}/indices?index=snp")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == 1
    assert data[0]["name"] == "S&P 500"

    # Verify mock was called with the correct indices
    mock_scrape_indices.assert_awaited_once()


def test_get_indices_by_region(test_client, mock_yahoo_auth, monkeypatch):
    """Test indices retrieval filtered by region"""
    # Create a spy to capture arguments passed to get_indices
    mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
    monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

    # Make the request with the region query parameter
    response = test_client.get(f"{VERSION}/indices?region=US")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) > 0

    # Verify get_indices was called with the right arguments
    mock_scrape_indices.assert_awaited_once()

    # Get the call arguments - the third argument should be ordered_indices
    args = mock_scrape_indices.call_args[0]
    assert len(args) >= 3
    us_indices = args[2]  # ordered_indices

    # Check that all indices in the argument are US indices
    for idx in us_indices:
        assert INDEX_REGIONS.get(idx) == Region.UNITED_STATES

    # Verify we have all US indices (7 US indices in your enum)
    assert len(us_indices) == 7

    # Check that specific US indices are included
    us_enum_values = {Index.GSPC, Index.DJI, Index.IXIC, Index.NYA,
                      Index.XAX, Index.RUT, Index.VIX}
    assert all(idx in us_enum_values for idx in us_indices)


def test_get_indices_multiple_filters(test_client, mock_yahoo_auth, monkeypatch):
    """Test indices retrieval with multiple filters"""
    # Mock the indices service function
    mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0], MOCK_INDICES_RESPONSE[1]])
    monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

    # Make the request with both index and region parameters
    # Using 'dax' (a European index) and US region
    response = test_client.get(f"{VERSION}/indices?index=dax&region=US")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == 2

    # Verify the correct indices were returned
    assert {item["name"] for item in data} == {"S&P 500", "Dow Jones Industrial Average"}

    # Verify mock was called
    mock_scrape_indices.assert_awaited_once()

    # Get the call arguments - the third argument should be ordered_indices
    args = mock_scrape_indices.call_args[0]
    assert len(args) >= 3
    ordered_indices = args[2]

    # Should include both DAX (European index specified directly) and US indices
    # Check that DAX is included
    assert Index.GDAXI in ordered_indices  # DAX index

    # Check that US indices are included
    us_indices = [idx for idx in ordered_indices if INDEX_REGIONS.get(idx) == Region.UNITED_STATES]
    assert len(us_indices) == 7  # All 7 US indices

    # Verify the total count (7 US indices + 1 DAX)
    assert len(ordered_indices) == 8


def test_get_indices_invalid_region(test_client, mock_yahoo_auth):
    """Test indices retrieval with invalid region parameter"""
    # Make the request with an invalid region
    response = test_client.get(f"{VERSION}/indices?region=INVALID_REGION")

    # Should return a 422 Unprocessable Entity
    assert response.status_code == 422

    # Response should contain validation error
    error_detail = response.json()["errors"]
    assert "region" in error_detail


def test_get_indices_invalid_index(test_client, mock_yahoo_auth):
    """Test indices retrieval with invalid index parameter"""
    # Make the request with an invalid index
    response = test_client.get(f"{VERSION}/indices?index=INVALID_INDEX")

    # Should return a 422 Unprocessable Entity
    assert response.status_code == 422

    # Response should contain validation error
    error_detail = response.json()["errors"]
    assert "index.0" in error_detail


@pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
async def test_fetch_index(mock_api_response, index, bypass_cache):
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
        assert result.name in ["S&P 500", "Dow Jones Industrial Average"]
        assert result.change.startswith("+") or result.change.startswith("-")
        assert result.percent_change.startswith("+") or result.percent_change.startswith("-")

        mock_fetch.assert_called()


@pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
async def test_fetch_index_failure(mock_api_response, index):
    """Test fetch_index function with mocked API response failure"""
    test_cookies = "mock_cookies"
    test_crumb = "mock_crumb"

    # Mock response data with an error
    error_response_data = {
        "quoteSummary": {
            "error": {
                "description": "Mocked error message"
            }
        }
    }

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

        mock_fetch.assert_called()
