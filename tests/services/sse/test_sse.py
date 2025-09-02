from unittest.mock import AsyncMock, patch

from orjson import orjson

from src.models import SimpleQuote
from src.routes import stream
from tests.conftest import VERSION


class TestSSE:
    async def test_quotes_generator(self, mock_finance_client):
        """Test the quotes_generator function"""
        # Mock data
        symbols = ["AAPL", "NVDA"]
        mock_quotes = [
            SimpleQuote(
                symbol="AAPL",
                name="Apple Inc.",
                price="150.00",
                pre_market_price="150.50",
                after_hours_price="149.75",
                change="+1.25",
                percent_change="+0.84%",
            ),
            SimpleQuote(
                symbol="NVDA",
                name="NVIDIA Corporation",
                price="300.00",
                pre_market_price="305.00",
                after_hours_price="298.00",
                change="-2.00",
                percent_change="-0.67%",
            ),
        ]

        # Patch dependencies
        with patch("src.routes.stream.get_simple_quotes", return_value=mock_quotes), patch("asyncio.sleep", AsyncMock()):
            # Get generator
            generator = stream.quotes_generator(mock_finance_client, symbols)

            # Get first response
            response = await anext(generator)

            # Verify response format
            assert response.startswith("quote: ")
            assert response.endswith("\n\n")

            # Parse JSON data
            json_str = response[7:-2]  # Remove "quote: " prefix and "\n\n" suffix
            data = orjson.loads(json_str)

            # Verify data structure
            assert isinstance(data, list)
            assert len(data) == 2

            for quote in data:
                assert isinstance(quote, dict)
                assert "symbol" in quote
                assert "name" in quote
                assert "price" in quote
                assert "change" in quote
                assert "percentChange" in quote

    async def test_stream_quotes_endpoint(self, test_client):
        """Test the stream_quotes endpoint"""
        # Test data
        symbols_str = "AAPL,NVDA"
        mock_quotes = [
            SimpleQuote(
                symbol="AAPL",
                name="Apple Inc.",
                price="150.00",
                pre_market_price="150.50",
                after_hours_price="149.75",
                change="+1.25",
                percent_change="+0.84%",
            )
        ]

        # Mock the generator function directly to return only one item
        async def mock_quotes_generator(symbols):
            quotes = [quote if isinstance(quote, dict) else quote.model_dump(by_alias=True, exclude_none=True) for quote in mock_quotes]
            data = orjson.dumps(quotes).decode("utf-8")
            yield f"quote: {data}\n\n"
            # No sleep or infinite loop here

        # Patch the generator function and the get_simple_quotes function
        with patch("src.routes.stream.quotes_generator", return_value=mock_quotes_generator(["AAPL"])):
            # Make request
            response = test_client.get(f"{VERSION}/stream/quotes?symbols={symbols_str}")

            # Verify response headers
            assert response.status_code == 200
            assert response.headers["content-type"].startswith("text/event-stream")

            # Read the response content
            content = response.read()
            data_str = content.decode("utf-8")

            # Verify data contains expected format
            assert data_str.startswith("quote: ")
            assert "\n\n" in data_str

            # Parse JSON data
            json_str = data_str[7 : data_str.find("\n\n")]
            data = orjson.loads(json_str)

            # Verify data structure
            assert isinstance(data, list)
            assert len(data) == 1

            quote = data[0]
            assert quote["symbol"] == "AAPL"
            assert quote["name"] == "Apple Inc."
            assert quote["price"] == "150.00"
            assert quote["change"] == "+1.25"
            assert quote["percentChange"] == "+0.84%"

    async def test_stream_quotes_validation(self, test_client):
        """Test validation error handling in stream_quotes endpoint"""
        # Test missing symbols parameter
        response = test_client.get(f"{VERSION}/stream/quotes")
        assert response.status_code == 422

        data = response.json()
        assert "detail" in data
        assert "errors" in data
        assert "symbols" in data["errors"]
