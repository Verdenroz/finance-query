from unittest.mock import AsyncMock

from src.models import INDEX_REGIONS, Region, Index
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
