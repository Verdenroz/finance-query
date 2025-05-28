from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from curl_cffi import requests
from fastapi import HTTPException

from src.clients.fetch_client import DEFAULT_TIMEOUT, CurlFetchClient


@pytest.fixture
def mock_session():
    """Create a mock requests.Session."""
    mock = MagicMock(spec=requests.Session)
    mock.headers = {}
    mock.proxies = {}
    return mock


@pytest.fixture
def fetch_client(mock_session):
    """Create a CurlFetchClient with mocked session."""
    return CurlFetchClient(session=mock_session)


class TestCurlFetchClient:
    def test_init_default(self):
        """Test initialization with default parameters."""
        with patch("src.clients.fetch_client.requests.Session") as mock_session_cls:
            mock_session = MagicMock()
            mock_session.headers = {}
            mock_session_cls.return_value = mock_session

            client = CurlFetchClient()

            mock_session_cls.assert_called_once_with(impersonate="chrome")
            assert client.timeout == DEFAULT_TIMEOUT
            assert "User-Agent" in mock_session.headers
            assert "Accept" in mock_session.headers
            assert "Accept-Language" in mock_session.headers
            assert "Accept-Encoding" in mock_session.headers
            assert "sec-ch-ua" in mock_session.headers
            assert client.session.proxies == mock_session.proxies

    def test_init_with_params(self, mock_session):
        """Test initialization with custom parameters."""
        custom_headers = {"Custom-Header": "test-value"}
        client = CurlFetchClient(session=mock_session, timeout=20, proxy="http://proxy.example.com", default_headers=custom_headers)

        assert client.timeout == 20
        assert mock_session.proxies == {"http": "http://proxy.example.com", "https": "http://proxy.example.com"}
        assert mock_session.headers == custom_headers

    def test_request_success(self, fetch_client, mock_session):
        """Test successful request."""
        mock_response = MagicMock()
        mock_session.request.return_value = mock_response

        response = fetch_client.request(
            url="https://example.com", method="GET", params={"key": "value"}, data={"data_key": "data_value"}, headers={"Header": "Value"}
        )

        mock_session.request.assert_called_once_with(
            method="GET",
            url="https://example.com",
            params={"key": "value"},
            json={"data_key": "data_value"},
            headers={"Header": "Value"},
            timeout=DEFAULT_TIMEOUT,
        )
        assert response == mock_response

    def test_request_with_error(self, fetch_client, mock_session):
        """Test request with error."""
        mock_session.request.side_effect = requests.RequestsError("Network error")

        with pytest.raises(HTTPException) as exc_info:
            fetch_client.request(url="https://example.com")

        assert exc_info.value.status_code == 500
        assert "HTTP request failed" in str(exc_info.value.detail)
        assert "Network error" in str(exc_info.value.detail)

    async def test_fetch_success(self, fetch_client, mock_session):
        """Test successful fetch."""
        mock_response = MagicMock()
        mock_response.text = "Response Text"
        mock_session.request.return_value = mock_response

        with patch("asyncio.to_thread", new_callable=AsyncMock) as mock_to_thread:
            mock_to_thread.return_value = mock_response

            result = await fetch_client.fetch(
                url="https://example.com", method="POST", params={"key": "value"}, data={"data_key": "data_value"}, headers={"Header": "Value"}
            )

            mock_to_thread.assert_called_once_with(
                fetch_client.request,
                "https://example.com",
                method="POST",
                params={"key": "value"},
                data={"data_key": "data_value"},
                headers={"Header": "Value"},
            )
            assert result == "Response Text"

    async def test_fetch_return_response(self, fetch_client, mock_session):
        """Test fetch with return_response=True."""
        mock_response = MagicMock()
        mock_response.text = "Response Text"
        mock_session.request.return_value = mock_response

        with patch("asyncio.to_thread", new_callable=AsyncMock) as mock_to_thread:
            mock_to_thread.return_value = mock_response

            result = await fetch_client.fetch(url="https://example.com", return_response=True)

            assert result == mock_response

    async def test_fetch_error_propagation(self, fetch_client):
        """Test that fetch propagates errors from request."""
        with patch.object(fetch_client, "request") as mock_request:
            mock_request.side_effect = HTTPException(500, "Test error")

            with patch("asyncio.to_thread", new_callable=AsyncMock) as mock_to_thread:
                mock_to_thread.side_effect = HTTPException(500, "Test error")

                with pytest.raises(HTTPException) as exc_info:
                    await fetch_client.fetch(url="https://example.com")

                assert exc_info.value.status_code == 500
                assert "Test error" in str(exc_info.value.detail)
