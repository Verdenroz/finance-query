from unittest.mock import AsyncMock

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
