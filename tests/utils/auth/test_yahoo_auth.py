import asyncio
from datetime import UTC, datetime, timedelta
from unittest.mock import MagicMock, patch

import pytest

from src.utils.yahoo_auth import YahooAuthError, YahooAuthManager


@pytest.fixture
def yahoo_auth_manager():
    """Create a YahooAuthManager instance for testing."""
    return YahooAuthManager()


@pytest.mark.asyncio
class TestYahooAuthManager:
    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_refresh_success(self, mock_session_class, yahoo_auth_manager):
        """Test successful refresh."""
        # Setup mock session
        mock_session = MagicMock()
        mock_session_class.return_value = mock_session

        # Mock response for getcrumb
        mock_crumb_response = MagicMock()
        mock_crumb_response.text = "abc123"

        # Set up the session get calls
        mock_session.get.side_effect = [
            MagicMock(),  # First call to fc.yahoo.com
            mock_crumb_response,  # Second call to get crumb
        ]

        # Set up session cookies
        mock_session.cookies = {"cookie1": "value1", "cookie2": "value2"}

        # Call refresh
        await yahoo_auth_manager.refresh()

        # Verify the calls
        assert mock_session.get.call_count == 2
        mock_session.get.assert_any_call("https://fc.yahoo.com", timeout=10, allow_redirects=True, proxies={'http': None, 'https': None})
        mock_session.get.assert_any_call("https://query1.finance.yahoo.com/v1/test/getcrumb", timeout=10, allow_redirects=True, proxies={'http': None, 'https': None})

        # Verify the values were set
        assert yahoo_auth_manager._crumb == "abc123"
        assert isinstance(yahoo_auth_manager._cookie, dict)
        assert yahoo_auth_manager._last_update is not None

    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_refresh_with_proxy(self, mock_session_class, yahoo_auth_manager):
        """Test refresh with proxy."""
        # Setup mock session
        mock_session = MagicMock()
        mock_session_class.return_value = mock_session

        # Mock response for getcrumb
        mock_crumb_response = MagicMock()
        mock_crumb_response.text = "abc123"

        # Set up the session get calls
        mock_session.get.side_effect = [
            MagicMock(),  # First call to fc.yahoo.com
            mock_crumb_response,  # Second call to get crumb
        ]

        # Call refresh with proxy
        await yahoo_auth_manager.refresh(proxy="http://proxy.example.com:8080")

        # Verify proxy was set
        assert mock_session.proxies == {"http": "http://proxy.example.com:8080", "https": "http://proxy.example.com:8080"}

    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_refresh_fallback_to_csrf(self, mock_session_class, yahoo_auth_manager):
        """Test refresh falls back to CSRF method when first crumb is invalid."""
        # Setup mock session
        mock_session = MagicMock()
        mock_session_class.return_value = mock_session

        # Mock responses
        mock_initial_crumb = MagicMock()
        mock_initial_crumb.text = "<html>invalid</html>"  # Invalid crumb requiring fallback

        mock_consent_response = MagicMock()
        mock_consent_response.text = """
            <html>
                <input type="hidden" name="csrfToken" value="test-csrf-token">
                <input type="hidden" name="sessionId" value="test-session-id">
            </html>
        """

        mock_final_crumb = MagicMock()
        mock_final_crumb.text = "valid-crumb-123"

        # Set up the session get and post calls
        mock_session.get.side_effect = [
            MagicMock(),  # First call to fc.yahoo.com
            mock_initial_crumb,  # First invalid crumb
            mock_consent_response,  # Consent page with CSRF and session ID
            MagicMock(),  # Copy consent call
            mock_final_crumb,  # Final valid crumb
        ]

        # Call refresh
        await yahoo_auth_manager.refresh()

        # Verify post request was made with correct data
        mock_session.post.assert_called_once()
        assert "test-session-id" in mock_session.post.call_args[0][0]

        # Verify the final crumb was set
        assert yahoo_auth_manager._crumb == "valid-crumb-123"

    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_refresh_no_csrf_token(self, mock_session_class, yahoo_auth_manager):
        """Test refresh fails when no CSRF token can be extracted."""
        # Setup mock session
        mock_session = MagicMock()
        mock_session_class.return_value = mock_session

        # Mock responses
        mock_initial_crumb = MagicMock()
        mock_initial_crumb.text = "<html>invalid</html>"  # Invalid crumb

        mock_consent_response = MagicMock()
        mock_consent_response.text = "<html>No CSRF token here</html>"  # No CSRF token

        # Set up the session get calls
        mock_session.get.side_effect = [
            MagicMock(),  # First call to fc.yahoo.com
            mock_initial_crumb,  # First invalid crumb
            mock_consent_response,  # Consent page without CSRF
        ]

        # Check that YahooAuthError is raised
        with pytest.raises(YahooAuthError, match="Failed to extract CSRF token and session id"):
            await yahoo_auth_manager.refresh()

    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_refresh_invalid_final_crumb(self, mock_session_class, yahoo_auth_manager):
        """Test refresh fails when final crumb is also invalid."""
        # Setup mock session
        mock_session = MagicMock()
        mock_session_class.return_value = mock_session

        # Mock responses
        mock_initial_crumb = MagicMock()
        mock_initial_crumb.text = "<html>invalid</html>"  # Invalid initial crumb

        mock_consent_response = MagicMock()
        mock_consent_response.text = """
            <html>
                <input type="hidden" name="csrfToken" value="test-csrf-token">
                <input type="hidden" name="sessionId" value="test-session-id">
            </html>
        """

        mock_final_crumb = MagicMock()
        mock_final_crumb.text = "<html>still invalid</html>"  # Invalid final crumb

        # Set up the session get and post calls
        mock_session.get.side_effect = [
            MagicMock(),  # First call to fc.yahoo.com
            mock_initial_crumb,  # First invalid crumb
            mock_consent_response,  # Consent page with CSRF
            MagicMock(),  # Copy consent call
            mock_final_crumb,  # Final invalid crumb
        ]

        # Check that YahooAuthError is raised
        with pytest.raises(YahooAuthError, match="Yahoo returned an invalid crumb"):
            await yahoo_auth_manager.refresh()

    @patch("src.utils.yahoo_auth.requests.Session")
    async def test_extract_csrf(self, mock_session_class, yahoo_auth_manager):
        """Test _extract_csrf method."""
        # HTML with both CSRF token and session ID
        html_with_both = """
            <html>
                <input type="hidden" name="csrfToken" value="csrf-value">
                <input type="hidden" name="sessionId" value="session-value">
            </html>
        """
        csrf, session = yahoo_auth_manager._extract_csrf(html_with_both)
        assert csrf == "csrf-value"
        assert session == "session-value"

        # HTML with only CSRF token
        html_with_csrf = """
            <html>
                <input type="hidden" name="csrfToken" value="csrf-only">
            </html>
        """
        csrf, session = yahoo_auth_manager._extract_csrf(html_with_csrf)
        assert csrf == "csrf-only"
        assert session is None

        # HTML with only session ID
        html_with_session = """
            <html>
                <input type="hidden" name="sessionId" value="session-only">
            </html>
        """
        csrf, session = yahoo_auth_manager._extract_csrf(html_with_session)
        assert csrf is None
        assert session == "session-only"

        # HTML with neither
        html_empty = "<html>Empty</html>"
        csrf, session = yahoo_auth_manager._extract_csrf(html_empty)
        assert csrf is None
        assert session is None

    @patch("src.utils.yahoo_auth.YahooAuthManager.refresh")
    async def test_get_or_refresh_initial(self, mock_refresh, yahoo_auth_manager):
        """Test get_or_refresh when no data is cached yet."""
        # Setup mock response for refresh
        mock_refresh.return_value = None
        yahoo_auth_manager._cookie = {"test": "cookie"}
        yahoo_auth_manager._crumb = "test-crumb"

        # Call get_or_refresh
        cookie, crumb = await yahoo_auth_manager.get_or_refresh()

        # Verify refresh was called
        mock_refresh.assert_called_once()
        assert cookie == {"test": "cookie"}
        assert crumb == "test-crumb"

    @patch("src.utils.yahoo_auth.YahooAuthManager.refresh")
    async def test_get_or_refresh_with_proxy(self, mock_refresh, yahoo_auth_manager):
        """Test get_or_refresh with proxy."""
        # Setup mock response
        mock_refresh.return_value = None
        yahoo_auth_manager._cookie = {"test": "cookie"}
        yahoo_auth_manager._crumb = "test-crumb"

        # Call get_or_refresh with proxy
        await yahoo_auth_manager.get_or_refresh(proxy="http://proxy.example.com")

        # Verify refresh was called with proxy
        mock_refresh.assert_called_once_with("http://proxy.example.com")

    @patch("src.utils.yahoo_auth.YahooAuthManager.refresh")
    async def test_get_or_refresh_cached_recent(self, mock_refresh, yahoo_auth_manager):
        """Test get_or_refresh when data is cached and recent."""
        # Setup cached data that's recent
        yahoo_auth_manager._cookie = {"cached": "cookie"}
        yahoo_auth_manager._crumb = "cached-crumb"
        yahoo_auth_manager._last_update = datetime.now(UTC)

        # Call get_or_refresh
        cookie, crumb = await yahoo_auth_manager.get_or_refresh()

        # Verify refresh was not called and cached values returned
        mock_refresh.assert_not_called()
        assert cookie == {"cached": "cookie"}
        assert crumb == "cached-crumb"

    @patch("src.utils.yahoo_auth.YahooAuthManager.refresh")
    async def test_get_or_refresh_cached_expired(self, mock_refresh, yahoo_auth_manager):
        """Test get_or_refresh when data is cached but expired."""
        # Setup cached data that's old
        yahoo_auth_manager._cookie = {"old": "cookie"}
        yahoo_auth_manager._crumb = "old-crumb"
        yahoo_auth_manager._last_update = datetime.now(UTC) - timedelta(seconds=35)  # Older than 30s

        # Setup new values after refresh
        def side_effect(proxy=None):
            yahoo_auth_manager._cookie = {"new": "cookie"}
            yahoo_auth_manager._crumb = "new-crumb"
            yahoo_auth_manager._last_update = datetime.now(UTC)

        mock_refresh.side_effect = side_effect

        # Call get_or_refresh
        cookie, crumb = await yahoo_auth_manager.get_or_refresh()

        # Verify refresh was called and new values returned
        mock_refresh.assert_called_once()
        assert cookie == {"new": "cookie"}
        assert crumb == "new-crumb"

    async def test_get_or_refresh_concurrency(self, yahoo_auth_manager):
        """Test concurrency handling in get_or_refresh."""
        # Replace refresh with a slow version that we can track
        original_refresh = yahoo_auth_manager.refresh
        refresh_started = asyncio.Event()
        refresh_continue = asyncio.Event()
        refresh_complete = asyncio.Event()
        refresh_count = 0

        async def slow_refresh(proxy=None):
            nonlocal refresh_count
            refresh_count += 1
            refresh_started.set()
            await refresh_continue.wait()
            await original_refresh(proxy)
            refresh_complete.set()

        yahoo_auth_manager.refresh = slow_refresh

        # Start two concurrent get_or_refresh calls
        task1 = asyncio.create_task(yahoo_auth_manager.get_or_refresh())

        # Wait for first refresh to start
        await refresh_started.wait()

        # Start second get_or_refresh while first is still running
        task2 = asyncio.create_task(yahoo_auth_manager.get_or_refresh())

        # Let the first refresh complete
        refresh_continue.set()

        # Get results from both tasks
        result1 = await task1
        result2 = await task2

        # Wait for everything to complete
        await refresh_complete.wait()

        # Verify only one refresh happened and both got same results
        assert refresh_count == 1
        assert result1 == result2
