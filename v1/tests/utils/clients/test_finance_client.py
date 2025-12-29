from unittest.mock import MagicMock, patch

import pytest
from fastapi import HTTPException

from src.clients.yahoo_client import YahooFinanceClient


@pytest.fixture
def yahoo_client():
    """Create a mock YahooFinanceClient for testing."""
    cookies = {"cookie1": "value1", "cookie2": "value2"}
    crumb = "test_crumb"
    return YahooFinanceClient(cookies=cookies, crumb=crumb)


class TestYahooFinanceClient:
    async def test_init(self, yahoo_client):
        """Test the client initialization."""
        # Check that cookies were set
        assert yahoo_client.session.cookies.get("cookie1") == "value1"
        assert yahoo_client.session.cookies.get("cookie2") == "value2"
        assert yahoo_client.crumb == "test_crumb"

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_success(self, mock_fetch, yahoo_client):
        """Test successful Yahoo request."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '{"data": "test"}'
        mock_fetch.return_value = mock_response

        response = await yahoo_client._yahoo_request("https://finance.yahoo.com/test")

        # Verify the request was made with the correct parameters
        mock_fetch.assert_called_once()
        assert mock_fetch.call_args[0][0] == "https://finance.yahoo.com/test"
        assert mock_fetch.call_args[1]["params"]["crumb"] == "test_crumb"
        assert "User-Agent" in mock_fetch.call_args[1]["headers"]

        # Check the response is returned correctly
        assert response == mock_response

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_with_cookies_dict(self, mock_fetch, yahoo_client):
        """Test Yahoo request with cookies as dictionary."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_fetch.return_value = mock_response

        cookies_dict = {"extra": "value"}
        await yahoo_client._yahoo_request("https://finance.yahoo.com/test", cookies=cookies_dict)

        # Check cookies were formatted properly in headers
        assert "Cookie" in mock_fetch.call_args[1]["headers"]
        assert "extra=value" in mock_fetch.call_args[1]["headers"]["Cookie"]
        assert "cookies" not in mock_fetch.call_args[1]

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_401_error(self, mock_fetch, yahoo_client):
        """Test Yahoo request with 401 error."""
        mock_response = MagicMock()
        mock_response.status_code = 401
        mock_fetch.return_value = mock_response

        with pytest.raises(HTTPException) as exc_info:
            await yahoo_client._yahoo_request("https://finance.yahoo.com/test")

        assert exc_info.value.status_code == 401
        assert "Yahoo auth failed" in str(exc_info.value.detail)

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_404_error(self, mock_fetch, yahoo_client):
        """Test Yahoo request with 404 error."""
        mock_response = MagicMock()
        mock_response.status_code = 404
        mock_fetch.return_value = mock_response

        with pytest.raises(HTTPException) as exc_info:
            await yahoo_client._yahoo_request("https://finance.yahoo.com/test")

        assert exc_info.value.status_code == 404
        assert "Yahoo symbol not found" in str(exc_info.value.detail)

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_429_error(self, mock_fetch, yahoo_client):
        """Test Yahoo request with 429 error."""
        mock_response = MagicMock()
        mock_response.status_code = 429
        mock_fetch.return_value = mock_response

        with pytest.raises(HTTPException) as exc_info:
            await yahoo_client._yahoo_request("https://finance.yahoo.com/test")

        assert exc_info.value.status_code == 429
        assert "Yahoo rate-limit exceeded" in str(exc_info.value.detail)

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_yahoo_request_other_error(self, mock_fetch, yahoo_client):
        """Test Yahoo request with other HTTP error."""
        mock_response = MagicMock()
        mock_response.status_code = 500
        mock_response.text = "Server error"
        mock_fetch.return_value = mock_response

        with pytest.raises(HTTPException) as exc_info:
            await yahoo_client._yahoo_request("https://finance.yahoo.com/test")

        assert exc_info.value.status_code == 500
        assert "Server error" in str(exc_info.value.detail)

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    async def test_json_success(self, mock_fetch, yahoo_client):
        """Test successful JSON parsing."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = '{"data": "test"}'
        mock_fetch.return_value = mock_response

        result = await yahoo_client._json("https://finance.yahoo.com/test")

        assert result == {"data": "test"}

    @patch("src.clients.yahoo_client.CurlFetchClient.fetch")
    @patch("src.clients.yahoo_client.orjson.loads")
    async def test_json_parse_error(self, mock_loads, mock_fetch, yahoo_client):
        """Test _json with JSON parsing error."""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = "invalid json"
        mock_fetch.return_value = mock_response
        mock_loads.side_effect = Exception("JSON parse error")

        with pytest.raises(HTTPException) as exc_info:
            await yahoo_client._json("https://finance.yahoo.com/test")

        assert exc_info.value.status_code == 500
        assert "Failed to parse JSON response" in str(exc_info.value.detail)

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_get_quote(self, mock_json, yahoo_client):
        """Test get_quote method."""
        mock_json.return_value = {"quoteSummary": {"result": [{"price": {"regularMarketPrice": 100}}]}}

        result = await yahoo_client.get_quote("AAPL")

        mock_json.assert_called_once()
        assert mock_json.call_args[0][0] == "https://query2.finance.yahoo.com/v10/finance/quoteSummary/AAPL"
        assert "modules" in mock_json.call_args[1]["params"]
        assert result == {"quoteSummary": {"result": [{"price": {"regularMarketPrice": 100}}]}}

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_get_simple_quotes(self, mock_json, yahoo_client):
        """Test get_simple_quotes method."""
        mock_json.return_value = {"quoteResponse": {"result": [{"symbol": "AAPL"}, {"symbol": "MSFT"}]}}

        result = await yahoo_client.get_simple_quotes(["AAPL", "MSFT"])

        mock_json.assert_called_once()
        assert mock_json.call_args[0][0] == "https://query1.finance.yahoo.com/v7/finance/quote"
        assert mock_json.call_args[1]["params"]["symbols"] == "AAPL,MSFT"
        assert result == {"quoteResponse": {"result": [{"symbol": "AAPL"}, {"symbol": "MSFT"}]}}

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_get_chart(self, mock_json, yahoo_client):
        """Test get_chart method."""
        mock_json.return_value = {"chart": {"result": [{"meta": {}, "timestamp": []}]}}

        result = await yahoo_client.get_chart("AAPL", "1d", "1mo")

        mock_json.assert_called_once()
        assert mock_json.call_args[0][0] == "https://query1.finance.yahoo.com/v8/finance/chart/AAPL"
        assert mock_json.call_args[1]["params"]["interval"] == "1d"
        assert mock_json.call_args[1]["params"]["range"] == "1mo"
        assert result == {"chart": {"result": [{"meta": {}, "timestamp": []}]}}

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_search(self, mock_json, yahoo_client):
        """Test search method."""
        mock_json.return_value = {"quotes": [{"symbol": "AAPL"}]}

        result = await yahoo_client.search("Apple")

        mock_json.assert_called_once()
        assert mock_json.call_args[0][0] == "https://query1.finance.yahoo.com/v1/finance/search"
        assert mock_json.call_args[1]["params"]["q"] == "Apple"
        assert mock_json.call_args[1]["params"]["quotesCount"] == 6
        assert result == {"quotes": [{"symbol": "AAPL"}]}

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_search_with_custom_hits(self, mock_json, yahoo_client):
        """Test search method with custom hits."""
        mock_json.return_value = {"quotes": [{"symbol": "AAPL"}]}

        await yahoo_client.search("Apple", hits=10)

        assert mock_json.call_args[1]["params"]["quotesCount"] == 10

    @patch("src.clients.yahoo_client.YahooFinanceClient._json")
    async def test_get_similar_quotes(self, mock_json, yahoo_client):
        """Test get_similar_quotes method."""
        mock_json.return_value = {"finance": {"result": [{"symbol": "MSFT"}]}}

        result = await yahoo_client.get_similar_quotes("AAPL", limit=5)

        mock_json.assert_called_once()
        assert mock_json.call_args[0][0] == "https://query2.finance.yahoo.com/v6/finance/recommendationsbysymbol/AAPL"
        assert mock_json.call_args[1]["params"]["count"] == 5
        assert result == {"finance": {"result": [{"symbol": "MSFT"}]}}
