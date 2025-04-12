import asyncio
import functools
import gzip
import hashlib
import os
from datetime import date
from typing import Optional, Any, TypeVar, Callable, get_type_hints, get_args

import orjson
from aiohttp import ClientSession
from async_lru import alru_cache
from pydantic import BaseModel
from redis import RedisError

from src.context import request_context
from src.market import MarketSchedule, MarketStatus
from src.models import HistoricalData, SimpleQuote, Quote, MarketMover, MarketIndex, News, MarketSector
from src.models.sector import MarketSectorDetails

T = TypeVar('T')


class RedisCacheHandler:
    """Handles caching operations with type-aware reconstruction of cached data."""

    def __init__(self, expire: int, market_closed_expire: Optional[int], market_schedule: MarketSchedule):
        self.expire = expire
        self.market_closed_expire = market_closed_expire
        self.market_schedule = market_schedule

    def get_expire_time(self) -> int:
        """Determine the appropriate expiration time based on market status."""
        is_closed = self.market_schedule.get_market_status()[0] != MarketStatus.OPEN
        return self.market_closed_expire if (self.market_closed_expire and is_closed) else self.expire

    @staticmethod
    def handle_data(obj: Any) -> Any:
        """Convert special types to JSON-serializable formats."""
        if isinstance(obj, date):
            return obj.isoformat()
        if isinstance(obj, BaseModel):
            return {
                "__type__": obj.__class__.__name__,
                "data": obj.model_dump(by_alias=True, exclude_none=True)
            }
        raise TypeError(f"Object of type {type(obj)} is not JSON serializable")

    def serialize_data(self, data: Any) -> bytes:
        """Serialize and compress data for storage."""
        return gzip.compress(orjson.dumps(data, default=self.handle_data))

    def reconstruct_type(self, data: Any, type_hint: type) -> Any:
        """Recursively reconstruct complex types from cached data."""
        if data is None:
            return None

        # Handle container types (dict, list)
        origin = getattr(type_hint, "__origin__", None)
        if origin is not None:
            if origin is dict:
                key_type, value_type = get_args(type_hint)
                return {k: self.reconstruct_type(v, value_type) for k, v in data.items()}
            elif origin is list:
                value_type = get_args(type_hint)[0]
                return [self.reconstruct_type(item, value_type) for item in data]

        # Handle BaseModel instances
        if isinstance(data, dict) and "__type__" in data:
            model_name = data["__type__"]
            model_data = data["data"]

            # Map model names to actual classes
            model_map = {
                "Quote": Quote,
                "SimpleQuote": SimpleQuote,
                "MarketMover": MarketMover,
                "MarketIndex": MarketIndex,
                "MarketSector": MarketSector,
                "MarketSectorDetails": MarketSectorDetails,
                "HistoricalData": HistoricalData,
                "News": News,
            }

            if model_name in model_map:
                return model_map[model_name](**model_data)

        return data

    def deserialize_data(self, data: bytes, type_hint: Optional[type] = None) -> Any:
        """Decompress and deserialize data with type reconstruction."""
        deserialized = orjson.loads(gzip.decompress(data))
        if type_hint:
            return self.reconstruct_type(deserialized, type_hint)
        return deserialized

    async def get_from_redis(self, key: str, return_type: type) -> Optional[Any]:
        """Retrieve and reconstruct data from Redis."""
        try:
            request = request_context.get()
            redis = request.app.state.redis

            if not redis.exists(key):
                return None

            key_type = redis.type(key)
            if key_type == b'string':
                cached_data = redis.get(key)
                return self.deserialize_data(cached_data, return_type)
            elif key_type == b'list':
                items = redis.lrange(key, 0, -1)
                if not items:
                    return None
                # For lists, we need to get the type of the list items
                item_type = get_args(return_type)[0] if hasattr(return_type, '__args__') else Any
                return [self.deserialize_data(item, item_type) for item in items]

            return None

        except (RedisError, orjson.JSONDecodeError) as e:
            print(f"Redis error: {str(e)}")
            return None

    def store_in_redis(self, key: str, result: Any, expire_time: int) -> None:
        """Store data in Redis with type information preservation."""
        try:
            request = request_context.get()
            pipe = request.app.state.redis.pipeline()

            if isinstance(result, dict):
                pipe.set(key, self.serialize_data(result))
            elif isinstance(result, BaseModel):
                pipe.set(key, self.serialize_data({
                    "__type__": result.__class__.__name__,
                    "data": result.model_dump(by_alias=True, exclude_none=True)
                }))
            elif isinstance(result, list):
                pipe.delete(key)
                serialized_items = [self.serialize_data(item) for item in result]
                if serialized_items:
                    pipe.rpush(key, *serialized_items)

            pipe.expire(key, expire_time)
            pipe.execute()

        except RedisError as e:
            print(f"Redis error: {str(e)}")


def cache(
        expire: int = 0,
        market_closed_expire: Optional[int] = None,
        memcache: bool = False,
        market_schedule: MarketSchedule = MarketSchedule(),
) -> Callable:
    """Cache decorator with type-aware reconstruction of cached data."""
    handler = RedisCacheHandler(expire, market_closed_expire, market_schedule)

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        # Get the return type hint from the function
        return_type = get_type_hints(func).get('return')

        if memcache or not os.getenv("REDIS_URL"):
            return alru_cache(maxsize=512, ttl=handler.get_expire_time())(func)

        @functools.wraps(func)
        async def wrapper(*args: Any, **kwargs: Any) -> T:
            async with asyncio.Lock():
                # Filter out non-serializable arguments
                filtered_args = [arg for arg in args if not isinstance(arg, ClientSession)]
                filtered_kwargs = {k: v for k, v in kwargs.items()
                                   if not isinstance(v, ClientSession)}

                key = (f"{func.__name__}:"
                       f"{hashlib.sha256(orjson.dumps((filtered_args, filtered_kwargs))).hexdigest()}")

                # Try to get from cache with proper type reconstruction
                cached_result = await handler.get_from_redis(key, return_type)
                if cached_result is not None:
                    return cached_result

                # If not in cache, call the function and store result
                result = await func(*args, **kwargs)
                if result is not None:
                    handler.store_in_redis(key, result, handler.get_expire_time())
                return result

        return wrapper

    return decorator