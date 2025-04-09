import asyncio
from datetime import date
from unittest.mock import MagicMock, AsyncMock, patch

import pytest
from redis import RedisError

from src.cache import cache, RedisCacheHandler
from src.context import request_context
from src.market import MarketSchedule, MarketStatus
from src.models import SimpleQuote


class TestRedisCacheHandler:
    @pytest.fixture
    def market_schedule(self):
        """Mock market schedule that returns OPEN by default"""
        schedule = MagicMock(spec=MarketSchedule)
        schedule.get_market_status.return_value = (MarketStatus.OPEN, None)
        return schedule

    @pytest.fixture
    def closed_market_schedule(self):
        """Mock market schedule that returns CLOSED by default"""
        schedule = MagicMock(spec=MarketSchedule)
        schedule.get_market_status.return_value = (MarketStatus.CLOSED, None)
        return schedule

    @pytest.fixture
    def handler(self, market_schedule):
        """Create a RedisCacheHandler with default expire time"""
        return RedisCacheHandler(expire=60, market_closed_expire=300, market_schedule=market_schedule)

    @pytest.fixture
    def closed_market_handler(self, closed_market_schedule):
        """Create a RedisCacheHandler with extended expire time during closed market"""
        return RedisCacheHandler(expire=60, market_closed_expire=300, market_schedule=closed_market_schedule)

    @pytest.fixture
    def mock_request(self):
        """Create a mock request with Redis client"""
        redis_mock = MagicMock()
        request_mock = MagicMock()
        request_mock.app.state.redis = redis_mock
        return request_mock

    def test_get_expire_time(self, handler, closed_market_handler):
        """Test that correct expiration times are returned based on market status"""
        # Open market should use default expire time
        assert handler.get_expire_time() == 60

        # Closed market should use extended expire time
        assert closed_market_handler.get_expire_time() == 300

    def test_handle_data(self, handler):
        """Test handling of special data types for serialization"""
        # Test date handling
        test_date = date(2023, 1, 1)
        assert handler.handle_data(test_date) == "2023-01-01"

        # Test BaseModel handling
        quote = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        result = handler.handle_data(quote)
        assert result["__type__"] == "SimpleQuote"
        assert result["data"]["symbol"] == "AAPL"
        assert result["data"]["name"] == "Apple"
        assert result["data"]["price"] == "150.0"
        assert result["data"]["change"] == "+1.00"
        assert result["data"]["percentChange"] == "+0.69%"

        # Test unsupported type
        with pytest.raises(TypeError):
            handler.handle_data(set())

    def test_serialize_deserialize_roundtrip(self, handler):
        """Test serialization and deserialization roundtrip"""
        # Test with simple dict
        data = {"key": "value", "number": 42}
        serialized = handler.serialize_data(data)
        deserialized = handler.deserialize_data(serialized)
        assert deserialized == data

        # Test with BaseModel
        quote = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        serialized = handler.serialize_data(quote)
        deserialized = handler.deserialize_data(serialized, SimpleQuote)
        assert isinstance(deserialized, SimpleQuote)
        assert deserialized.symbol == "AAPL"
        assert deserialized.price == "150.0"

        # Test with list of models
        quotes = [
            SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%"),
            SimpleQuote(symbol="MSFT", name="Microsoft", price="250.0", change="-2.00", percent_change="-0.80%")
        ]
        serialized = handler.serialize_data(quotes)
        deserialized = handler.deserialize_data(serialized, list[SimpleQuote])
        assert isinstance(deserialized, list)
        assert all(isinstance(q, SimpleQuote) for q in deserialized)
        assert deserialized[0].symbol == "AAPL"
        assert deserialized[1].symbol == "MSFT"

    async def test_get_from_redis(self, handler, mock_request):
        """Test retrieving and reconstructing data from Redis"""
        # Setup the request context
        redis_mock = mock_request.app.state.redis
        token = request_context.set(mock_request)
        quote = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        quote2 = SimpleQuote(symbol="MSFT", name="Microsoft", price="250.0", change="-2.00", percent_change="-0.80%")

        try:
            # Case 1: Key doesn't exist
            redis_mock.exists.return_value = False
            result = await handler.get_from_redis("nonexistent_key", SimpleQuote)
            assert result is None

            # Case 2: String key type with SimpleQuote
            redis_mock.exists.return_value = True
            redis_mock.type.return_value = b'string'

            # Mock serialized SimpleQuote
            serialized = handler.serialize_data({
                "__type__": "SimpleQuote",
                "data": quote.dict()
            })
            redis_mock.get.return_value = serialized

            result = await handler.get_from_redis("quote_key", SimpleQuote)
            assert isinstance(result, SimpleQuote)
            assert result.symbol == "AAPL"
            assert result.price == "150.0"

            # Case 3: List key type with list of SimpleQuotes
            redis_mock.type.return_value = b'list'
            quote1 = handler.serialize_data({
                "__type__": "SimpleQuote",
                "data": quote.dict()
            })
            quote2 = handler.serialize_data({
                "__type__": "SimpleQuote",
                "data": quote2.dict()
            })
            redis_mock.lrange.return_value = [quote1, quote2]

            result = await handler.get_from_redis("quotes_list", list[SimpleQuote])
            assert isinstance(result, list)
            assert len(result) == 2
            assert all(isinstance(q, SimpleQuote) for q in result)
            assert result[0].symbol == "AAPL"
            assert result[1].symbol == "MSFT"

            # Case 4: Redis error
            # Reset the previous behavior
            redis_mock.reset_mock()
            redis_mock.exists.return_value = True
            redis_mock.type.return_value = b'string'
            redis_mock.get.side_effect = RedisError("Redis error")

            result = await handler.get_from_redis("error_key", SimpleQuote)
            assert result is None

        finally:
            request_context.reset(token)

    def test_store_in_redis(self, handler, mock_request):
        """Test storing data in Redis with type preservation"""
        # Setup the request context and mock pipeline
        redis_mock = mock_request.app.state.redis
        pipeline_mock = MagicMock()
        redis_mock.pipeline.return_value = pipeline_mock
        token = request_context.set(mock_request)
        quote = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        quote2 = SimpleQuote(symbol="MSFT", name="Microsoft", price="250.0", change="-2.00", percent_change="-0.80%")

        try:
            # Case 1: Store a dictionary
            data = {"key": "value", "number": 42}
            handler.store_in_redis("dict_key", data, 60)
            pipeline_mock.set.assert_called_with("dict_key", handler.serialize_data(data))
            pipeline_mock.expire.assert_called_with("dict_key", 60)
            pipeline_mock.execute.assert_called_once()

            # Reset mocks
            pipeline_mock.reset_mock()

            # Case 2: Store a BaseModel
            handler.store_in_redis("quote_key", quote, 120)
            pipeline_mock.set.assert_called_once()
            pipeline_mock.expire.assert_called_with("quote_key", 120)
            pipeline_mock.execute.assert_called_once()

            # Reset mocks
            pipeline_mock.reset_mock()

            # Case 3: Store a list
            quotes = [
                quote,
                quote2
            ]
            handler.store_in_redis("quotes_list", quotes, 180)
            pipeline_mock.delete.assert_called_with("quotes_list")
            pipeline_mock.rpush.assert_called_once()
            pipeline_mock.expire.assert_called_with("quotes_list", 180)
            pipeline_mock.execute.assert_called_once()

            # Case 4: Redis error
            pipeline_mock.reset_mock()
            pipeline_mock.execute.side_effect = RedisError("Redis error")
            handler.store_in_redis("error_key", data, 60)
            # No exception should be raised, error should be handled

        finally:
            request_context.reset(token)


class TestCacheDecorator:
    @pytest.fixture
    def mock_redis_env(self, monkeypatch):
        """Set up environment for Redis cache"""
        monkeypatch.setenv("REDIS_URL", "redis://localhost:6379")

    @pytest.fixture
    def market_schedule(self):
        """Mock market schedule"""
        schedule = MagicMock(spec=MarketSchedule)
        schedule.get_market_status.return_value = (MarketStatus.OPEN, None)
        return schedule

    @pytest.fixture
    def setup_context(self):
        """Set up request context with Redis client"""
        redis_mock = MagicMock()
        request_mock = MagicMock()
        request_mock.app.state.redis = redis_mock
        token = request_context.set(request_mock)
        yield redis_mock
        request_context.reset(token)

    @patch('src.cache.RedisCacheHandler.get_from_redis')
    @patch('src.cache.RedisCacheHandler.store_in_redis')
    async def test_redis_cache(self, mock_store, mock_get, mock_redis_env, setup_context, market_schedule):
        """Test that Redis cache works correctly"""
        redis_mock = setup_context

        # Configure the mock to indicate cache miss then hit
        mock_get.side_effect = [None,  SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")]

        # Create a cached function
        @cache(expire=60, market_closed_expire=300, market_schedule=market_schedule)
        async def get_quote(symbol: str) -> SimpleQuote:
            return  SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")

        # First call - should cache miss and call the function
        result1 = await get_quote("AAPL")
        assert result1.symbol == "AAPL"
        assert result1.price == "150.0"
        mock_get.assert_called_once()
        mock_store.assert_called_once()

        # Reset call count
        mock_get.reset_mock()
        mock_store.reset_mock()

        # Second call - should be a cache hit
        result2 = await get_quote("AAPL")
        assert result2.symbol == "AAPL"
        assert result2.price == "150.0"
        mock_get.assert_called_once()
        mock_store.assert_not_called()  # Should not store again

    @patch('os.getenv')
    async def test_memcache(self, mock_getenv, market_schedule):
        """Test that memcache (alru_cache) works correctly"""
        # Configure no REDIS_URL to force memcache
        mock_getenv.return_value = None

        call_count = 0

        @cache(expire=60, memcache=True, market_schedule=market_schedule)
        async def get_data(key: str) -> dict:
            nonlocal call_count
            call_count += 1
            return {"key": key, "value": call_count}

        # Check the cache info before any calls
        assert hasattr(get_data, 'cache_info')
        info_before = get_data.cache_info()
        assert info_before.hits == 0
        assert info_before.misses == 0

        # First call - should be a cache miss
        result1 = await get_data("test")
        assert result1 == {"key": "test", "value": 1}
        assert call_count == 1

        # Get cache info after first call
        info_after_miss = get_data.cache_info()
        assert info_after_miss.hits == 0
        assert info_after_miss.misses == 1

        # Second call with same key - should be a cache hit
        result2 = await get_data("test")
        assert result2 == {"key": "test", "value": 1}  # Value should be the same
        assert call_count == 1  # Function should not be called again

        # Get cache info after cache hit
        info_after_hit = get_data.cache_info()
        assert info_after_hit.hits == 1
        assert info_after_hit.misses == 1

        # Call with different key - should be a cache miss
        result3 = await get_data("different")
        assert result3 == {"key": "different", "value": 2}
        assert call_count == 2

        # Get final cache info
        info_final = get_data.cache_info()
        assert info_final.hits == 1
        assert info_final.misses == 2

    @patch('src.cache.asyncio.Lock')
    async def test_cache_concurrency(self, mock_lock, bypass_cache):
        """Test that cache uses a lock to prevent concurrent function execution"""
        # Setup mock lock
        mock_lock_instance = AsyncMock()
        mock_lock_context = AsyncMock()
        mock_lock_instance.__aenter__.return_value = mock_lock_context
        mock_lock.return_value = mock_lock_instance

        call_count = 0

        @cache(expire=60, memcache=False)
        async def slow_function() -> int:
            nonlocal call_count
            call_count += 1
            await asyncio.sleep(0.1)
            return call_count

        # Call the function concurrently
        tasks = [slow_function() for _ in range(5)]
        results = await asyncio.gather(*tasks)

        # Lock should be acquired once per call
        assert mock_lock_instance.__aenter__.call_count == 5
        assert mock_lock_instance.__aexit__.call_count == 5

        # Function should be called 5 times (bypassing cache)
        assert call_count == 5
        assert all(result in [1, 2, 3, 4, 5] for result in results)

    @patch('os.getenv')
    async def test_cache_expiry(self, mock_getenv):
        """Test that memcache respects TTL expiry"""
        # Configure no REDIS_URL to force memcache
        mock_getenv.return_value = None

        call_count = 0

        @cache(expire=1, memcache=True)  # 1 second TTL
        async def get_data() -> int:
            nonlocal call_count
            call_count += 1
            return call_count

        # First call
        result1 = await get_data()
        assert result1 == 1
        assert call_count == 1

        # Second call (immediately) - should be cached
        result2 = await get_data()
        assert result2 == 1
        assert call_count == 1

        # Wait for cache to expire
        await asyncio.sleep(1.1)

        # Third call after expiry - should call function again
        result3 = await get_data()
        assert result3 == 2
        assert call_count == 2