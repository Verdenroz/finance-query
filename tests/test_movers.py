from unittest.mock import patch, AsyncMock

from src.models.marketmover import MarketMover, MoverCount
from src.services.movers.get_movers import get_actives, get_gainers, get_losers
from tests.test_utils import timeout, bypass_cache, mock_context

# Test data
MOCK_MOVER_RESPONSE_TWENTY_FIVE = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percent_change": f"+{i * 0.5:.2f}%"
    } for i in range(1, 26)
]

MOCK_MOVER_RESPONSE_FIFTY = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percent_change": f"+{i * 0.5:.2f}%"
    } for i in range(1, 51)
]

MOCK_MOVER_RESPONSE_HUNDRED = [
    {
        "symbol": f"SYM{i}",
        "name": f"Company {i}",
        "price": f"{100 + i:.2f}",
        "change": f"+{i * 0.5:.2f}",
        "percent_change": f"+{i * 0.5:.2f}%"
    } for i in range(1, 101)
]

MOVERS_TEST_TIMEOUT = 1


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_actives_api_success(mock_context):
    """Test successful API active stocks fetching"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            mock_response = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_FIFTY]
            mock_fetch.return_value = mock_response

            result = await get_actives(count=MoverCount.FIFTY)

            assert len(result) == 50
            assert all(isinstance(item, MarketMover) for item in result)
            assert result == mock_response
            mock_fetch.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_actives_fallback_to_scraping(mock_context):
    """Test fallback to scraping when API fails for active stocks"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.movers.get_movers.scrape_movers', new_callable=AsyncMock) as mock_scrape:
                mock_fetch.side_effect = Exception("API Error")
                mock_scrape.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_FIFTY]

                result = await get_actives(count=MoverCount.FIFTY)

                assert len(result) == 50
                assert all(isinstance(item, MarketMover) for item in result)
                mock_fetch.assert_awaited_once()
                mock_scrape.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_gainers_api_success(mock_context):
    """Test successful API gainers fetching"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            mock_response = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_TWENTY_FIVE]
            mock_fetch.return_value = mock_response

            result = await get_gainers(count=MoverCount.TWENTY_FIVE)

            assert len(result) == 25
            assert all(isinstance(item, MarketMover) for item in result)
            assert result == mock_response
            mock_fetch.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_gainers_fallback_to_scraping(mock_context):
    """Test fallback to scraping when API fails for gainers"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.movers.get_movers.scrape_movers', new_callable=AsyncMock) as mock_scrape:
                mock_fetch.side_effect = Exception("API Error")
                mock_scrape.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_TWENTY_FIVE]

                result = await get_gainers(count=MoverCount.TWENTY_FIVE)

                assert len(result) == 25
                assert all(isinstance(item, MarketMover) for item in result)
                mock_fetch.assert_awaited_once()
                mock_scrape.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_losers_api_success(mock_context):
    """Test successful API losers fetching"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            mock_response = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_HUNDRED]
            mock_fetch.return_value = mock_response

            result = await get_losers(count=MoverCount.HUNDRED)

            assert len(result) == 100
            assert all(isinstance(item, MarketMover) for item in result)
            assert result == mock_response
            mock_fetch.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_get_losers_fallback_to_scraping(mock_context):
    """Test fallback to scraping when API fails for losers"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            with patch('src.services.movers.get_movers.scrape_movers', new_callable=AsyncMock) as mock_scrape:
                mock_fetch.side_effect = Exception("API Error")
                mock_scrape.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_HUNDRED]

                result = await get_losers(count=MoverCount.HUNDRED)

                assert len(result) == 100
                assert all(isinstance(item, MarketMover) for item in result)
                mock_fetch.assert_awaited_once()
                mock_scrape.assert_awaited_once()


@timeout(MOVERS_TEST_TIMEOUT)
async def test_different_mover_counts(mock_context):
    """Test that different MoverCount values are correctly used"""
    with bypass_cache('src.services.movers.get_movers.cache'):
        with patch('src.services.movers.get_movers.fetch_movers', new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_TWENTY_FIVE]
            # Test with MoverCount.TWENTY_FIVE
            response = await get_actives(count=MoverCount.TWENTY_FIVE)
            assert len(response) == 25
            assert all(isinstance(item, MarketMover) for item in response)
            mock_fetch.assert_awaited_once()

            # Test with MoverCount.FIFTY (default)
            mock_fetch.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_FIFTY]
            mock_fetch.reset_mock()
            response = await get_actives()
            assert len(response) == 50
            assert all(isinstance(item, MarketMover) for item in response)
            mock_fetch.assert_awaited_once()

            # Test with MoverCount.HUNDRED
            mock_fetch.reset_mock()
            mock_fetch.return_value = [MarketMover(**item) for item in MOCK_MOVER_RESPONSE_HUNDRED]
            response = await get_actives(count=MoverCount.HUNDRED)
            assert len(response) == 100
            assert all(isinstance(item, MarketMover) for item in response)
            mock_fetch.assert_awaited_once()
