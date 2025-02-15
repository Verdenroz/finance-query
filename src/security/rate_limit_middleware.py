from fastapi import status
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse

from src.security.dependencies import check_health_rate_limit, increment_and_check
from .rate_limit_manager import SecurityConfig


class RateLimitMiddleware(BaseHTTPMiddleware):
    def __init__(self, app):
        super().__init__(app)

    async def dispatch(self, request: Request, call_next):
        api_key = request.headers.get("x-api-key")
        client_ip = request.client.host

        # Skip security for open paths
        if SecurityConfig.is_open_path(request.url.path):
            return await call_next(request)

        # Special handling for health check endpoint
        if request.url.path == "/health":
            is_allowed, rate_info = await check_health_rate_limit(ip=client_ip, api_key=api_key)

            if not is_allowed:
                return JSONResponse(
                    {
                        "detail": "Health check rate limit exceeded. Try again later.",
                        "rate_limit_info": rate_info
                    },
                    status_code=status.HTTP_429_TOO_MANY_REQUESTS
                )

            response = await call_next(request)
            response.headers["X-RateLimit-Reset"] = str(rate_info["reset_in"])
            return response

        # Check rate limit for all other requests
        is_allowed, rate_info = await increment_and_check(ip=client_ip, api_key=api_key)

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
        if rate_info:  # Only add headers if rate info exists (not admin)
            response.headers.update({
                "X-RateLimit-Limit": str(rate_info["limit"]),
                "X-RateLimit-Remaining": str(rate_info["remaining"]),
                "X-RateLimit-Reset": str(rate_info["reset_in"])
            })
        return response
