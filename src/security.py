import os

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse

from src.redis import r

DEMO_API_KEY = "FinanceQueryDemoAWSHT"
ADMIN_API_KEY = os.getenv("ADMIN_API_KEY")
RATE_LIMIT = 500  # requests per day


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

        if api_key == DEMO_API_KEY:
            key = f"rate_limit:{api_key}"
            count = await r.get(key)
            if count is None:
                # Set the key and set the expiration time to 24 hours
                await r.set(key, 1, ex=86400)
                count = 0
            elif int(count) >= RATE_LIMIT:
                # Rate limit exceeded
                return JSONResponse({"detail": "Rate limit exceeded"}, status_code=429)
            else:
                # Increment the count
                await r.incr(key)

            remaining = RATE_LIMIT - int(count) - 1
            response = await call_next(request)
            response.headers["X-RateLimit-Limit"] = str(RATE_LIMIT)
            response.headers["X-RateLimit-Remaining"] = str(remaining)
            response.headers["X-RateLimit-Reset"] = str(await r.ttl(key))
            return response

        elif ADMIN_API_KEY and api_key == ADMIN_API_KEY:
            # No rate limit for admin API key
            response = await call_next(request)
            return response
        else:
            # When there is no API key or an invalid API key
            return JSONResponse({"detail": "Not authenticated"}, status_code=403)
