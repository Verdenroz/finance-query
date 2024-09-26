import os

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.websockets import WebSocket

from src.redis import r

DEMO_API_KEY = "FinanceQueryDemoAWSHT"
ADMIN_API_KEY = os.getenv("ADMIN_API_KEY")
RATE_LIMIT = 2000  # requests per day


async def validate_api_key(api_key: str) -> bool:
    if api_key not in {DEMO_API_KEY, ADMIN_API_KEY}:
        return False
    return True


async def enforce_rate_limit(api_key: str) -> bool:
    """
    Enforce rate limiting for the demo API key
    :param api_key: should be FinanceQueryDemoAWSHT, but can be any string

    :return: True if the rate limit has not been exceeded, False otherwise
    """
    if api_key == DEMO_API_KEY:
        key = f"rate_limit:{api_key}"
        count = await r.get(key)
        if count is None:
            # Set the key to 1 and expire it in 24 hours if it doesn't exist
            await r.set(key, 1, ex=86400)
        elif int(count) >= RATE_LIMIT:
            # Rate limit exceeded
            return False
        else:
            # Increment the count
            await r.incr(key)

    return True


async def create_rate_limit_metadata(api_key: str) -> dict:
    """
    Create metadata for the rate limit for the websocket routes since they cannot return headers
    :param api_key: Should be FinanceQueryDemoAWSHT, but can be any string

    :return: metadata for the rate limit as a dictionary
    """
    key = f"rate_limit:{api_key}"
    count = await r.get(key)
    remaining = RATE_LIMIT - int(count) if count else RATE_LIMIT
    reset = await r.ttl(key)
    return {
        "metadata": {
            "rate limit": RATE_LIMIT,
            "remaining requests": remaining,
            "reset": reset
        }
    }


async def validate_websocket(websocket: WebSocket) -> tuple[bool, dict]:
    """
    Validate the websocket connection and enforce rate limiting
    :param websocket: the websocket connection

    :return: a tuple containing a boolean indicating if the connection is valid and a dictionary of metadata
    """
    # Skip rate limiting if the environment variable is set
    if not os.getenv('USE_SECURITY', 'False') == 'True':
        return True, {}

    api_key = websocket.headers.get("x-api-key")
    is_demo = api_key == DEMO_API_KEY

    if not await validate_api_key(api_key):
        await websocket.close(code=1008, reason="Not authenticated")
        return False, {}

    if is_demo:
        if not await enforce_rate_limit(api_key):
            await websocket.close(code=1008, reason="Rate limit exceeded")
            return False, {}
        metadata = await create_rate_limit_metadata(api_key)
        return True, metadata

    return True, {}


class RateLimitMiddleware(BaseHTTPMiddleware):
    """
    Middleware to enforce rate limiting on the API
    """

    def __init__(self, app):
        super().__init__(app)

    async def dispatch(self, request: Request, call_next):
        """"
        Check the API key and enforce rate limiting

        Demo API key has a rate limit of 500 requests per day

        Admin API key has no rate limit

        All other API keys are not authenticated and will receive a 403 Forbidden response
        """
        api_key = request.headers.get("x-api-key")

        if not await validate_api_key(api_key):
            return JSONResponse({"detail": "Not authenticated"}, status_code=403)

        if api_key == DEMO_API_KEY:
            if not await enforce_rate_limit(api_key):
                return JSONResponse({"detail": "Rate limit exceeded"}, status_code=429)

            key = f"rate_limit:{api_key}"
            count = await r.get(key)
            remaining = RATE_LIMIT - int(count)
            response = await call_next(request)
            response.headers["X-RateLimit-Limit"] = str(RATE_LIMIT)
            response.headers["X-RateLimit-Remaining"] = str(remaining)
            response.headers["X-RateLimit-Reset"] = str(await r.ttl(key))
            return response

        elif ADMIN_API_KEY and api_key == ADMIN_API_KEY:
            # No rate limit for admin API key
            response = await call_next(request)
            return response
