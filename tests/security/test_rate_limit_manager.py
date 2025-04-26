import time
from unittest.mock import AsyncMock

from starlette.websockets import WebSocket

from src.security.rate_limit_manager import RateLimitManager, SecurityConfig, RateLimitEntry


# Dummy websocket class for testing validate_websocket
class DummyWebSocket(AsyncMock):
    def __init__(self, headers: dict, client_ip: str):
        super().__init__(spec=WebSocket)
        self.headers = headers
        self.client = type("DummyClient", (), {"host": client_ip})


class TestRateLimitManager:
    async def test_get_rate_limit_info_no_entry(self):
        manager = RateLimitManager()
        ip = "127.0.0.1"
        info = await manager.get_rate_limit_info(ip)
        assert info["count"] == 0
        assert info["remaining"] == SecurityConfig.RATE_LIMIT
        assert info["reset_in"] == 86400
        assert info["limit"] == SecurityConfig.RATE_LIMIT

    async def test_increment_and_check_increments(self, monkeypatch):
        monkeypatch.setattr(SecurityConfig, "ADMIN_API_KEY", "test")
        monkeypatch.setattr(SecurityConfig, "RATE_LIMIT", 3)
        manager = RateLimitManager()
        ip = "127.0.0.1"

        allowed, info = await manager.increment_and_check(ip, api_key=None)
        assert allowed is True
        assert info["count"] == 1

        allowed, info = await manager.increment_and_check(ip, api_key=None)
        assert allowed is True
        assert info["count"] == 2

        allowed, info = await manager.increment_and_check(ip, api_key=None)
        assert allowed is True
        assert info["count"] == 3

        allowed, info = await manager.increment_and_check(ip, api_key=None)
        assert allowed is False
        assert info["count"] == 3

    async def test_increment_and_check_admin(self, monkeypatch):
        admin_key = "admin123"
        monkeypatch.setenv("ADMIN_API_KEY", admin_key)
        # Update in‑memory config to reflect the new key.
        SecurityConfig.ADMIN_API_KEY = admin_key

        manager = RateLimitManager()
        ip = "127.0.0.1"
        allowed, info = await manager.increment_and_check(ip, api_key=admin_key)
        assert allowed is True
        # Expect admin to bypass rate limiting and return empty info.
        assert info == {}

    async def test_get_health_check_info_no_entry(self):
        manager = RateLimitManager()
        ip = "127.0.0.1"
        info = await manager.get_health_check_info(ip)
        assert info["can_access"] is True
        assert info["reset_in"] == SecurityConfig.HEALTH_CHECK_INTERVAL

    async def test_check_health_rate_limit(self, monkeypatch):
        monkeypatch.setattr(SecurityConfig, "HEALTH_CHECK_INTERVAL", 1800)
        manager = RateLimitManager()
        ip = "127.0.0.1"
        api_key = "non_admin"

        allowed, info = await manager.check_health_rate_limit(ip, api_key)
        assert allowed is True
        assert info["reset_in"] == 1800

        allowed, info = await manager.check_health_rate_limit(ip, api_key)
        assert allowed is False
        # Use <= here because the time difference may be negligible.
        assert info["reset_in"] <= 1800

    async def test_check_health_rate_limit_admin(self, monkeypatch):
        admin_key = "admin123"
        monkeypatch.setenv("ADMIN_API_KEY", admin_key)
        SecurityConfig.ADMIN_API_KEY = admin_key
        manager = RateLimitManager()
        ip = "127.0.0.1"

        allowed, info = await manager.check_health_rate_limit(ip, api_key=admin_key)
        assert allowed is True
        assert info["reset_in"] == SecurityConfig.HEALTH_CHECK_INTERVAL

    async def test_validate_websocket_skip_security(self, monkeypatch):
        monkeypatch.setenv("USE_SECURITY", "False")
        manager = RateLimitManager()
        dummy_ws = DummyWebSocket(headers={}, client_ip="127.0.0.1")
        valid, metadata = await manager.validate_websocket(dummy_ws)
        assert valid is True
        assert metadata == {}

    async def test_validate_websocket_rate_limit_exceeded(self, monkeypatch):
        monkeypatch.setenv("USE_SECURITY", "True")
        monkeypatch.setattr(SecurityConfig, "RATE_LIMIT", 1)
        manager = RateLimitManager()

        dummy_ws = DummyWebSocket(headers={"x-api-key": "invalid"}, client_ip="127.0.0.1")
        valid, metadata = await manager.validate_websocket(dummy_ws)
        assert valid is True

        valid, metadata = await manager.validate_websocket(dummy_ws)
        assert valid is False
        dummy_ws.close.assert_called_once_with(code=1008, reason="Rate limit exceeded")

    async def test_validate_websocket_admin_bypass(self, monkeypatch):
        admin_key = "admin123"
        monkeypatch.setenv("ADMIN_API_KEY", admin_key)
        monkeypatch.setenv("USE_SECURITY", "True")
        SecurityConfig.ADMIN_API_KEY = admin_key
        manager = RateLimitManager()
        dummy_ws = DummyWebSocket(headers={"x-api-key": admin_key}, client_ip="127.0.0.1")
        valid, metadata = await manager.validate_websocket(dummy_ws)
        assert valid is True
        # For admin bypass, metadata should be empty.
        assert metadata == {}

    async def test_cleanup(self):
        manager = RateLimitManager()
        manager.rate_limits["rate_limit:127.0.0.1"] = RateLimitEntry(count=5, expire_at=time.time() + 1000)
        manager.health_checks["health_check:127.0.0.1"] = time.time() + 1000

        await manager.cleanup()
        assert manager.rate_limits == {}
        assert manager.health_checks == {}

    async def test_get_rate_limit_info_after_expiry(self):
        manager = RateLimitManager()
        ip = "127.0.0.1"
        manager.rate_limits[f"rate_limit:{ip}"] = RateLimitEntry(count=10, expire_at=time.time() - 10)
        info = await manager.get_rate_limit_info(ip)
        assert info["count"] == 0
        assert info["remaining"] == SecurityConfig.RATE_LIMIT
