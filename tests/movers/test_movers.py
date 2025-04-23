import hashlib
from pathlib import Path
from unittest.mock import AsyncMock, patch

import orjson
import pytest
import requests

from src.models.marketmover import MoverCount, MarketMover
from src.services.movers import get_actives, get_gainers, get_losers
from src.services.movers.fetchers import fetch_movers, scrape_movers
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


class TestMovers:
    @pytest.fixture
    def movers_html(self):
        """
        Fixture that provides a function to get cached HTML content for URLs.
        """
        cache_dir = Path(__file__).resolve().parent.parent / "data" / "movers"
        cache_dir.mkdir(parents=True, exist_ok=True)
        html_cache = {}

        def get_cached_html(url):
            if url in html_cache:
                return html_cache[url]

            filename = hashlib.md5(url.encode()).hexdigest()
            if 'most-active' in url:
                endpoint = 'actives'
            elif 'gainers' in url:
                endpoint = 'gainers'
            elif 'losers' in url:
                endpoint = 'losers'
            else:
                endpoint = 'unknown'

            count = url.split('count=')[-1]
            cache_file = cache_dir / f"{endpoint}_{count}_{filename}.html"

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
        Fixture that provides mock Yahoo Finance API responses for movers.
        """

        def get_mock_response(count=50):
            mock_items = []
            for i in range(1, count + 1):
                mock_items.append({
                    "symbol": f"SYM{i}",
                    "longName": f"Company {i} Long",
                    "shortName": f"Company {i}",
                    "regularMarketPrice": {"fmt": f"{100 + i:.2f}"},
                    "regularMarketChange": {"fmt": f"+{i * 0.5:.2f}"},
                    "regularMarketChangePercent": {"fmt": f"+{i * 0.5:.2f}%"}
                })
            return {
                "finance": {
                    "result": [
                        {"quotes": mock_items}
                    ]
                }
            }

        return get_mock_response

    @pytest.mark.parametrize("count", ["25", "50", "100"])
    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_success(self, test_client, count, endpoint, mock_yahoo_auth, monkeypatch):
        """Test successful movers retrieval with different count values"""
        mock_service = AsyncMock(return_value=COUNT_RESPONSE_MAP[count])
        if endpoint == "actives":
            monkeypatch.setattr("src.routes.movers.get_actives", mock_service)
        elif endpoint == "gainers":
            monkeypatch.setattr("src.routes.movers.get_gainers", mock_service)
        else:
            monkeypatch.setattr("src.routes.movers.get_losers", mock_service)

        response = test_client.get(f"{VERSION}/{endpoint}?count={count}")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == int(count)
        assert data == COUNT_RESPONSE_MAP[count]
        mock_service.assert_awaited_once_with(MoverCount(count))

    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_default_count(self, test_client, endpoint, mock_yahoo_auth, monkeypatch):
        """Test movers retrieval with default count (50)"""
        mock_service = AsyncMock(return_value=MOCK_MOVER_RESPONSE_FIFTY)
        if endpoint == "actives":
            monkeypatch.setattr("src.routes.movers.get_actives", mock_service)
        elif endpoint == "gainers":
            monkeypatch.setattr("src.routes.movers.get_gainers", mock_service)
        else:
            monkeypatch.setattr("src.routes.movers.get_losers", mock_service)

        response = test_client.get(f"{VERSION}/{endpoint}")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 50
        assert data == MOCK_MOVER_RESPONSE_FIFTY
        mock_service.assert_awaited_once_with(MoverCount.FIFTY)

    @pytest.mark.parametrize("endpoint", ENDPOINTS)
    def test_get_movers_invalid_count(self, test_client, endpoint, mock_yahoo_auth):
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
        test_url = (
            "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?"
            "count=50&formatted=true&scrIds=MOST_ACTIVES"
        )
        test_count = 50
        mock_response = mock_api_response(test_count)
        expected_movers = []
        for item in mock_response["finance"]["result"][0]["quotes"]:
            mover = MarketMover(
                symbol=item["symbol"],
                name=item["longName"],
                price=item["regularMarketPrice"]["fmt"],
                change=item["regularMarketChange"]["fmt"],
                percent_change=item["regularMarketChangePercent"]["fmt"]
            )
            expected_movers.append(mover)

        with patch('src.services.movers.fetchers.movers_api.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = orjson.dumps(mock_response)
            result = await fetch_movers(test_url)

        assert isinstance(result, list)
        assert len(result) == test_count
        assert all(isinstance(m, MarketMover) for m in result)
        expected_params = {
            "fields": (
                "symbol,longName,shortName,regularMarketPrice,"
                "regularMarketChange,regularMarketChangePercent"
            ),
        }
        mock_fetch.assert_called_once_with(url=test_url, params=expected_params)
        for i, mover in enumerate(result):
            assert mover.symbol == f"SYM{i + 1}"
            assert mover.name == f"Company {i + 1} Long"
            assert mover.price == f"{100 + i + 1:.2f}"
            assert mover.change == f"+{(i + 1) * 0.5:.2f}"
            assert mover.percent_change == f"+{(i + 1) * 0.5:.2f}%"

    async def test_scrape_movers(self, movers_html, bypass_cache):
        """Test scrape_movers function with cached HTML content"""
        test_url = (
            "https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50"
        )
        html_content = movers_html(test_url)
        with patch('src.services.movers.fetchers.movers_scraper.fetch', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = html_content
            result = await scrape_movers(test_url)

        assert isinstance(result, list)
        assert len(result) == 50
        assert all(isinstance(m, MarketMover) for m in result)
        assert all(m.change.startswith(('+', '-', '0')) for m in result)
        assert all(m.percent_change.endswith('%') for m in result)
        mock_fetch.assert_called_once_with(url=test_url)

    @pytest.mark.parametrize("service_func,api_url,scrape_url", [
        (get_actives,
         "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=MOST_ACTIVES",
         "https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50"),
        (get_gainers,
         "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=DAY_GAINERS",
         "https://finance.yahoo.com/markets/stocks/gainers/?start=0&count=50"),
        (get_losers,
         "https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count=50&formatted=true&scrIds=DAY_LOSERS",
         "https://finance.yahoo.com/markets/stocks/losers/?start=0&count=50")
    ])
    async def test_get_movers_services_fallback(self, service_func, api_url, scrape_url, bypass_cache,
                                                movers_html):
        """Test get_movers service functions when API fetch fails and falls back to scraping"""
        test_count = MoverCount.FIFTY
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.side_effect = Exception("API failure")
            html_content = movers_html(scrape_url)
            with patch('src.services.movers.fetchers.movers_scraper.fetch',
                       new_callable=AsyncMock) as mock_scrape_fetch:
                mock_scrape_fetch.return_value = html_content
                result = await service_func(test_count)

        assert isinstance(result, list)
        assert all(isinstance(m, MarketMover) for m in result)
        mock_fetch.assert_called_once_with(api_url)
        mock_scrape_fetch.assert_called_once_with(url=scrape_url)
