import asyncio
import functools
from contextlib import contextmanager
from typing import Optional, Union, Callable
from unittest.mock import MagicMock, patch

import pytest
import pytest_asyncio
from fastapi import Request

from src.context import request_context


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


@pytest_asyncio.fixture
async def mock_context():
    """Setup mock request context and Redis"""
    # Create mock Redis instance
    mock_redis = MagicMock()
    mock_redis.exists.return_value = False  # Force cache miss by default
    mock_redis.pipeline.return_value = mock_redis  # Allow pipeline calls

    # Create mock request that mimics FastAPI Request
    mock_request = MagicMock(spec=Request)
    mock_request.app = MagicMock()
    mock_request.app.state = MagicMock()
    mock_request.app.state.redis = mock_redis

    # Set up the request context
    token = request_context.set(mock_request)

    try:
        yield mock_request
    finally:
        request_context.reset(token)  # Clean up the context after the test


# Helper function to bypass cache for testing
@contextmanager
def bypass_cache():
    """Temporarily bypass the cache mechanism during tests"""
    with patch('src.services.movers.get_movers.cache', return_value=lambda f: f):
        yield
