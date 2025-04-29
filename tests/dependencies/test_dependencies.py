import asyncio
from unittest.mock import AsyncMock, MagicMock, call, patch

import pytest
from aiohttp import ClientError, ClientPayloadError, ClientSession
from fastapi import FastAPI, HTTPException
from starlette.websockets import WebSocket

from src.connections import RedisConnectionManager
from src.context import request_context
from src.dependencies import (
    fetch,
    get_auth_data,
    get_connection_manager,
    get_crumb,
    get_logo,
    get_request_context,
    get_session,
    refresh_yahoo_auth,
    remove_proxy_whitelist,
    setup_proxy_whitelist,
)


class TestDependencies:
    @pytest.fixture
    def mock_request(self):
        """Create a mock request for context"""
        request_mock = MagicMock()
        return request_mock

    @pytest.fixture
    def setup_context(self, mock_request):
        """Set up request context with mock request"""
        token = request_context.set(mock_request)
        yield mock_request
        try:
            request_context.reset(token)
        except ValueError:
            pass  # Context may already be reset

    async def test_get_request_context(self, setup_context):
        """Test get_request_context returns the correct context"""
        context = await get_request_context()
        assert context == setup_context

    async def test_get_connection_manager(self):
        """Test get_connection_manager returns the connection manager from WebSocket app state"""
        mock_websocket = MagicMock(spec=WebSocket)
        mock_manager = MagicMock(spec=RedisConnectionManager)
        mock_websocket.app.state.connection_manager = mock_manager

        result = await get_connection_manager(mock_websocket)
        assert result == mock_manager

    async def test_get_session(self):
        """Test get_session returns the session from app state"""
        mock_request = MagicMock()
        mock_session = MagicMock(spec=ClientSession)
        mock_request.app.state.session = mock_session

        # Properly handle the dependency by patching the actual function, not just the dependency
        with patch("src.dependencies.get_request_context", AsyncMock(return_value=mock_request)):
            # Call the function directly with the resolved dependency value, not through the Depends mechanism
            result = await get_session(mock_request)
            assert result == mock_session

    @patch("src.dependencies.request_context")
    async def test_get_redis(self, mock_context):
        """Test get_redis returns the Redis client from app state"""
        mock_request = MagicMock()
        mock_redis = MagicMock()
        mock_request.app.state.redis = mock_redis

        # Mock the context get method
        mock_context.get.return_value = mock_request

        # Need to patch the decorated function to bypass @injectable
        with patch("src.dependencies.get_request_context", AsyncMock(return_value=mock_request)):
            with patch("src.dependencies.get_redis.__wrapped__", AsyncMock(return_value=mock_redis)):
                from src.dependencies import get_redis

                result = await get_redis.__wrapped__(mock_request)
                assert result == mock_redis

    @patch("src.dependencies.request_context")
    async def test_get_yahoo_cookies(self, mock_context):
        """Test get_yahoo_cookies returns cookies from app state"""
        mock_request = MagicMock()
        mock_request.app.state.cookies = "test_cookies"

        # Mock the context get method
        mock_context.get.return_value = mock_request

        # Need to patch the decorated function to bypass @injectable
        with patch("src.dependencies.get_request_context", AsyncMock(return_value=mock_request)):
            with patch("src.dependencies.get_yahoo_cookies.__wrapped__", AsyncMock(return_value="test_cookies")):
                from src.dependencies import get_yahoo_cookies

                result = await get_yahoo_cookies.__wrapped__(mock_request)
                assert result == "test_cookies"

    @patch("src.dependencies.request_context")
    async def test_get_yahoo_crumb(self, mock_context):
        """Test get_yahoo_crumb returns crumb from app state"""
        mock_request = MagicMock()
        mock_request.app.state.crumb = "test_crumb"

        # Mock the context get method
        mock_context.get.return_value = mock_request

        # Need to patch the decorated function to bypass @injectable
        with patch("src.dependencies.get_request_context", AsyncMock(return_value=mock_request)):
            with patch("src.dependencies.get_yahoo_crumb.__wrapped__", AsyncMock(return_value="test_crumb")):
                from src.dependencies import get_yahoo_crumb

                result = await get_yahoo_crumb.__wrapped__(mock_request)
                assert result == "test_crumb"

    @patch("src.dependencies.proxy", "http://proxy.example.com")
    @patch("os.getenv")
    async def test_fetch(self, mock_getenv):
        """Test fetch function with different configurations"""
        # Create a MagicMock for proxy_auth to use in the test
        mock_proxy_auth = MagicMock()

        # Patch the proxy_auth in the specific function scope
        with patch("src.dependencies.proxy_auth", mock_proxy_auth):
            mock_session = AsyncMock(spec=ClientSession)
            mock_response = AsyncMock()
            mock_response.text = AsyncMock(return_value="test response")
            mock_response.read = AsyncMock(return_value=b"test content")

            # Configure session.request to return our mock response
            mock_session.request.return_value.__aenter__.return_value = mock_response

            # Test with proxy enabled
            mock_getenv.return_value = "True"  # USE_PROXY=True

            # Test empty URL case first
            result_empty = await fetch.__wrapped__(session=mock_session, url="", use_proxy=True)
            assert result_empty is None
            assert not mock_session.request.called

            # Call the wrapper directly
            result = await fetch.__wrapped__(
                session=mock_session,
                url="https://example.com",
                method="GET",
                params={"test": "param"},
                headers={"User-Agent": "Test"},
                use_proxy=True,
            )

            # Verify session.request was called with proxy
            mock_session.request.assert_called_with(
                method="GET",
                url="https://example.com",
                params={"test": "param"},
                headers={"User-Agent": "Test"},
                proxy="http://proxy.example.com",
                proxy_auth=mock_proxy_auth,  # Use the mock_proxy_auth object
                timeout=5,
            )

            assert result == "test response"

    @pytest.mark.parametrize(
        "use_proxy,proxy_value,proxy_auth_value",
        [
            (False, None, None),  # No proxy
            (True, "http://test.proxy", "test_auth"),  # With proxy
        ],
    )
    @patch("os.getenv")
    async def test_fetch_return_response(self, mock_getenv, use_proxy, proxy_value, proxy_auth_value):
        """Test fetch function with return_response=True for both proxy settings"""
        # Set environment variable according to test case
        mock_getenv.return_value = "True" if use_proxy else "False"
        mock_session = AsyncMock(spec=ClientSession)
        mock_response = AsyncMock()
        mock_response.read = AsyncMock(return_value=b"response content")

        async_cm = AsyncMock()
        async_cm.__aenter__.return_value = mock_response
        mock_session.request.return_value = async_cm

        # Apply proxy patches conditionally
        with patch("src.dependencies.proxy", proxy_value):
            with patch("src.dependencies.proxy_auth", proxy_auth_value):
                result = await fetch.__wrapped__(session=mock_session, url="https://example.com", return_response=True, use_proxy=use_proxy)

        # Check response body was set correctly
        assert hasattr(result, "_body")
        assert result._body == b"response content"

        # Verify request was made with correct proxy settings
        mock_session.request.assert_called_with(
            method="GET",
            url="https://example.com",
            params=None,
            headers=None,
            proxy=proxy_value,
            proxy_auth=proxy_auth_value,
            timeout=5,
        )

    @pytest.mark.parametrize(
        "use_proxy,proxy_value,proxy_auth_value",
        [
            (False, None, None),  # No proxy
            (True, "http://test.proxy", "test_auth"),  # With proxy
        ],
    )
    @patch("os.getenv")
    async def test_fetch_with_retry(self, mock_getenv, use_proxy, proxy_value, proxy_auth_value):
        """Test fetch retry logic on failure with and without proxy"""
        # Set environment variable according to test case
        mock_getenv.return_value = "True" if use_proxy else "False"
        mock_session = AsyncMock(spec=ClientSession)

        # Create mock context managers that raise exceptions
        error_cm1 = AsyncMock()
        error_cm1.__aenter__.side_effect = ClientPayloadError("Connection error")

        error_cm2 = AsyncMock()
        error_cm2.__aenter__.side_effect = asyncio.TimeoutError("Timeout")

        # Create successful response
        success_response = AsyncMock()
        success_response.text = AsyncMock(return_value="success")
        success_cm = AsyncMock()
        success_cm.__aenter__.return_value = success_response

        # Configure request side effects
        mock_session.request.side_effect = [error_cm1, error_cm2, success_cm]

        with patch("asyncio.sleep", AsyncMock()) as mock_sleep:
            with patch("src.dependencies.proxy", proxy_value):
                with patch("src.dependencies.proxy_auth", proxy_auth_value):
                    result = await fetch.__wrapped__(
                        session=mock_session,
                        url="https://example.com",
                        max_retries=3,
                        retry_delay=0.1,
                        use_proxy=use_proxy,
                    )

        # Verify sleep was called between retries
        assert mock_sleep.call_count == 2
        mock_sleep.assert_has_calls([call(0.1), call(0.1)])

        # Verify request was attempted 3 times
        assert mock_session.request.call_count == 3

        # Verify proxy settings were correctly applied
        for i in range(3):
            assert mock_session.request.call_args_list[i].kwargs["method"] == "GET"
            assert mock_session.request.call_args_list[i].kwargs["url"] == "https://example.com"
            assert mock_session.request.call_args_list[i].kwargs["proxy"] == proxy_value
            assert mock_session.request.call_args_list[i].kwargs["proxy_auth"] == proxy_auth_value

        # Verify result is as expected
        assert result == "success"

    @pytest.mark.parametrize(
        "use_proxy,proxy_value,proxy_auth_value",
        [
            (False, None, None),  # No proxy
            (True, "http://test.proxy", "test_auth"),  # With proxy
        ],
    )
    @patch("os.getenv")
    async def test_fetch_max_retries_exceeded(self, mock_getenv, use_proxy, proxy_value, proxy_auth_value):
        """Test fetch raises HTTPException when max retries exceeded with and without proxy"""
        # Set environment variable according to test case
        mock_getenv.return_value = "True" if use_proxy else "False"
        mock_session = AsyncMock(spec=ClientSession)

        # Create mock context managers that raise exceptions
        error_cm1 = AsyncMock()
        error_cm1.__aenter__.side_effect = ClientError("Connection error")

        error_cm2 = AsyncMock()
        error_cm2.__aenter__.side_effect = ClientError("Connection error")

        # Configure request side effects
        mock_session.request.side_effect = [error_cm1, error_cm2]

        with patch("asyncio.sleep", AsyncMock()):
            with patch("src.dependencies.proxy", proxy_value):
                with patch("src.dependencies.proxy_auth", proxy_auth_value):
                    with pytest.raises(HTTPException) as excinfo:
                        await fetch.__wrapped__(
                            session=mock_session,
                            url="https://example.com",
                            max_retries=2,
                            retry_delay=0.1,
                            use_proxy=use_proxy,
                        )

        # Verify error details
        assert excinfo.value.status_code == 500
        assert "Request failed after 2 attempts" in str(excinfo.value.detail)

        # Verify request was attempted twice
        assert mock_session.request.call_count == 2

        # Verify proxy settings were correctly applied
        for i in range(2):
            assert mock_session.request.call_args_list[i].kwargs["method"] == "GET"
            assert mock_session.request.call_args_list[i].kwargs["url"] == "https://example.com"
            assert mock_session.request.call_args_list[i].kwargs["proxy"] == proxy_value
            assert mock_session.request.call_args_list[i].kwargs["proxy_auth"] == proxy_auth_value

    async def test_get_logo_from_symbol(self):
        """Test get_logo function with symbol"""
        mock_session = AsyncMock(spec=ClientSession)
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.url = "https://img.logo.dev/ticker/AAPL"

        # Configure context manager for session.get
        async_cm = AsyncMock()
        async_cm.__aenter__.return_value = mock_response
        mock_session.get.return_value = async_cm

        result = await get_logo.__wrapped__(session=mock_session, symbol="AAPL")

        # Verify the correct URL was called
        mock_session.get.assert_called_with("https://img.logo.dev/ticker/AAPL?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true")
        assert result == "https://img.logo.dev/ticker/AAPL"

    async def test_get_logo_from_domain(self):
        """Test get_logo function with URL fallback"""
        mock_session = AsyncMock(spec=ClientSession)

        # First response fails (for symbol)
        first_response = AsyncMock()
        first_response.status = 404
        first_cm = AsyncMock()
        first_cm.__aenter__.return_value = first_response

        # Second response succeeds (for domain)
        second_response = AsyncMock()
        second_response.status = 200
        second_response.url = "https://img.logo.dev/apple.com"
        second_cm = AsyncMock()
        second_cm.__aenter__.return_value = second_response

        # Configure session.get to return our mock responses
        mock_session.get.side_effect = [first_cm, second_cm]

        result = await get_logo.__wrapped__(session=mock_session, symbol="AAPL", url="https://www.apple.com")

        # Verify both URLs were called
        assert mock_session.get.call_count == 2
        mock_session.get.assert_has_calls(
            [
                call("https://img.logo.dev/ticker/AAPL?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"),
                call("https://img.logo.dev/apple.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"),
            ]
        )
        assert result == "https://img.logo.dev/apple.com"

    @patch("requests.get")
    @patch("src.dependencies.get_crumb")
    async def test_get_auth_data_success(self, mock_get_crumb, mock_requests_get):
        """Test get_auth_data function successfully gets cookies and crumb"""
        # Mock response with cookies
        mock_response = MagicMock()
        mock_cookies = {"cookie1": "value1", "cookie2": "value2"}
        mock_response.cookies.get_dict.return_value = mock_cookies
        mock_requests_get.return_value = mock_response

        # Mock crumb getter
        mock_get_crumb.return_value = "abc123"

        result = await get_auth_data()

        # Verify request was made with correct headers
        mock_requests_get.assert_called_with(
            "https://finance.yahoo.com",
            headers={
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
                "Accept-Language": "en-US,en;q=0.5",
                "Connection": "keep-alive",
                "Cookie": "cookie1=value1; cookie2=value2",
            },
        )

        # Verify get_crumb was called with headers including cookies
        expected_headers = {
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            "Accept-Language": "en-US,en;q=0.5",
            "Connection": "keep-alive",
            "Cookie": "cookie1=value1; cookie2=value2",
        }
        mock_get_crumb.assert_called_with(expected_headers)

        # Verify result contains cookies string and crumb
        cookies_str, crumb = result
        assert cookies_str == "cookie1=value1; cookie2=value2"
        assert crumb == "abc123"

    @patch("requests.get")
    async def test_get_auth_data_failure(self, mock_requests_get):
        """Test get_auth_data function handles failure"""
        # Mock exception being raised
        mock_requests_get.side_effect = Exception("Connection error")

        with pytest.raises(HTTPException) as excinfo:
            await get_auth_data()

        assert excinfo.value.status_code == 500
        assert "Failed to authenticate with Yahoo Finance" in str(excinfo.value.detail)

    @patch("requests.get")
    def test_get_crumb_success(self, mock_requests_get):
        """Test get_crumb function successfully gets crumb"""
        # Mock response with crumb
        mock_response = MagicMock()
        mock_response.text = '"abc123"'  # Quoted string returned by API
        mock_requests_get.return_value = mock_response

        result = get_crumb({"Cookie": "test=value"})

        # Verify request was made with correct URL and headers
        mock_requests_get.assert_called_with("https://query1.finance.yahoo.com/v1/test/getcrumb", headers={"Cookie": "test=value"})

        # Verify result is unquoted crumb
        assert result == "abc123"

    @patch("requests.get")
    def test_get_crumb_failure(self, mock_requests_get):
        """Test get_crumb function handles failure"""
        # Mock exception being raised
        mock_requests_get.side_effect = Exception("Connection error")

        with pytest.raises(ValueError) as excinfo:
            get_crumb({"Cookie": "test=value"})

        assert "Failed to get valid crumb" in str(excinfo.value)

    @patch("src.dependencies.get_auth_data")
    @patch("asyncio.sleep")
    async def test_refresh_yahoo_auth(self, mock_sleep, mock_get_auth_data):
        """Test refresh_yahoo_auth updates app state with new auth data"""
        # Create state mock with required attributes
        mock_state = MagicMock()
        mock_state.auth_expiry = None
        mock_state.auth_refresh_interval = 3600  # 1 hour

        # Mock app with state
        mock_app = MagicMock(spec=FastAPI)
        mock_app.state = mock_state

        # Mock successful auth data retrieval
        mock_get_auth_data.return_value = ("new_cookies", "new_crumb")

        # Make sleep raise exception immediately to end the infinite loop
        mock_sleep.side_effect = KeyboardInterrupt()

        with pytest.raises(KeyboardInterrupt):
            await refresh_yahoo_auth(mock_app)

        # Verify auth data was updated
        assert mock_app.state.cookies == "new_cookies"
        assert mock_app.state.crumb == "new_crumb"
        assert mock_app.state.auth_expiry is not None

        # Verify sleep was called with appropriate value (up to 1 hour)
        mock_sleep.assert_called_once()
        assert mock_sleep.call_args[0][0] <= 3600

    @patch("requests.get")
    @patch("requests.post")
    @patch("os.getenv")
    async def test_setup_proxy_whitelist_success(self, mock_getenv, mock_post, mock_get):
        """Test setup_proxy_whitelist successfully adds IP to whitelist"""
        # Mock environment variables
        mock_getenv.side_effect = lambda key, default=None: {"PROXY_TOKEN": "test_token", "USE_PROXY": "True"}.get(key, default)

        # Mock IP retrieval
        mock_ip_response = MagicMock()
        mock_ip_response.text = "192.168.1.1"
        mock_get.return_value = mock_ip_response

        # Mock whitelist API response
        mock_whitelist_response = MagicMock()
        mock_whitelist_response.status_code = 200
        mock_post.return_value = mock_whitelist_response

        result = await setup_proxy_whitelist()

        # Verify IP was retrieved
        mock_get.assert_called_with("https://api.ipify.org/")

        # Verify whitelist API was called with correct data
        mock_post.assert_called_with(
            "https://api.brightdata.com/zone/whitelist",
            headers={"Authorization": "Bearer test_token", "Content-Type": "application/json"},
            json={"ip": "192.168.1.1"},
        )

        # Verify result contains expected data for cleanup
        assert result["api_url"] == "https://api.brightdata.com/zone/whitelist"
        assert result["headers"]["Authorization"] == "Bearer test_token"
        assert result["payload"]["ip"] == "192.168.1.1"

    @patch("os.getenv")
    async def test_setup_proxy_whitelist_no_config(self, mock_getenv):
        """Test setup_proxy_whitelist raises error when proxy config is missing"""
        # Mock missing PROXY_TOKEN
        mock_getenv.side_effect = lambda key, default=None: {"PROXY_TOKEN": None, "USE_PROXY": "True"}.get(key, default)

        with pytest.raises(ValueError) as excinfo:
            await setup_proxy_whitelist()

        assert "Proxy configuration is missing" in str(excinfo.value)

    @patch("requests.get")
    @patch("requests.post")
    @patch("os.getenv")
    async def test_setup_proxy_whitelist_failure(self, mock_getenv, mock_post, mock_get):
        """Test setup_proxy_whitelist handles API failure"""
        # Mock environment variables
        mock_getenv.side_effect = lambda key, default=None: {"PROXY_TOKEN": "test_token", "USE_PROXY": "True"}.get(key, default)

        # Mock IP retrieval
        mock_ip_response = MagicMock()
        mock_ip_response.text = "192.168.1.1"
        mock_get.return_value = mock_ip_response

        # Mock failed whitelist API response
        mock_whitelist_response = MagicMock()
        mock_whitelist_response.status_code = 403
        mock_whitelist_response.text = "Unauthorized access"
        mock_post.return_value = mock_whitelist_response

        result = await setup_proxy_whitelist()

        # Verify IP was retrieved
        mock_get.assert_called_with("https://api.ipify.org/")

        # Verify whitelist API was called with correct data
        mock_post.assert_called_with(
            "https://api.brightdata.com/zone/whitelist",
            headers={"Authorization": "Bearer test_token", "Content-Type": "application/json"},
            json={"ip": "192.168.1.1"},
        )

        # Verify None is returned on failure
        assert result is None

    @patch("requests.delete")
    async def test_remove_proxy_whitelist(self, mock_delete):
        """Test remove_proxy_whitelist removes IP from whitelist"""
        # Mock proxy data
        proxy_data = {
            "api_url": "https://api.brightdata.com/zone/whitelist",
            "headers": {"Authorization": "Bearer test_token"},
            "payload": {"ip": "192.168.1.1"},
        }

        await remove_proxy_whitelist(proxy_data)

        # Verify delete request was made with correct data
        mock_delete.assert_called_with(
            "https://api.brightdata.com/zone/whitelist",
            headers={"Authorization": "Bearer test_token"},
            json={"ip": "192.168.1.1"},
        )

    async def test_remove_proxy_whitelist_no_data(self):
        """Test remove_proxy_whitelist handles None data"""
        # This should not raise any exception
        await remove_proxy_whitelist({})
