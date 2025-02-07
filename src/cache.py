import asyncio
import decimal
import functools
import gzip
import hashlib
import os
from datetime import date
from typing import Optional, Any, TypeVar, Callable, Union

import orjson
from aiohttp import ClientSession
from async_lru import alru_cache
from dotenv import load_dotenv
from pydantic import BaseModel
from redis import RedisError

from src.context import request_context
from src.market import MarketSchedule, MarketStatus
from src.models import HistoricalData, SimpleQuote, Quote, MarketMover, Index, News, MarketSector
from src.models.analysis import SMAData, EMAData, WMAData, VWMAData, RSIData, SRSIData, STOCHData, CCIData, MACDData, \
    ADXData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData, Analysis, Indicator
from src.models.sector import MarketSectorDetails

load_dotenv()

indicators = {
    "SMA": SMAData,
    "EMA": EMAData,
    "WMA": WMAData,
    "VWMA": VWMAData,
    "RSI": RSIData,
    "SRSI": SRSIData,
    "STOCH": STOCHData,
    "CCI": CCIData,
    "MACD": MACDData,
    "ADX": ADXData,
    "AROON": AROONData,
    "BBANDS": BBANDSData,
    "OBV": OBVData,
    "SUPERTREND": SuperTrendData,
    "ICHIMOKU": IchimokuData
}

T = TypeVar('T')


class CacheHandler:
    """Handles caching operations with different storage backends and data types."""

    def __init__(self, expire: int, market_closed_expire: Optional[int], market_schedule: MarketSchedule):
        self.expire = expire
        self.market_closed_expire = market_closed_expire
        self.market_schedule = market_schedule
        self.use_redis = os.getenv('USE_REDIS', 'False').lower() == 'true'

    def get_expire_time(self) -> int:
        """Determine the appropriate expiration time based on market status."""
        is_closed = self.market_schedule.get_market_status()[0] != MarketStatus.OPEN
        return self.market_closed_expire if (self.market_closed_expire and is_closed) else self.expire

    @staticmethod
    def handle_data(obj: Any) -> Any:
        """Convert special types to JSON-serializable formats."""
        if isinstance(obj, decimal.Decimal):
            return float(obj)
        if isinstance(obj, BaseModel):
            return obj.model_dump()
        if hasattr(obj, 'to_dict'):
            return obj.to_dict()
        raise TypeError(f"Object of type {type(obj)} is not JSON serializable")

    def serialize_data(self, data: Any) -> bytes:
        """Serialize and compress data for storage."""
        return gzip.compress(orjson.dumps(data, default=self.handle_data))

    @staticmethod
    def deserialize_data(data: bytes) -> Any:
        """Decompress and deserialize data from storage."""
        return orjson.loads(gzip.decompress(data))

    def store_in_redis(self, key: str, result: Any, expire_time: int) -> None:
        """Store data in Redis with appropriate serialization based on type."""
        try:
            request = request_context.get()
            pipe = request.app.state.redis.pipeline()

            if isinstance(result, dict):
                if result and isinstance(next(iter(result.values())), HistoricalData):
                    # Handle HistoricalData dictionary
                    serialized = {k: v.model_dump(by_alias=True, exclude_none=True)
                                  for k, v in result.items()}
                    pipe.set(key, self.serialize_data(serialized))
                else:
                    pipe.set(key, self.serialize_data(result))

            elif isinstance(result, (SimpleQuote, Quote, MarketMover, Index, News,
                                     MarketSector, MarketSectorDetails)):
                pipe.set(key, gzip.compress(
                    result.model_dump_json(by_alias=True, exclude_none=True).encode()
                ))

            elif isinstance(result, list):
                pipe.delete(key)
                if result and isinstance(result[0], (SimpleQuote, Quote, MarketMover,
                                                     Index, News, MarketSector)):
                    items = [item.dict() for item in result]
                else:
                    items = result
                for item in items:
                    pipe.rpush(key, self.serialize_data(item))

            pipe.expire(key, expire_time)
            pipe.execute()

        except RedisError as e:
            print(f"Redis caching error: {str(e)}")
            return None

    @staticmethod
    def process_cached_result(result: dict) -> Union[dict, Analysis]:
        """Process and transform cached results if needed."""
        if isinstance(result, dict) and 'Technical Analysis' in result:
            indicator_data = {}
            indicator_name = result['type']
            if indicator_name in indicators:
                for key, value in result['Technical Analysis'].items():
                    indicator_value = indicators[indicator_name](**value)
                    indicator_data[date.fromisoformat(key)] = indicator_value
                return Analysis(
                    type=Indicator(indicator_name),
                    indicators=indicator_data
                ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
        return result


def cache(
        expire: int = 0,
        market_closed_expire: Optional[int] = None,
        memcache: bool = False,
        market_schedule: MarketSchedule = MarketSchedule(),
) -> Callable:
    """
    Cache decorator that supports both Redis and in-memory caching.

    :param expire: default expiration time
    :param market_closed_expire: when market is closed
    :param memcache: whether to use in-memory caching
    :param market_schedule: market schedule for determining cache expiration
    """
    handler = CacheHandler(expire, market_closed_expire, market_schedule)

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        if memcache or not handler.use_redis:
            return alru_cache(maxsize=512, ttl=handler.get_expire_time())(func)

        @functools.wraps(func)
        async def wrapper(*args: Any, **kwargs: Any) -> T:
            async with asyncio.Lock():
                # Filter out non-serializable arguments
                filtered_args = [arg for arg in args if not isinstance(arg, ClientSession)]
                filtered_kwargs = {k: v for k, v in kwargs.items()
                                   if not isinstance(v, ClientSession)}

                # Generate cache key
                key = (f"{func.__name__}:"
                       f"{hashlib.sha256(orjson.dumps((filtered_args, filtered_kwargs))).hexdigest()}")

                try:
                    request = request_context.get()
                    redis = request.app.state.redis

                    if redis.exists(key):
                        key_type = redis.type(key)
                        if key_type == b'string':
                            result = handler.deserialize_data(redis.get(key))
                            return handler.process_cached_result(result)
                        elif key_type == b'list':
                            items = redis.lrange(key, 0, -1)
                            if not items:
                                raise KeyError("Cache key exists but no items found")
                            return [handler.deserialize_data(item) for item in items]
                        else:
                            raise ValueError(f"Unexpected Redis key type: {key_type}")

                except orjson.JSONDecodeError as e:
                    redis.delete(key)
                    print(f"Cache corruption detected: {str(e)}")

                # Cache miss - execute function and store result
                result = await func(*args, **kwargs)
                if result is not None:
                    handler.store_in_redis(key, result, handler.get_expire_time())
                return result

        return wrapper

    return decorator
