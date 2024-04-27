import functools
import hashlib
import os
from datetime import datetime

import orjson
import pytz
import redis
from dotenv import load_dotenv

from src.schemas import TimeSeries, HistoricalData, Quote, Stock

load_dotenv()

r = redis.Redis(
    host=os.getenv("REDIS_HOST"),
    port=os.getenv("REDIS_PORT"),
    db=0,
    username=os.getenv("REDIS_USERNAME"),
    password=os.getenv("REDIS_PASSWORD"),
)


def is_market_open() -> bool:
    now = datetime.now(pytz.timezone('US/Eastern'))
    open_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=9, minute=30, second=0)
    close_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=16, minute=0, second=0)
    # Check if current time is within market hours and it's a weekday
    return open_time <= now <= close_time and 0 <= now.weekday() < 5


def cache(expire, check_market=False):
    def decorator(func):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            key = f"{func.__name__}:{hashlib.sha256(orjson.dumps((args, kwargs))).hexdigest()}"

            if r.exists(key):
                result_list = []
                for item in r.lrange(key, 0, -1):
                    result_list.append(orjson.loads(item))
                return result_list

            result = await func(*args, **kwargs)

            if isinstance(result, list) and result and isinstance(result[0], (TimeSeries, Stock, Quote)):
                result_list = [item.dict() for item in result]
            elif isinstance(result, (TimeSeries, Stock, Quote)):
                result_list = result.dict()
            else:
                result_list = result

            if not check_market or (check_market and not is_market_open()):
                for item in result_list:
                    r.rpush(key, orjson.dumps(item))
                r.expire(key, expire)

            return result

        return wrapper

    return decorator
