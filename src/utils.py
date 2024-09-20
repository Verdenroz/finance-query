from datetime import datetime

import pytz


def is_market_open() -> bool:
    now = datetime.now(pytz.timezone('US/Eastern'))
    open_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=9, minute=30, second=0)
    close_time = datetime.now(pytz.timezone('US/Eastern')).replace(hour=16, minute=0, second=0)
    # Check if current time is within market hours and it's a weekday
    return open_time <= now <= close_time and 0 <= now.weekday() < 5



