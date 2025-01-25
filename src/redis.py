import asyncio
import decimal
import functools
import gzip
import hashlib
import os
from datetime import date

import orjson
from aiohttp import ClientSession
from async_lru import alru_cache
from dotenv import load_dotenv
from pydantic import BaseModel
from redis import asyncio as aioredis, RedisError

from src.market import MarketSchedule, MarketStatus
from src.schemas import TimeSeries, SimpleQuote, Quote, MarketMover, Index, News, MarketSector
from src.schemas.analysis import SMAData, EMAData, WMAData, VWMAData, RSIData, SRSIData, STOCHData, CCIData, MACDData, \
    ADXData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData, Analysis, Indicator
from src.schemas.sector import MarketSectorDetails

load_dotenv()

r = aioredis.Redis(
    connection_pool=aioredis.ConnectionPool(
        host=os.environ.get('REDIS_HOST', 'localhost'),
        port=int(os.environ.get('REDIS_PORT', 6379)),
        username=os.environ.get('REDIS_USERNAME'),
        password=os.environ.get('REDIS_PASSWORD'),
        max_connections=10000
    ),
    auto_close_connection_pool=True,
    single_connection_client=True,
)

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


def cache(expire, market_closed_expire=None, memcache=False, market_schedule=MarketSchedule()):
    """
        This decorator caches the result of the function it decorates.

        The cache key is generated by hashing the function name and its arguments.
        The result is stored in a Redis cache with a specified expiration time.

        If the cache key exists in the Redis cache, the cached value is returned.
        If the cache key does not exist, the function is called and Redis stores the result.

        :param expire: The expiration time for the cache key
        :param market_closed_expire: The expiration time for the cache key after the market closes
        :param memcache: Flag to use in-memory caching instead of Redis
        :param market_schedule: DI for MarketSchedule class to check if market is open/closed

        :return: The result of the function or the cached value
        """
    use_redis = os.getenv('USE_REDIS', 'False').lower() == 'true' and not memcache
    # Determine expiration time
    is_closed = market_schedule.get_market_status()[0] != MarketStatus.OPEN
    expire_time = market_closed_expire if (market_closed_expire and is_closed) else expire

    async def cache_in_redis(key, result):
        """
        Caches the result in Redis based on the result type.
        :param key: the cache key (function name and hashed arguments)
        :param result: the response of the endpoint to cache
        :return:
        """

        def handle_data(obj):
            """Handle special data types for JSON serialization."""
            if isinstance(obj, decimal.Decimal):
                return float(obj)
            elif isinstance(obj, BaseModel):
                return obj.model_dump()
            elif hasattr(obj, 'to_dict'):
                return obj.to_dict()
            raise TypeError(f"Object of type {type(obj)} is not JSON serializable")

        try:
            async with await r.pipeline() as pipe:
                if isinstance(result, dict):
                    # Handle dictionary data
                    data = gzip.compress(orjson.dumps(result, default=handle_data))
                    await pipe.set(key, data)
                    await pipe.expire(key, expire_time)

                elif isinstance(result, (
                        SimpleQuote, Quote, MarketMover, Index, News, MarketSector, MarketSectorDetails, TimeSeries)):
                    # Handle Pydantic models
                    data = gzip.compress(result.model_dump_json(by_alias=True, exclude_none=True).encode())
                    await pipe.set(key, data)
                    await pipe.expire(key, expire_time)

                else:
                    # Handle lists
                    result_list = result
                    if (isinstance(result, list) and result and
                            isinstance(result[0], (SimpleQuote, Quote, MarketMover, Index, News, MarketSector))):
                        result_list = [item.dict() for item in result]

                    # Delete any existing key before adding new list
                    await pipe.delete(key)
                    for item in result_list:
                        await pipe.rpush(key, gzip.compress(orjson.dumps(item)))
                    await pipe.expire(key, expire_time)

                await pipe.execute()
        except RedisError as e:
            # Log the error and continue without caching
            print(f"Redis caching error: {str(e)}")
            return None

    def decorator(func):
        # Use alru_cache for in-memory caching
        if not use_redis:
            return alru_cache(maxsize=512, ttl=expire_time)(func)

        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            lock = asyncio.Lock()
            async with lock:
                # Filter out non-serializable objects
                filtered_args = [arg for arg in args if not isinstance(arg, ClientSession)]
                filtered_kwargs = {k: v for k, v in kwargs.items() if not isinstance(v, ClientSession)}

                # Generate cache key
                key = f"{func.__name__}:{hashlib.sha256(orjson.dumps((filtered_args, filtered_kwargs))).hexdigest()}"

                # Check cache
                try:
                    if await r.exists(key):
                        key_type = await r.type(key)

                        if key_type == b'string':
                            data = gzip.decompress(await r.get(key))
                            result = orjson.loads(data)
                        elif key_type == b'list':
                            result_list = []
                            items = await r.lrange(key, 0, -1)
                            if not items:  # Handle empty list case
                                raise KeyError("Cache key exists but no items found")

                            for item in items:
                                data = gzip.decompress(item)
                                result_list.append(orjson.loads(data))
                            result = result_list
                        else:
                            raise ValueError(f"Unexpected Redis key type: {key_type}")

                        # Handle Technical Analysis indicators
                        if isinstance(result, dict) and 'Technical Analysis' in result:
                            indicator_data = {}
                            indicator_name = result['type']
                            for key, value in result['Technical Analysis'].items():
                                if indicator_name in indicators:
                                    indicator_value = indicators[indicator_name](**value)
                                    indicator_data[date.fromisoformat(key)] = indicator_value
                            return Analysis(
                                type=Indicator(indicator_name),
                                indicators=indicator_data
                            ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

                        return result

                except orjson.JSONDecodeError as e:
                    # If cache is corrupted, delete it and continue
                    await r.delete(key)
                    print(f"Cache corruption detected: {str(e)}")

                # Get fresh result
                result = await func(*args, **kwargs)
                if result is None:
                    return None

                # Cache the result
                await cache_in_redis(key, result)

                return result

        return wrapper

    return decorator
