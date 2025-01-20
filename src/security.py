import os
import time
from dataclasses import dataclass
from typing import Set, Dict

from fastapi import status
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.websockets import WebSocket


@dataclass
class RateLimitEntry:
    count: int
    expire_at: float


class SecurityConfig:
    ADMIN_API_KEY: str = os.getenv("ADMIN_API_KEY")
    RATE_LIMIT: int = 2000  # requests per day
    HEALTH_CHECK_INTERVAL: int = 1800  # 30 minutes in seconds

    # Define paths that skip security checks
    OPEN_PATHS: Set[str] = {"/ping", "/docs", "/openapi.json", "/redoc"}

    @classmethod
    def is_open_path(cls, path: str) -> bool:
        return path in cls.OPEN_PATHS

    @classmethod
    def is_admin_key(cls, api_key: str | None) -> bool:
        return api_key == cls.ADMIN_API_KEY


class RateLimitManager:
    def __init__(self):
        self.rate_limits: Dict[str, RateLimitEntry] = {}
        self.health_checks: Dict[str, float] = {}

    def _clean_expired(self) -> None:
        """Remove expired entries from rate limit and health check dictionaries"""
        current_time = time.time()
        self.rate_limits = {
            k: v for k, v in self.rate_limits.items()
            if v.expire_at > current_time
        }
        self.health_checks = {
            k: v for k, v in self.health_checks.items()
            if v > current_time
        }

    def _get_rate_limit_key(self, api_key: str, ip: str) -> str:
        """Generate a rate limit key based on API key or IP"""
        return f"rate_limit:{api_key or ip}"

    async def get_rate_limit_info(self, api_key: str, ip: str) -> dict:
        self._clean_expired()
        current_time = time.time()
        key = self._get_rate_limit_key(api_key, ip)
        entry = self.rate_limits.get(key)

        if not entry:
            return {
                "count": 0,
                "remaining": SecurityConfig.RATE_LIMIT,
                "reset_in": 86400,
                "limit": SecurityConfig.RATE_LIMIT
            }

        return {
            "count": entry.count,
            "remaining": SecurityConfig.RATE_LIMIT - entry.count,
            "reset_in": int(entry.expire_at - current_time),
            "limit": SecurityConfig.RATE_LIMIT
        }

    async def get_health_check_info(self, ip: str) -> dict:
        self._clean_expired()
        current_time = time.time()
        key = f"health_check:{ip}"
        expire_at = self.health_checks.get(key)

        if expire_at is None:
            return {
                "can_access": True,
                "reset_in": SecurityConfig.HEALTH_CHECK_INTERVAL
            }

        return {
            "can_access": False,
            "reset_in": int(expire_at - current_time)
        }

    async def check_health_rate_limit(self, ip: str, api_key: str | None) -> tuple[bool, dict]:
        """Returns (is_allowed, rate_limit_info) for health check endpoint"""
        # Always allow admin key access first
        if SecurityConfig.is_admin_key(api_key):
            return True, {"reset_in": SecurityConfig.HEALTH_CHECK_INTERVAL}

        self._clean_expired()
        current_time = time.time()
        key = f"health_check:{ip}"
        expire_at = self.health_checks.get(key)

        if expire_at is None:
            self.health_checks[key] = current_time + SecurityConfig.HEALTH_CHECK_INTERVAL
            return True, {"reset_in": SecurityConfig.HEALTH_CHECK_INTERVAL}

        return False, {"reset_in": int(expire_at - current_time)}

    async def increment_and_check(self, api_key: str, ip: str) -> tuple[bool, dict]:
        """Returns (is_allowed, rate_limit_info)"""
        # Always check admin key first
        if SecurityConfig.is_admin_key(api_key):
            return True, {}

        self._clean_expired()
        current_time = time.time()
        key = self._get_rate_limit_key(api_key, ip)
        entry = self.rate_limits.get(key)

        if entry is None:
            # New entry
            self.rate_limits[key] = RateLimitEntry(
                count=1,
                expire_at=current_time + 86400
            )
        else:
            if entry.count >= SecurityConfig.RATE_LIMIT:
                return False, await self.get_rate_limit_info(api_key, ip)
            entry.count += 1

        return True, await self.get_rate_limit_info(api_key, ip)

    async def validate_websocket(self, websocket: WebSocket) -> tuple[bool, dict]:
        """
        Validate the websocket connection and enforce rate limiting
        Returns: (is_valid, metadata)
        """
        # Skip rate limiting if security is disabled
        if not os.getenv('USE_SECURITY', 'False') == 'True':
            return True, {}

        api_key = websocket.headers.get("x-api-key")
        client_ip = websocket.client.host

        # Always check admin key first
        if SecurityConfig.is_admin_key(api_key):
            return True, {}

        # Handle rate limiting for all other connections
        is_allowed, rate_info = await self.increment_and_check(api_key, client_ip)
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


class RateLimitMiddleware(BaseHTTPMiddleware):
    def __init__(self, app, rate_limit_manager: RateLimitManager):
        super().__init__(app)
        self.rate_limit_manager = rate_limit_manager

    async def dispatch(self, request: Request, call_next):
        api_key = request.headers.get("x-api-key")
        client_ip = request.client.host

        # Skip security for open paths
        if SecurityConfig.is_open_path(request.url.path):
            return await call_next(request)

        # Special handling for health check endpoint
        if request.url.path == "/health":
            is_allowed, rate_info = await self.rate_limit_manager.check_health_rate_limit(client_ip, api_key)

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

        # Check rate limit for all other requests (admin key check is handled inside increment_and_check)
        is_allowed, rate_info = await self.rate_limit_manager.increment_and_check(api_key, client_ip)

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