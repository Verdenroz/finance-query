import asyncio
from datetime import date
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from redis import RedisError

from src.cache import MemCacheHandler, RedisCacheHandler, cache
from src.context import request_context
from src.market import MarketSchedule, MarketStatus
from src.models import SimpleQuote


class TestRedisCacheHandler:
    @pytest.fixture
    def market_schedule(self):
        sched = MagicMock(spec=MarketSchedule)
        sched.get_market_status.return_value = (MarketStatus.OPEN, None)
        return sched

    @pytest.fixture
    def closed_market_schedule(self):
        sched = MagicMock(spec=MarketSchedule)
        sched.get_market_status.return_value = (MarketStatus.CLOSED, None)
        return sched

    @pytest.fixture
    def handler(self, market_schedule):
        return RedisCacheHandler(expire=60, market_closed_expire=300, market_schedule=market_schedule)

    @pytest.fixture
    def closed_handler(self, closed_market_schedule):
        return RedisCacheHandler(expire=60, market_closed_expire=300, market_schedule=closed_market_schedule)

    @pytest.fixture
    def mock_request(self):
        redis_mock = MagicMock()
        request_mock = MagicMock()
        request_mock.app.state.redis = redis_mock
        return request_mock

    def test_get_expire_time(self, handler, closed_handler):
        assert handler.get_expire_time() == 60
        assert closed_handler.get_expire_time() == 300

    def test_handle_data(self, handler):
        # date handling
        d = date(2023, 1, 1)
        assert handler.handle_data(d) == "2023-01-01"
        # BaseModel handling
        q = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        out = handler.handle_data(q)
        assert out["__type__"] == "SimpleQuote"
        assert out["data"]["symbol"] == "AAPL"
        # unsupported type
        with pytest.raises(TypeError):
            handler.handle_data(set())

    def test_serialize_deserialize_roundtrip(self, handler):
        data = {"key": "value", "num": 42}
        ser = handler.serialize_data(data)
        deser = handler.deserialize_data(ser)
        assert deser == data

        q = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
        ser = handler.serialize_data(q)
        deser = handler.deserialize_data(ser, SimpleQuote)
        assert isinstance(deser, SimpleQuote)
        assert deser.symbol == "AAPL"

    async def test_get_from_redis(self, handler, mock_request):
        token = request_context.set(mock_request)
        try:
            redis = mock_request.app.state.redis
            # missing key
            redis.exists.return_value = False
            assert await handler.get("missing", SimpleQuote) is None

            # string key
            redis.exists.return_value = True
            redis.type.return_value = b"string"
            q = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
            ser = handler.serialize_data(
                {"__type__": "SimpleQuote", "data": q.model_dump(by_alias=True, exclude_none=True)}
            )
            redis.get.return_value = ser
            out = await handler.get("str", SimpleQuote)
            assert isinstance(out, SimpleQuote)
            assert out.symbol == "AAPL"

            # list key
            redis.type.return_value = b"list"
            ser1 = handler.serialize_data(
                {"__type__": "SimpleQuote", "data": q.model_dump(by_alias=True, exclude_none=True)}
            )
            ser2 = handler.serialize_data(
                {"__type__": "SimpleQuote", "data": q.model_dump(by_alias=True, exclude_none=True)}
            )
            redis.lrange.return_value = [ser1, ser2]
            lst = await handler.get("lst", list[SimpleQuote])
            assert isinstance(lst, list)
            assert all(isinstance(i, SimpleQuote) for i in lst)

            # redis error on string path
            redis.type.return_value = b"string"
            redis.get.side_effect = RedisError("boom")
            assert await handler.get("err", SimpleQuote) is None

            # Case 5: Redis error on list path
            redis.type.return_value = b"list"
            redis.lrange.side_effect = RedisError("boom")
            assert await handler.get("err_list", list[SimpleQuote]) is None
        finally:
            request_context.reset(token)

    async def test_set_in_redis(self, handler, mock_request):
        """Test storing data in Redis with type preservation via async set()"""
        token = request_context.set(mock_request)
        try:
            redis = mock_request.app.state.redis
            pipe = MagicMock()
            redis.pipeline.return_value = pipe
            data = {"foo": "bar"}

            # await the async set method
            await handler.set("d", data, 10)
            pipe.set.assert_called_with("d", handler.serialize_data(data))
            pipe.expire.assert_called_with("d", 10)
            pipe.execute.assert_called_once()

            pipe.reset_mock()
            q = SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")
            await handler.set("q", q, 20)
            # BaseModel path
            assert pipe.set.call_count == 1
            pipe.expire.assert_called_with("q", 20)

            pipe.reset_mock()
            # list path
            await handler.set("l", [q, q], 30)
            pipe.delete.assert_called_with("l")
            assert pipe.rpush.call_count == 2
            pipe.expire.assert_called_with("l", 30)

            # error path should not raise
            pipe.execute.side_effect = RedisError("fail")
            await handler.set("e", data, 5)
        finally:
            request_context.reset(token)


class TestMemCacheHandler:
    @pytest.fixture
    def market_schedule(self):
        sched = MagicMock(spec=MarketSchedule)
        sched.get_market_status.return_value = (MarketStatus.OPEN, None)
        return sched

    @pytest.fixture
    def closed_market_schedule(self):
        sched = MagicMock(spec=MarketSchedule)
        sched.get_market_status.return_value = (MarketStatus.CLOSED, None)
        return sched

    @pytest.fixture
    def handler(self, market_schedule):
        return MemCacheHandler(expire=1, market_closed_expire=5, market_schedule=market_schedule)

    @pytest.fixture
    def closed_handler(self, closed_market_schedule):
        return MemCacheHandler(expire=1, market_closed_expire=5, market_schedule=closed_market_schedule)

    def test_get_expire_time(self, handler, closed_handler):
        assert handler.get_expire_time() == 1
        assert closed_handler.get_expire_time() == 5

    async def test_set_and_get(self, handler):
        assert await handler.get("x", str) is None

        await handler.set("x", "v1", expire_time=1)
        assert await handler.get("x", str) == "v1"

        await asyncio.sleep(1.1)
        assert await handler.get("x", str) is None


class TestCacheDecoratorRedis:
    @pytest.fixture
    def mock_redis_env(self, monkeypatch):
        monkeypatch.setenv("REDIS_URL", "redis://localhost:6379")

    @pytest.fixture
    def market_schedule(self):
        sched = MagicMock(spec=MarketSchedule)
        sched.get_market_status.return_value = (MarketStatus.OPEN, None)
        return sched

    @pytest.fixture
    def setup_context(self):
        redis = MagicMock()
        req = MagicMock()
        req.app.state.redis = redis
        token = request_context.set(req)
        yield redis
        request_context.reset(token)

    @patch("src.cache.RedisCacheHandler.get")
    @patch("src.cache.RedisCacheHandler.set")
    async def test_redis_cache(self, mock_set, mock_get, mock_redis_env, setup_context, market_schedule):
        mock_get.side_effect = [
            None,
            SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%"),
        ]

        @cache(expire=60, market_closed_expire=300, market_schedule=market_schedule)
        async def get_quote(symbol: str) -> SimpleQuote:
            return SimpleQuote(symbol="AAPL", name="Apple", price="150.0", change="+1.00", percent_change="+0.69%")

        r1 = await get_quote("AAPL")
        assert r1.symbol == "AAPL"
        mock_get.assert_called_once()
        mock_set.assert_called_once()

        mock_get.reset_mock()
        mock_set.reset_mock()

        r2 = await get_quote("AAPL")
        assert r2.symbol == "AAPL"
        mock_get.assert_called_once()
        mock_set.assert_not_called()

    @patch("asyncio.Lock")
    async def test_cache_concurrency(self, mock_lock, mock_redis_env):
        # Ensure we stub Redis branch too
        redis = MagicMock()
        req = MagicMock()
        req.app.state.redis = redis
        token = request_context.set(req)
        try:
            lock_inst = AsyncMock()
            mock_lock.return_value = lock_inst
            call_count = 0

            @cache(expire=60, memcache=False)
            async def slow():
                nonlocal call_count
                call_count += 1
                await asyncio.sleep(0)
                return call_count

            tasks = [slow() for _ in range(3)]
            results = await asyncio.gather(*tasks)

            # Lock should be acquired for each invocation
            assert lock_inst.__aenter__.call_count == 3
            assert lock_inst.__aexit__.call_count == 3
            # Function should have been called 3 times
            assert call_count == 3
            # All callers receive the final call_count (due to concurrency)
            assert all(r == call_count for r in results)
        finally:
            request_context.reset(token)


class TestCacheDecoratorMemcache:
    @pytest.fixture(autouse=True)
    def no_redis(self, monkeypatch):
        monkeypatch.setenv("REDIS_URL", "")
        yield
        monkeypatch.delenv("REDIS_URL", raising=False)

    async def test_memcache_honors_ttl(self):
        count = 0

        @cache(expire=1, memcache=True)
        async def f(x: str) -> dict:
            nonlocal count
            count += 1
            return {"x": x, "cnt": count}

        r1 = await f("a")
        assert r1 == {"x": "a", "cnt": 1}

        r2 = await f("a")
        assert count == 1
        assert r2 == {"x": "a", "cnt": 1}
        await asyncio.sleep(1.1)

        r3 = await f("a")
        assert count == 2
        assert r3 == {"x": "a", "cnt": 2}

    async def test_memcache_isolated_by_args(self):
        count = 0

        @cache(expire=5, memcache=True)
        async def f(x: int) -> int:
            nonlocal count
            count += 1
            return count

        a = await f(1)
        b = await f(2)
        assert a == 1 and b == 2
        assert await f(1) == 1
        assert await f(2) == 2
        assert count == 2
