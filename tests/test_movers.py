from unittest.mock import AsyncMock

import pytest

from src.models.marketmover import MoverCount
from tests.conftest import VERSION

# Mock response data for different count values
MOCK_MOVER_RESPONSE_TWENTY_FIVE = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percentChange": f"+{i * 0.5:.2f}%"
    } for i in range(1, 26)
]

MOCK_MOVER_RESPONSE_FIFTY = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percentChange": f"+{i * 0.5:.2f}%"
    } for i in range(1, 51)
]

MOCK_MOVER_RESPONSE_HUNDRED = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percentChange": f"+{i * 0.5:.2f}%"
    } for i in range(1, 101)
]

# Test data mapping count values to responses
COUNT_RESPONSE_MAP = {
    "25": MOCK_MOVER_RESPONSE_TWENTY_FIVE,
    "50": MOCK_MOVER_RESPONSE_FIFTY,
    "100": MOCK_MOVER_RESPONSE_HUNDRED
}

# Test data for endpoints
ENDPOINTS = ["actives", "gainers", "losers"]


@pytest.mark.parametrize("count", ["25", "50", "100"])
@pytest.mark.parametrize("endpoint", ENDPOINTS)
def test_get_movers_success(test_client, count, endpoint, mock_yahoo_auth, monkeypatch):
    """Test successful movers retrieval with different count values"""
    # Mock the corresponding service function
    mock_service = AsyncMock(return_value=COUNT_RESPONSE_MAP[count])

    if endpoint == "actives":
        monkeypatch.setattr("src.routes.movers.get_actives", mock_service)
    elif endpoint == "gainers":
        monkeypatch.setattr("src.routes.movers.get_gainers", mock_service)
    else:  # losers
        monkeypatch.setattr("src.routes.movers.get_losers", mock_service)

    # Make the request
    response = test_client.get(f"{VERSION}/{endpoint}?count={count}")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == int(count)
    assert data == COUNT_RESPONSE_MAP[count]

    # Verify mock was called
    mock_service.assert_awaited_once_with(MoverCount(count))


@pytest.mark.parametrize("endpoint", ENDPOINTS)
def test_get_movers_default_count(test_client, endpoint, mock_yahoo_auth, monkeypatch):
    """Test movers retrieval with default count (50)"""
    # Mock the corresponding service function
    mock_service = AsyncMock(return_value=MOCK_MOVER_RESPONSE_FIFTY)

    if endpoint == "actives":
        monkeypatch.setattr("src.routes.movers.get_actives", mock_service)
    elif endpoint == "gainers":
        monkeypatch.setattr("src.routes.movers.get_gainers", mock_service)
    else:  # losers
        monkeypatch.setattr("src.routes.movers.get_losers", mock_service)

    # Make the request without count parameter (should use default 50)
    response = test_client.get(f"{VERSION}/{endpoint}")
    data = response.json()

    # Assertions
    assert response.status_code == 200
    assert len(data) == 50
    assert data == MOCK_MOVER_RESPONSE_FIFTY

    # Verify mock was called with default count
    mock_service.assert_awaited_once_with(MoverCount.FIFTY)


@pytest.mark.parametrize("endpoint", ENDPOINTS)
def test_get_movers_invalid_count(test_client, endpoint, mock_yahoo_auth):
    """Test movers retrieval with invalid count value"""
    # Make request with invalid count
    response = test_client.get(f"{VERSION}/{endpoint}?count=42")
    data = response.json()

    # Assertions for validation error
    assert response.status_code == 422
    assert "detail" in data
    assert "errors" in data
    assert "count" in data["errors"]
    assert "Input should be '25', '50' or '100'" in data["errors"]["count"]

