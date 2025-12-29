from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from fastapi import HTTPException

from src.clients.yahoo_client import YahooFinanceClient


@pytest.fixture
def yahoo_client():
    """Create a YahooFinanceClient instance for testing"""
    cookies = {"test_cookie": "test_value"}
    crumb = "test_crumb_123"
    return YahooFinanceClient(cookies=cookies, crumb=crumb, timeout=10)


@pytest.fixture
def mock_response():
    """Create a mock HTTP response"""
    response = MagicMock()
    response.status_code = 200
    response.text = '{"test": "data"}'
    return response


class TestYahooFinanceClient:
    """Test suite for YahooFinanceClient"""

    def test_init(self):
        """Test YahooFinanceClient initialization"""
        cookies = {"session": "abc123"}
        crumb = "crumb_xyz"
        client = YahooFinanceClient(cookies=cookies, crumb=crumb, timeout=15, proxy="http://proxy.com")

        assert client.crumb == crumb
        assert client.timeout == 15
        assert client.proxies == {"http": "http://proxy.com", "https": "http://proxy.com"}

    async def test_yahoo_request_adds_crumb(self, yahoo_client, mock_response):
        """Test that _yahoo_request adds crumb to params"""
        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_response)):
            await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            yahoo_client.fetch.assert_called_once()
            call_kwargs = yahoo_client.fetch.call_args.kwargs
            assert call_kwargs["params"]["crumb"] == "test_crumb_123"

    async def test_yahoo_request_adds_user_agent(self, yahoo_client, mock_response):
        """Test that _yahoo_request adds User-Agent header"""
        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_response)):
            await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            call_kwargs = yahoo_client.fetch.call_args.kwargs
            assert "User-Agent" in call_kwargs["headers"]
            assert "Mozilla" in call_kwargs["headers"]["User-Agent"]

    async def test_yahoo_request_handles_401(self, yahoo_client):
        """Test that _yahoo_request raises HTTPException for 401 status"""
        mock_resp = MagicMock()
        mock_resp.status_code = 401
        mock_resp.text = "Unauthorized"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            assert exc_info.value.status_code == 401
            assert "auth failed" in exc_info.value.detail.lower()

    async def test_yahoo_request_handles_404(self, yahoo_client):
        """Test that _yahoo_request raises HTTPException for 404 status"""
        mock_resp = MagicMock()
        mock_resp.status_code = 404
        mock_resp.text = "Not Found"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            assert exc_info.value.status_code == 404
            assert "not found" in exc_info.value.detail.lower()

    async def test_yahoo_request_handles_429(self, yahoo_client):
        """Test that _yahoo_request raises HTTPException for 429 status"""
        mock_resp = MagicMock()
        mock_resp.status_code = 429
        mock_resp.text = "Too Many Requests"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            assert exc_info.value.status_code == 429
            assert "rate-limit" in exc_info.value.detail.lower()

    async def test_yahoo_request_handles_generic_error(self, yahoo_client):
        """Test that _yahoo_request raises HTTPException for generic errors"""
        mock_resp = MagicMock()
        mock_resp.status_code = 500
        mock_resp.text = "Internal Server Error"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client._yahoo_request("https://test.yahoo.com/api")

            assert exc_info.value.status_code == 500

    async def test_json_success(self, yahoo_client, mock_response):
        """Test that _json successfully parses JSON response"""
        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_response)):
            result = await yahoo_client._json("https://test.yahoo.com/api")

            assert result == {"test": "data"}

    async def test_json_parse_error(self, yahoo_client):
        """Test that _json handles JSON parse errors"""
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.text = "not valid json"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client._json("https://test.yahoo.com/api")

            assert exc_info.value.status_code == 500
            assert "Failed to parse JSON" in exc_info.value.detail

    async def test_get_quote(self, yahoo_client, mock_response):
        """Test get_quote method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"quote": "data"})) as mock_json:
            result = await yahoo_client.get_quote("AAPL")

            assert result == {"quote": "data"}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "AAPL" in call_args.args[0]
            assert "modules" in call_args.kwargs["params"]

    async def test_get_simple_quotes(self, yahoo_client):
        """Test get_simple_quotes method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"quotes": []})) as mock_json:
            result = await yahoo_client.get_simple_quotes(["AAPL", "MSFT", "GOOG"])

            assert result == {"quotes": []}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert call_args.kwargs["params"]["symbols"] == "AAPL,MSFT,GOOG"

    async def test_get_chart(self, yahoo_client):
        """Test get_chart method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"chart": "data"})) as mock_json:
            result = await yahoo_client.get_chart("AAPL", "1d", "1mo")

            assert result == {"chart": "data"}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "AAPL" in call_args.args[0]
            assert call_args.kwargs["params"]["interval"] == "1d"
            assert call_args.kwargs["params"]["range"] == "1mo"

    async def test_search(self, yahoo_client):
        """Test search method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"results": []})) as mock_json:
            result = await yahoo_client.search("AAPL", hits=10)

            assert result == {"results": []}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert call_args.kwargs["params"]["q"] == "AAPL"
            assert call_args.kwargs["params"]["quotesCount"] == 10

    async def test_get_similar_quotes(self, yahoo_client):
        """Test get_similar_quotes method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"similar": []})) as mock_json:
            result = await yahoo_client.get_similar_quotes("AAPL", limit=5)

            assert result == {"similar": []}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "AAPL" in call_args.args[0]
            assert call_args.kwargs["params"]["count"] == 5

    async def test_get_fundamentals_timeseries(self, yahoo_client):
        """Test get_fundamentals_timeseries method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"timeseries": {}})) as mock_json:
            result = await yahoo_client.get_fundamentals_timeseries(
                symbol="AAPL", period1=1640995200, period2=1672531200, types=["annualTotalRevenue", "quarterlyNetIncome"]
            )

            assert result == {"timeseries": {}}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "AAPL" in call_args.args[0]
            assert call_args.kwargs["params"]["period1"] == 1640995200
            assert call_args.kwargs["params"]["period2"] == 1672531200
            assert call_args.kwargs["params"]["type"] == "annualTotalRevenue,quarterlyNetIncome"

    async def test_get_quote_summary(self, yahoo_client):
        """Test get_quote_summary method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"quoteSummary": {}})) as mock_json:
            result = await yahoo_client.get_quote_summary(symbol="AAPL", modules=["institutionOwnership", "majorHoldersBreakdown"])

            assert result == {"quoteSummary": {}}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "AAPL" in call_args.args[0]
            assert call_args.kwargs["params"]["modules"] == "institutionOwnership,majorHoldersBreakdown"

    async def test_get_quote_type(self, yahoo_client):
        """Test get_quote_type method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"quoteType": {"quartrId": "123"}})) as mock_json:
            result = await yahoo_client.get_quote_type("AAPL")

            assert result == {"quoteType": {"quartrId": "123"}}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "quoteType/AAPL" in call_args.args[0]

    async def test_get_earnings_calls_list(self, yahoo_client):
        """Test get_earnings_calls_list method"""
        mock_html = """
        <html>
            <body>
                <a href="/quote/AAPL/foo-Q4-2024-earnings_call-12345">Q4 2024</a>
                <a href="/quote/AAPL/bar-Q3-2024-earnings_call-67890">Q3 2024</a>
            </body>
        </html>
        """
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.text = mock_html

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            result = await yahoo_client.get_earnings_calls_list("AAPL")

            assert len(result) == 2
            assert result[0]["eventId"] == "12345"
            assert result[0]["quarter"] == "Q4"
            assert result[0]["year"] == 2024
            assert result[1]["eventId"] == "67890"
            assert result[1]["quarter"] == "Q3"
            assert result[1]["year"] == 2024

    async def test_get_earnings_calls_list_handles_http_error(self, yahoo_client):
        """Test get_earnings_calls_list handles HTTP errors"""
        mock_resp = MagicMock()
        mock_resp.status_code = 500
        mock_resp.text = "Server Error"

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            with pytest.raises(HTTPException) as exc_info:
                await yahoo_client.get_earnings_calls_list("AAPL")

            assert exc_info.value.status_code == 500

    async def test_get_earnings_transcript(self, yahoo_client):
        """Test get_earnings_transcript method"""
        with patch.object(yahoo_client, "_json", new=AsyncMock(return_value={"transcriptContent": {}})) as mock_json:
            result = await yahoo_client.get_earnings_transcript(event_id="12345", company_id="comp123")

            assert result == {"transcriptContent": {}}
            mock_json.assert_called_once()
            call_args = mock_json.call_args
            assert "transcript" in call_args.args[0]
            assert call_args.kwargs["params"]["eventId"] == "12345"
            assert call_args.kwargs["params"]["quartrId"] == "comp123"

    async def test_get_earnings_calls_list_deduplicates_event_ids(self, yahoo_client):
        """Test that get_earnings_calls_list deduplicates event IDs"""
        mock_html = """
        <html>
            <body>
                <a href="/quote/AAPL/foo-Q4-2024-earnings_call-12345">Q4 2024</a>
                <a href="/quote/AAPL/bar-Q4-2024-earnings_call-12345">Q4 2024 Duplicate</a>
            </body>
        </html>
        """
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.text = mock_html

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            result = await yahoo_client.get_earnings_calls_list("AAPL")

            # Should only return 1 result despite 2 links with same eventId
            assert len(result) == 1
            assert result[0]["eventId"] == "12345"

    async def test_get_earnings_calls_list_no_calls_found(self, yahoo_client):
        """Test get_earnings_calls_list when no calls are found"""
        mock_html = """
        <html>
            <body>
                <p>No earnings calls available</p>
            </body>
        </html>
        """
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.text = mock_html

        with patch.object(yahoo_client, "fetch", new=AsyncMock(return_value=mock_resp)):
            result = await yahoo_client.get_earnings_calls_list("AAPL")

            assert result == []
