import inspect
import logging
from collections.abc import Awaitable, Callable
from functools import wraps
from typing import ParamSpec, TypeVar

from fastapi import HTTPException

P = ParamSpec("P")
R = TypeVar("R")


def retry(
    fallback: Callable[..., Awaitable[R]],
    *,
    retries: int = 0,
) -> Callable[[Callable[P, Awaitable[R]]], Callable[P, Awaitable[R]]]:
    logger = logging.getLogger(__name__)

    # parameters the fallback is willing to accept
    _fallback_params = set(inspect.signature(fallback).parameters)

    def decorator(primary: Callable[P, Awaitable[R]]) -> Callable[P, Awaitable[R]]:
        primary_sig = inspect.signature(primary)

        @wraps(primary)
        async def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:  # type: ignore
            attempt = 0
            while True:
                try:
                    return await primary(*args, **kwargs)
                except Exception as exc:
                    if isinstance(exc, HTTPException):
                        logger.exception("HTTPException in %s: %s", primary.__name__, exc)
                        raise

                    attempt += 1
                    logger.exception(
                        "Error in %s (attempt %d/%d): %s",
                        primary.__name__,
                        attempt,
                        retries + 1,
                        exc,
                    )
                    if attempt <= retries:
                        continue  # try the primary again

                    # ---- build args/kwargs for the fallback ----
                    bound = primary_sig.bind_partial(*args, **kwargs)
                    bound.apply_defaults()

                    # Pass original kwargs directly if they're in fallback parameters
                    fallback_kwargs = {}
                    # First add parameters from bound arguments
                    for k, v in bound.arguments.items():
                        if k in _fallback_params:
                            fallback_kwargs[k] = v
                    # Then add any kwargs that might be directly usable by fallback
                    for k, v in kwargs.items():
                        if k in _fallback_params and k not in fallback_kwargs:
                            fallback_kwargs[k] = v

                    logger.info("Switching to fallback %s with kwargs %s", fallback.__name__, fallback_kwargs)
                    return await fallback(**fallback_kwargs)

        return wrapper

    return decorator
