import os
from decimal import Decimal
from enum import Enum

import json
import redis
import hashlib
import functools

from dotenv import load_dotenv

from src.schemas import TimeSeries, HistoricalData

load_dotenv()


class TimePeriod(Enum):
    THREE_MONTHS = "3M"
    SIX_MONTHS = "6M"
    YTD = "YTD"
    YEAR = "1Y"
    FIVE_YEARS = "5Y"
    MAX = "max"


class Interval(Enum):
    DAILY = "daily"
    WEEKLY = "weekly"
    MONTHLY = "monthly"


r = redis.Redis(
    host=os.getenv("REDIS_HOST"),
    port=os.getenv("REDIS_PORT"),
    db=0,
    username=os.getenv("REDIS_USERNAME"),
    password=os.getenv("REDIS_PASSWORD"),
    decode_responses=True,
)


class TimeSeriesEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, TimeSeries):
            return {'history': obj.history}
        elif isinstance(obj, Decimal):
            return str(obj)
        elif isinstance(obj, TimePeriod):
            return obj.value
        elif isinstance(obj, Interval):
            return obj.value
        return super().default(obj)


def cache(expire):
    """
    This decorator caches the result of the function it decorates.

    The cache key is generated by hashing the function name and its arguments.
    The result is stored in a Redis cache with a specified expiration time.

    If the cache key exists in the Redis cache, the cached value is returned.
    If the cache key does not exist, the function is called and Redis stores the result.

    :param expire: The expiration time for the cache key
    :return: The result of the function or the cached value
    """
    def decorator(func):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            # Generate a unique key for the function and its arguments
            key = f"{func.__name__}:{hashlib.sha256(json.dumps((args, kwargs), cls=TimeSeriesEncoder).encode()).hexdigest()}"

            # Check if the key exists in the Redis cache
            if (result := r.get(key)) is not None:
                # If the key exists, retrieve the cached value and return it
                result_dict = json.loads(result)
                if 'history' in result_dict:
                    result_dict['history'] = {
                        k: HistoricalData(
                            open=Decimal(v['open']),
                            high=Decimal(v['high']),
                            low=Decimal(v['low']),
                            adj_close=Decimal(v['adj_close']),
                            volume=int(v['volume'])
                        ) for k, v in result_dict['history'].items()
                    }
                return TimeSeries(**result_dict)

            # If the key does not exist, call the function
            result = await func(*args, **kwargs)

            # Convert the TimeSeries object into a dictionary before storing it in the Redis cache
            result_dict = result.to_dict() if isinstance(result, TimeSeries) else result

            # Store the result in the Redis cache with an expiration time
            r.set(key, json.dumps(result_dict, cls=TimeSeriesEncoder), ex=expire)

            # Return the result
            return result

        return wrapper

    return decorator
