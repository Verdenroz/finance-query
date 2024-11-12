import os
from typing import Set

from fastapi import status
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.websockets import WebSocket


class SecurityConfig:
    DEMO_API_KEY: str = "FinanceQueryDemoAWSHT"
    ADMIN_API_KEY: str = os.getenv("ADMIN_API_KEY")
    RATE_LIMIT: int = 2000  # requests per day

    # Define paths that skip security checks
    OPEN_PATHS: Set[str] = {"/ping", "/docs", "/openapi.json", "/redoc"}

    @classmethod
    def is_open_path(cls, path: str) -> bool:
        return path in cls.OPEN_PATHS


class RateLimitManager:
    def __init__(self, redis_client):
        self.r = redis_client

    async def get_rate_limit_info(self, api_key: str) -> dict:
        key = f"rate_limit:{api_key}"
        count = await self.r.get(key)
        count = int(count) if count else 0
        ttl = await self.r.ttl(key)

        return {
            "count": count,
            "remaining": SecurityConfig.RATE_LIMIT - count,
            "reset_in": ttl if ttl > 0 else 86400,
            "limit": SecurityConfig.RATE_LIMIT
        }

    async def increment_and_check(self, api_key: str) -> tuple[bool, dict]:
        """Returns (is_allowed, rate_limit_info)"""
        if api_key != SecurityConfig.DEMO_API_KEY:
            return True, {}

        key = f"rate_limit:{api_key}"
        count = await self.r.get(key)

        if count is None:
            await self.r.set(key, 1, ex=86400)
        else:
            count = int(count)
            if count >= SecurityConfig.RATE_LIMIT:
                return False, await self.get_rate_limit_info(api_key)
            await self.r.incr(key)

        return True, await self.get_rate_limit_info(api_key)

    async def validate_websocket(self, websocket: WebSocket) -> tuple[bool, dict]:
        """
        Validate the websocket connection and enforce rate limiting
        Returns: (is_valid, metadata)
        """
        # Skip rate limiting if security is disabled
        if not os.getenv('USE_SECURITY', 'False') == 'True':
            return True, {}

        api_key = websocket.headers.get("x-api-key")
        # Validate API key
        if api_key not in {SecurityConfig.DEMO_API_KEY, SecurityConfig.ADMIN_API_KEY}:
            await websocket.close(code=1008, reason="Invalid API key")
            return False, {}

        # Handle rate limiting for demo key
        if api_key == SecurityConfig.DEMO_API_KEY:
            is_allowed, rate_info = await self.increment_and_check(api_key)
            if not is_allowed:
                await websocket.close(code=1008, reason="Rate limit exceeded")
                return False, {}

            return True, {
                "metadata": {
                    "rate_limit": rate_info["limit"],
                    "remaining_requests": rate_info["remaining"],
                    "reset": rate_info["reset_in"]
                }
            }

        return True, {}


class RateLimitMiddleware(BaseHTTPMiddleware):
    def __init__(self, app, rate_limit_manager: RateLimitManager):
        super().__init__(app)
        self.rate_limit_manager = rate_limit_manager

    async def dispatch(self, request: Request, call_next):
        # Skip security for open paths
        if SecurityConfig.is_open_path(request.url.path):
            return await call_next(request)

        api_key = request.headers.get("x-api-key")

        # Check API key validity
        if api_key not in {SecurityConfig.DEMO_API_KEY, SecurityConfig.ADMIN_API_KEY}:
            return JSONResponse(
                {"detail": "Invalid API key"},
                status_code=status.HTTP_401_UNAUTHORIZED
            )

        # Check rate limit for demo key
        if api_key == SecurityConfig.DEMO_API_KEY:
            is_allowed, rate_info = await self.rate_limit_manager.increment_and_check(api_key)

            if not is_allowed:
                return JSONResponse(
                    {
                        "detail": "Rate limit exceeded",
                        "rate_limit_info": rate_info
                    },
                    status_code=status.HTTP_429_TOO_MANY_REQUESTS
                )

            # Continue with the request and add rate limit headers
            response = await call_next(request)
            response.headers.update({
                "X-RateLimit-Limit": str(rate_info["limit"]),
                "X-RateLimit-Remaining": str(rate_info["remaining"]),
                "X-RateLimit-Reset": str(rate_info["reset_in"])
            })
            return response

        # Admin key - no rate limiting
        return await call_next(request)
