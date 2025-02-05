from concurrent.futures import ThreadPoolExecutor
from datetime import datetime

import psutil

# Initialize thread pool
thread_pool = ThreadPoolExecutor(max_workers=psutil.cpu_count(logical=True) * 2)


def get_adaptive_chunk_size() -> int:
    """Calculate adaptive chunk size based on system resources."""
    cpu_count = psutil.cpu_count()
    memory_info = psutil.virtual_memory()
    available_memory = memory_info.available

    base_chunk_size = 5
    chunk_size = base_chunk_size * cpu_count * (available_memory // (512 * 1024 * 1024))
    return max(base_chunk_size, min(chunk_size, 100))


def is_within_pre_market_time(pre_market_time: int) -> bool:
    return int(datetime.now().timestamp()) <= pre_market_time


def is_within_post_market_time(post_market_time: int) -> bool:
    return int(datetime.now().timestamp()) >= post_market_time


def format_date(date_string: str) -> str | None:
    if not date_string:
        return None
    try:
        date = datetime.fromtimestamp(int(date_string))
        return date.strftime("%b %d, %Y")
    except (ValueError, TypeError):
        return None


def format_percent(value) -> str | None:
    if not value or not isinstance(value, dict) or "raw" not in value:
        return None
    return f"{value['raw'] * 100:.2f}%"


def format_change(value: str) -> str:
    if value and value[0] not in {'-', '+'}:
        return f"+{value}"
    return value


def get_fmt(obj, key) -> str | None:
    if not obj or not isinstance(obj, dict):
        return None
    value = obj.get(key, {})
    if isinstance(value, dict):
        value = value.get("fmt", None)
    return value


def get_raw(obj, key) -> str | None:
    if not obj or not isinstance(obj, dict):
        return None
    value = obj.get(key, {})
    if isinstance(value, dict):
        value = value.get("raw", None)
    return value


def get_morningstar_risk_rating(raw_risk: int) -> str | None:
    risk_mapping = {
        1: "Below Average",
        2: "Average",
        3: "Above Average",
        4: "High",
    }
    return risk_mapping.get(raw_risk, None)
