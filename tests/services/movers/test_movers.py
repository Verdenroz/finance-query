from unittest.mock import AsyncMock, patch

import orjson
import pytest

from src.models.marketmover import MarketMover, MoverCount
from src.services.movers import get_actives, get_gainers, get_losers
from src.services.movers.fetchers import fetch_movers, scrape_movers
from tests.conftest import VERSION


# Mock response data factory
def create_mock_mover_response(count: int) -> list[dict]:
    """Generate mock mover response data for given count."""
    return [
        {
            "symbol": f"SYM{i}",
            "name": f"Company {i}",
            "price": f"{100 + i:.2f}",
            "change": f"+{i * 0.5:.2f}",
            "percentChange": f"+{i * 0.5:.2f}%",
        }
        for i in range(1, count + 1)
    ]


# Test data mapping count values to responses
COUNT_RESPONSE_MAP = {
    "25": create_mock_mover_response(25),
    "50": create_mock_mover_response(50),
    "100": create_mock_mover_response(100),
}

# Test data for endpoints
ENDPOINTS = ["actives", "gainers", "losers"]

# Maps endpoints to patch
SERVICE_MAP = {
    "actives": "src.routes.movers.get_actives",
    "gainers": "src.routes.movers.get_gainers",
    "losers": "src.routes.movers.get_losers",
}


def patch_mover_service(endpoint: str, monkeypatch, mock_service):
    """Helper function to patch the correct service based on endpoint."""
    service_path = SERVICE_MAP[endpoint]
    monkeypatch.setattr(service_path, mock_service)


def assert_mover_list_valid(result: list, expected_count: int):
    """Helper function to validate mover list structure."""
    assert isinstance(result, list)
    assert len(result) == expected_count
    assert all(isinstance(m, MarketMover) for m in result)


def assert_api_response_valid(response, expected_data: list, status_code: int = 200):
    """Helper function to validate API response structure."""
    data = response.json()
    assert response.status_code == status_code
    assert len(data) == len(expected_data)
    assert data == expected_data


class TestMovers:
    @pytest.fixture
    def mock_api_response(self):
        """Fixture that provides mock Yahoo Finance API responses for movers."""

        def get_mock_response(count=50):
            mock_items = [
                {
                    "symbol": f"SYM{i}",
                    "longName": f"Company {i} Long",
                    "shortName": f"Company {i}",
                    "regularMarketPrice": {"fmt": f"{100 + i:.2f}"},
                    "regularMarketChange": {"fmt": f"+{i * 0.5:.2f}"},
                    "regularMarketChangePercent": {"fmt": f"+{i * 0.5:.2f}%"},
                }
                for i in range(1, count + 1)
            ]
            return {"finance": {"result": [{"quotes": mock_items}]}}

        return get_mock_response

    @pytest.mark.parametrize("count", ["25", "50", "100"])
    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_success(self, test_client, count, endpoint, mock_finance_client, monkeypatch):
        """Test successful movers retrieval with different count values"""
        expected_data = COUNT_RESPONSE_MAP[count]
        mock_service = AsyncMock(return_value=expected_data)
        patch_mover_service(endpoint, monkeypatch, mock_service)

        response = test_client.get(f"{VERSION}/{endpoint}?count={count}")

        assert_api_response_valid(response, expected_data)
        mock_service.assert_awaited_once_with(MoverCount(count))

    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_default_count(self, test_client, endpoint, mock_finance_client, monkeypatch):
        """Test movers retrieval with default count (50)"""
        expected_data = COUNT_RESPONSE_MAP["50"]
        mock_service = AsyncMock(return_value=expected_data)
        patch_mover_service(endpoint, monkeypatch, mock_service)

        response = test_client.get(f"{VERSION}/{endpoint}")

        assert_api_response_valid(response, expected_data)
        mock_service.assert_awaited_once_with(MoverCount.FIFTY)

    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_invalid_count(self, test_client, endpoint, mock_finance_client):
        """Test movers retrieval with invalid count value"""
        response = test_client.get(f"{VERSION}/{endpoint}?count=42")
        data = response.json()

        assert response.status_code == 422
        assert "detail" in data
        assert "errors" in data
        assert "count" in data["errors"]
        assert "Input should be '25', '50' or '100'" in data["errors"]["count"]

    async def test_fetch_movers(self, mock_api_response, bypass_cache):
        """Test fetch_movers function with mocked API response"""
        test_url = "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=MOST_ACTIVES"
        test_count = 50
        mock_response = mock_api_response(test_count)
        expected_movers = []
        for item in mock_response["finance"]["result"][0]["quotes"]:
            mover = MarketMover(
                symbol=item["symbol"],
                name=item["longName"],
                price=item["regularMarketPrice"]["fmt"],
                change=item["regularMarketChange"]["fmt"],
                percent_change=item["regularMarketChangePercent"]["fmt"],
            )
            expected_movers.append(mover)

        with patch("src.services.movers.fetchers.movers_api.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = orjson.dumps(mock_response)
            result = await fetch_movers(test_url)

        assert isinstance(result, list)
        assert len(result) == test_count
        assert all(isinstance(m, MarketMover) for m in result)
        expected_params = {
            "fields": ("symbol,longName,shortName,regularMarketPrice,regularMarketChange,regularMarketChangePercent"),
        }
        mock_fetch.assert_called_once_with(url=test_url, params=expected_params)
        for i, mover in enumerate(result):
            assert mover.symbol == f"SYM{i + 1}"
            assert mover.name == f"Company {i + 1} Long"
            assert mover.price == f"{100 + i + 1:.2f}"
            assert mover.change == f"+{(i + 1) * 0.5:.2f}"
            assert mover.percent_change == f"+{(i + 1) * 0.5:.2f}%"

    async def test_scrape_movers(self, html_cache_manager, bypass_cache):
        """Test scrape_movers function with cached HTML content"""
        test_url = "https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50"
        html_content = html_cache_manager(test_url, context="movers_actives_50")
        with patch("src.services.movers.fetchers.movers_scraper.fetch", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content
            result = await scrape_movers(test_url)

        assert isinstance(result, list)
        assert len(result) == 50
        assert all(isinstance(m, MarketMover) for m in result)
        assert all(m.change.startswith(("+", "-", "0")) for m in result)
        assert all(m.percent_change.endswith("%") for m in result)
        mock_fetch.assert_called_once_with(url=test_url)

    @pytest.mark.parametrize(
        "service_func,api_url,scrape_url",
        [
            (
                get_actives,
                "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=MOST_ACTIVES",
                "https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50",
            ),
            (
                get_gainers,
                "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=DAY_GAINERS",
                "https://finance.yahoo.com/markets/stocks/gainers/?start=0&count=50",
            ),
            (
                get_losers,
                "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=DAY_LOSERS",
                "https://finance.yahoo.com/markets/stocks/losers/?start=0&count=50",
            ),
        ],
    )
    async def test_get_movers_services_fallback(self, service_func, api_url, scrape_url, bypass_cache, html_cache_manager):
        """Test get_movers service functions when API fetch fails and falls back to scraping"""
        test_count = MoverCount.FIFTY
        with patch("src.services.movers.get_movers.fetch_movers", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = Exception("API failure")
            # Determine context from URL
            if "most-active" in scrape_url:
                context = "movers_actives_50"
            elif "gainers" in scrape_url:
                context = "movers_gainers_50"
            elif "losers" in scrape_url:
                context = "movers_losers_50"
            else:
                context = "movers_unknown_50"
            html_content = html_cache_manager(scrape_url, context=context)
            with patch("src.services.movers.fetchers.movers_scraper.fetch", new_callable=AsyncMock) as mock_scrape_fetch:
                mock_scrape_fetch.return_value = html_content
                result = await service_func(test_count)

        assert isinstance(result, list)
        assert all(isinstance(m, MarketMover) for m in result)
        mock_fetch.assert_called_once_with(api_url)
        mock_scrape_fetch.assert_called_once_with(url=scrape_url)
