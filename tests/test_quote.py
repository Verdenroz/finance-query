import asyncio
from unittest.mock import patch, AsyncMock, MagicMock

import pytest
import pytest_asyncio
from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from src.services.quotes.get_quotes import get_quotes, get_simple_quotes
from tests.test_utils import timeout

# Test data
MOCK_QUOTE_RESPONSE = {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "138.85",
    "after_hours_price": "138.46",
    "change": "+3.56",
    "percent_change": "+2.63%",
    "open": "136.41",
    "high": "139.20",
    "low": "135.50",
    "year_high": "153.13",
    "year_low": "66.25",
    "volume": 194448420,
    "avg_volume": 244250595,
    "market_cap": "3.4T",
    "beta": "1.62",
    "pe": "54.88",
    "dividend": "0.04",
    "dividend_yield": "0.03%",
    "ex_dividend": "Dec 05, 2024",
    "earnings_date": "Feb 26, 2025",
    "last_dividend": "0.01",
    "sector": "Technology",
    "industry": "Semiconductors",
    "about": "NVIDIA Corporation provides graphics...",
    "employees": "29600",
    "logo": "https://logo.clearbit.com/nvidia.com"
}

MOCK_SIMPLE_QUOTE_RESPONSE = {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "138.85",
    "after_hours_price": "138.46",
    "change": "+3.56",
    "percent_change": "+2.63%",
    "logo": "https://logo.clearbit.com/nvidia.com"
}

QUOTE_TEST_TIMEOUT = 1


@pytest_asyncio.fixture
async def mock_auth_data():
    """Get real authentication data from Yahoo Finance"""
    with patch('src.dependencies.request_context', MagicMock()):
        cookies, crumb = "mock_cookies", "mock_crumb"
        return cookies, crumb


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_quotes_api_success(mock_auth_data):
    """Test successful API quote fetching"""
    cookies, crumb = mock_auth_data

    with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as mock_fetch:
        mock_response = [Quote(**MOCK_QUOTE_RESPONSE)]
        mock_fetch.return_value = mock_response

        result = await get_quotes(
            symbols=["NVDA"],
            cookies=cookies,
            crumb=crumb
        )

        assert len(result) == 1
        assert isinstance(result[0], Quote)
        assert result == mock_response
        mock_fetch.assert_awaited_once_with(["NVDA"], cookies, crumb)


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_quotes_fallback_to_scraping(mock_auth_data):
    """Test fallback to scraping when API fails"""
    cookies, crumb = mock_auth_data

    with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as mock_fetch:
        with patch('src.services.quotes.get_quotes.scrape_quotes', new_callable=AsyncMock) as mock_scrape:
            mock_fetch.side_effect = Exception("API Error")
            mock_scrape.return_value = [Quote(**MOCK_QUOTE_RESPONSE)]

            result = await get_quotes(
                symbols=["NVDA"],
                cookies=cookies,
                crumb=crumb
            )

            assert len(result) == 1
            assert isinstance(result[0], Quote)
            assert result == [Quote(**MOCK_QUOTE_RESPONSE)]
            mock_fetch.assert_awaited_once_with(["NVDA"], cookies, crumb)
            mock_scrape.assert_awaited_once_with(["NVDA"])


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_quotes_no_credentials():
    """Test immediate fallback to scraping with no credentials"""
    with patch('src.services.quotes.get_quotes.scrape_quotes', new_callable=AsyncMock) as mock_scrape:
        mock_scrape.return_value = [Quote(**MOCK_QUOTE_RESPONSE)]

        result = await get_quotes(
            symbols=["NVDA"],
            cookies="",
            crumb=""
        )

        assert len(result) == 1
        assert isinstance(result[0], Quote)
        assert result == [Quote(**MOCK_QUOTE_RESPONSE)]
        mock_scrape.assert_awaited_once_with(["NVDA"])


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_quotes_unknown_symbol(mock_auth_data):
    """Test unknown symbol handling"""
    cookies, crumb = mock_auth_data
    unknown_symbol = "UNKNOWN"

    with patch('src.services.quotes.get_quotes.fetch_quotes', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.side_effect = HTTPException(status_code=404, detail=f"Symbol not found: {unknown_symbol}")

        with pytest.raises(HTTPException) as exc_info:
            await get_quotes(
                symbols=[unknown_symbol],
                cookies=cookies,
                crumb=crumb
            )

        assert exc_info.value.status_code == 404
        assert f"Symbol not found: {unknown_symbol}" in str(exc_info.value.detail)
        mock_fetch.assert_awaited_once_with([unknown_symbol], cookies, crumb)


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_simple_quotes_api_success(mock_auth_data):
    """Test successful API simple quote fetching"""
    cookies, crumb = mock_auth_data

    with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]

        result = await get_simple_quotes(
            symbols=["NVDA"],
            cookies=cookies,
            crumb=crumb
        )

        assert len(result) == 1
        assert isinstance(result[0], SimpleQuote)
        assert result == [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]
        mock_fetch.assert_awaited_once_with(["NVDA"], cookies, crumb)


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_simple_quotes_fallback_to_scraping(mock_auth_data):
    """Test fallback to scraping when API fails"""
    cookies, crumb = mock_auth_data

    with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as mock_fetch:
        with patch('src.services.quotes.get_quotes.scrape_simple_quotes', new_callable=AsyncMock) as mock_scrape:
            mock_fetch.side_effect = Exception("API Error")
            mock_scrape.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]

            result = await get_simple_quotes(
                symbols=["NVDA"],
                cookies=cookies,
                crumb=crumb
            )

            assert len(result) == 1
            assert isinstance(result[0], SimpleQuote)
            assert result == [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]
            mock_fetch.assert_awaited_once_with(["NVDA"], cookies, crumb)
            mock_scrape.assert_awaited_once_with(["NVDA"])


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_simple_quotes_no_credentials():
    """Test immediate fallback to scraping with no credentials"""
    with patch('src.services.quotes.get_quotes.scrape_simple_quotes', new_callable=AsyncMock) as mock_scrape:
        mock_scrape.return_value = [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]

        result = await get_simple_quotes(
            symbols=["NVDA"],
            cookies="",
            crumb=""
        )

        assert len(result) == 1
        assert isinstance(result[0], SimpleQuote)
        assert result == [SimpleQuote(**MOCK_SIMPLE_QUOTE_RESPONSE)]
        mock_scrape.assert_awaited_once_with(["NVDA"])


@timeout(QUOTE_TEST_TIMEOUT)
async def test_get_simple_quotes_unknown_symbol(mock_auth_data):
    """Test unknown symbol handling"""
    cookies, crumb = mock_auth_data
    unknown_symbol = "UNKNOWN"

    with patch('src.services.quotes.get_quotes.fetch_simple_quotes', new_callable=AsyncMock) as mock_fetch:
        mock_fetch.side_effect = HTTPException(status_code=404, detail=f"Symbol not found: {unknown_symbol}")

        with pytest.raises(HTTPException) as exc_info:
            await get_simple_quotes(
                symbols=[unknown_symbol],
                cookies=cookies,
                crumb=crumb
            )

        assert exc_info.value.status_code == 404
        assert f"Symbol not found: {unknown_symbol}" in str(exc_info.value.detail)
        mock_fetch.assert_awaited_once_with([unknown_symbol], cookies, crumb)
