import asyncio
import functools
import gzip
import hashlib
import os
import time
from collections.abc import Callable
from datetime import date
from typing import Any, Optional, TypeVar, get_args, get_type_hints

import orjson
from pydantic import BaseModel
from redis import RedisError

from src import models as models_module
from src.clients.fetch_client import CurlFetchClient
from src.context import request_context
from src.utils.logging import get_logger, log_cache_operation
from src.utils.market import MarketSchedule, MarketStatus

T = TypeVar("T")
logger = get_logger(__name__)


class BaseCacheHandler:
    def get_expire_time(self) -> int:
        raise NotImplementedError

    async def get(self, key: str, return_type: type) -> Optional[Any]:
        raise NotImplementedError

    async def set(self, key: str, result: Any, expire_time: int) -> None:
        raise NotImplementedError


class RedisCacheHandler(BaseCacheHandler):
    def __init__(self, expire: int, market_closed_expire: Optional[int], market_schedule: MarketSchedule):
        self.expire = expire
        self.market_closed_expire = market_closed_expire
        self.market_schedule = market_schedule

    def get_expire_time(self) -> int:
        is_closed = self.market_schedule.get_market_status()[0] != MarketStatus.OPEN
        return self.market_closed_expire if (self.market_closed_expire and is_closed) else self.expire

    @staticmethod
    def handle_data(obj: Any) -> Any:
        if isinstance(obj, date):
            return obj.isoformat()
        if isinstance(obj, BaseModel):
            return {"__type__": obj.__class__.__name__, "data": obj.model_dump(by_alias=True, exclude_none=True)}
        raise TypeError(f"Object of type {type(obj)} is not JSON serializable")

    def serialize_data(self, data: Any) -> bytes:
        return gzip.compress(orjson.dumps(data, default=self.handle_data))

    def reconstruct_type(self, data: Any, type_hint: type) -> Any:
        if data is None:
            return None
        origin = getattr(type_hint, "__origin__", None)
        if origin is dict:
            key_type, value_type = get_args(type_hint)
            return {k: self.reconstruct_type(v, value_type) for k, v in data.items()}
        if origin is list:
            value_type = get_args(type_hint)[0]
            return [self.reconstruct_type(item, value_type) for item in data]
        if isinstance(data, dict) and "__type__" in data:
            model_name = data["__type__"]
            model_data = data["data"]
            # Dynamically build model_map from all BaseModel subclasses in models module
            model_map = {
                name: cls
                for name in dir(models_module)
                if not name.startswith("_") and isinstance(cls := getattr(models_module, name), type) and issubclass(cls, BaseModel) and cls is not BaseModel
            }
            if model_name in model_map:
                return model_map[model_name](**model_data)
        return data

    def deserialize_data(self, data: bytes, type_hint: Optional[type] = None) -> Any:
        deserialized = orjson.loads(gzip.decompress(data))
        if type_hint:
            return self.reconstruct_type(deserialized, type_hint)
        return deserialized

    async def get(self, key: str, return_type: type) -> Optional[Any]:
        try:
            request = request_context.get()
            redis = request.app.state.redis
            if not redis.exists(key):
                return None
            kt = redis.type(key)
            if kt == b"string":
                return self.deserialize_data(redis.get(key), return_type)
            if kt == b"list":
                items = redis.lrange(key, 0, -1)
                if not items:
                    return None
                item_type = get_args(return_type)[0] if hasattr(return_type, "__args__") else Any
                return [self.deserialize_data(item, item_type) for item in items]
            return None
        except (RedisError, orjson.JSONDecodeError):
            return None

    async def set(self, key: str, result: Any, expire_time: int) -> None:
        try:
            request = request_context.get()
            pipe = request.app.state.redis.pipeline()
            if isinstance(result, dict):
                pipe.set(key, self.serialize_data(result))
            elif isinstance(result, BaseModel):
                pipe.set(
                    key,
                    self.serialize_data(
                        {
                            "__type__": result.__class__.__name__,
                            "data": result.model_dump(by_alias=True, exclude_none=True),
                        }
                    ),
                )
            elif isinstance(result, list):
                pipe.delete(key)
                for item in result:
                    pipe.rpush(key, self.serialize_data(item))
            pipe.expire(key, expire_time)
            pipe.execute()
        except RedisError:
            pass


class MemCacheHandler(BaseCacheHandler):
    def __init__(self, expire: int, market_closed_expire: Optional[int], market_schedule: MarketSchedule):
        self.expire = expire
        self.market_closed_expire = market_closed_expire
        self.market_schedule = market_schedule
        self._store: dict[str, tuple[Any, float]] = {}

    def get_expire_time(self) -> int:
        is_closed = self.market_schedule.get_market_status()[0] != MarketStatus.OPEN
        return self.market_closed_expire if (self.market_closed_expire and is_closed) else self.expire

    async def get(self, key: str, return_type: type) -> Optional[Any]:
        entry = self._store.get(key)
        if not entry:
            return None
        result, expires = entry
        if time.time() < expires:
            return result
        del self._store[key]
        return None

    async def set(self, key: str, result: Any, expire_time: int) -> None:
        self._store[key] = (result, time.time() + expire_time)


def cache(
    expire: int = 0,
    market_closed_expire: Optional[int] = None,
    memcache: bool = False,
    market_schedule: MarketSchedule = MarketSchedule(),
) -> Callable[..., Any]:
    # Use memcache if specified or Redis is not available
    HandlerClass = MemCacheHandler if memcache or not os.getenv("REDIS_URL") else RedisCacheHandler
    handler = HandlerClass(expire, market_closed_expire, market_schedule)

    def decorator(func: Callable[..., Any]) -> Callable[..., Any]:
        return_type = get_type_hints(func).get("return")

        @functools.wraps(func)
        async def wrapper(*args: Any, **kwargs: Any) -> Any:
            # Skip caching if bypass is enabled or expire_time <= 0
            bypass_cache = os.getenv("BYPASS_CACHE")
            if (bypass_cache is not None and bypass_cache.lower() == "true") or handler.get_expire_time() <= 0:
                if bypass_cache and bypass_cache.lower() == "true":
                    logger.debug("Cache BYPASS - caching disabled via BYPASS_CACHE", extra={"function": func.__name__, "bypass_reason": "BYPASS_CACHE=true"})
                else:
                    logger.debug(
                        "Cache BYPASS - expire time is zero",
                        extra={"function": func.__name__, "bypass_reason": "expire_time <= 0", "expire_time": handler.get_expire_time()},
                    )
                return await func(*args, **kwargs)

            # Build cache key from serializable args
            from unittest.mock import AsyncMock, MagicMock, Mock

            mock_types = (CurlFetchClient, AsyncMock, MagicMock, Mock)
            filtered_args = [a for a in args if not isinstance(a, mock_types)]
            filtered_kwargs = {k: v for k, v in kwargs.items() if not isinstance(v, mock_types)}
            key_raw = orjson.dumps((filtered_args, filtered_kwargs))
            key = f"v1:{func.__name__}:{hashlib.sha256(key_raw).hexdigest()}"

            # Use a fresh lock per call bound to the correct loop
            lock = asyncio.Lock()
            async with lock:
                # Try cache
                cached = await handler.get(key, return_type)
                if cached is not None:
                    log_cache_operation(logger, "get", f"{func.__name__}:{key[-8:]}", hit=True)
                    return cached

                # Miss: call function and cache
                log_cache_operation(logger, "get", f"{func.__name__}:{key[-8:]}", hit=False)
                result = await func(*args, **kwargs)
                if result is not None:
                    log_cache_operation(logger, "set", f"{func.__name__}:{key[-8:]}")
                    await handler.set(key, result, handler.get_expire_time())
                return result

        return wrapper

    return decorator
