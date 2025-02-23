import asyncio
import functools
from typing import Optional, Union, Callable

import pytest


def timeout(
        seconds: Union[int, float],
        message: Optional[str] = None,
        debug: bool = False
):
    """
    Decorator for async test functions to add timeout functionality. Fail the test if it takes longer than the specified
    number of seconds to complete.

    Args:
        seconds: Number of seconds before timeout
        message: Custom message to show on timeout
        debug: If True, prints debug information when timeout occurs
    """

    def decorator(func: Callable):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            timeout_message = message or f"Test '{func.__name__}' timed out after {seconds} seconds"
            try:
                async with asyncio.timeout(seconds):
                    return await func(*args, **kwargs)
            except asyncio.TimeoutError:
                pytest.fail(reason=timeout_message, pytrace=debug)

        return wrapper

    return decorator
