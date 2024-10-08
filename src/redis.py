import asyncio
import decimal
import functools
import gzip
import hashlib
import os
from datetime import date

import orjson
from aiohttp import ClientSession
from dotenv import load_dotenv
from pydantic import BaseModel
from redis import asyncio as aioredis

from src.schemas import TimeSeries, SimpleQuote, Quote, MarketMover, Index, News, MarketSector
from src.schemas.analysis import SMAData, EMAData, WMAData, VWMAData, RSIData, SRSIData, STOCHData, CCIData, MACDData, \
    ADXData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData, Analysis, Indicator
from src.schemas.sector import MarketSectorDetails
from src.utils import is_market_open

load_dotenv()

r = aioredis.Redis(
    connection_pool=aioredis.ConnectionPool(
        host=os.environ['REDIS_HOST'],
        port=int(os.environ['REDIS_PORT']),
        username=os.environ['REDIS_USERNAME'],
        password=os.environ['REDIS_PASSWORD'],
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


def cache(expire, after_market_expire=None):
    """
        This decorator caches the result of the function it decorates.

        The cache key is generated by hashing the function name and its arguments.
        The result is stored in a Redis cache with a specified expiration time.

        If the cache key exists in the Redis cache, the cached value is returned.
        If the cache key does not exist, the function is called and Redis stores the result.

        :param expire: The expiration time for the cache key
        :param after_market_expire: The expiration time for the cache key after the market closes
        :return: The result of the function or the cached value
        """
    lock = asyncio.Lock()

    async def cache_result(key, result, expire_time):
        """
        Caches the result in Redis based on the result type.
        :param key: the cache key (function name and hashed arguments)
        :param result: the response of the endpoint to cache
        :param expire_time: the expiration time for the cache key
        :return:
        """

        def handle_data(obj):
            if isinstance(obj, decimal.Decimal):
                return float(obj)
            elif isinstance(obj, BaseModel):
                return obj.model_dump()
            elif hasattr(obj, 'to_dict'):
                return obj.to_dict()
            raise TypeError

        # Cache the result in Redis
        if isinstance(result, dict):
            # Create instances of indicator classes for Technical Analysis
            data = gzip.compress(orjson.dumps(result, default=handle_data))
            await r.set(key, data, ex=expire_time)

        elif isinstance(result, (SimpleQuote, Quote, MarketMover, Index, News, MarketSector, MarketSectorDetails, TimeSeries)):
            # Caches a string to Redis
            await r.set(key, gzip.compress(result.model_dump_json(by_alias=True, exclude_none=True).encode()),
                        ex=expire_time)

        else:
            # Caches a list to Redis
            result_list = result
            if (isinstance(result, list) and result
                    and isinstance(result[0], (SimpleQuote, Quote, MarketMover, Index, News, MarketSector))):
                result_list = [item.dict() for item in result]
            for item in result_list:
                await r.rpush(key, gzip.compress(orjson.dumps(item)))

            await r.expire(key, expire_time)

    def decorator(func):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            async with lock:  # Locks to prevent duplicate caches

                # Filter out the ClientSession object from args and kwargs
                filtered_args = [arg for arg in args if not isinstance(arg, (ClientSession))]
                filtered_kwargs = {k: v for k, v in kwargs.items() if not isinstance(v, (ClientSession))}

                key = f"{func.__name__}:{hashlib.sha256(orjson.dumps((filtered_args, filtered_kwargs))).hexdigest()}"

                if await r.exists(key):
                    if await r.type(key) == b'string':
                        data = gzip.decompress(await r.get(key))
                        result = orjson.loads(data)
                    elif await r.type(key) == b'list':
                        result_list = []
                        for item in await r.lrange(key, 0, -1):
                            data = gzip.decompress(item)
                            result_list.append(orjson.loads(data))
                        result = result_list

                    # Create instances of indicator classes for Technical Analysis
                    if 'Technical Analysis' in result:
                        indicator_data = {}
                        indicator_name = result['type']
                        for key, value in result['Technical Analysis'].items():
                            if indicator_name in indicators:
                                indicator_value = indicators[indicator_name](**value)
                                indicator_data[date.fromisoformat(key)] = indicator_value
                        return Analysis(type=Indicator(indicator_name), indicators=indicator_data).model_dump(
                            exclude_none=True, by_alias=True,
                            serialize_as_any=True)
                    return result

                result = await func(*args, **kwargs)

                # Set the expiration time based on the market hours
                if after_market_expire and not is_market_open():
                    expire_time = after_market_expire
                else:
                    expire_time = expire

                await cache_result(key, result, expire_time)

                return result

        return wrapper

    return decorator
