from datetime import UTC
from unittest.mock import AsyncMock, patch

from pydantic import BaseModel
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.models import SimpleQuote
from src.routes import sockets
from src.routes.sockets import handle_websocket_connection, safe_convert_to_dict

# Test data
symbol = "NVDA"


class TestSocketsHandler:
    async def test_websocket_profile(self, test_client, monkeypatch):
        """Test the websocket_profile endpoint using the TestClient and monkeypatch"""
        mock_quote_data = {
            "symbol": "NVDA",
            "name": "NVIDIA Corporation",
            "sector": "Technology",
            "industry": "Semiconductors",
            "marketCap": 1456000000000,
            "employees": 26000,
            "description": "NVIDIA Corporation designs and manufactures graphics processors and related software.",
        }

        # Create mock functions
        async def mock_get_quotes(*args, **kwargs):
            return [mock_quote_data]

        async def mock_get_similar_quotes(*args, **kwargs):
            return []

        async def mock_get_sector_for_symbol(*args, **kwargs):
            return None

        async def mock_scrape_news_for_quote(*args, **kwargs):
            return []

        # Apply monkey patches
        monkeypatch.setattr(sockets, "get_quotes", mock_get_quotes)
        monkeypatch.setattr(sockets, "get_similar_quotes", mock_get_similar_quotes)
        monkeypatch.setattr(sockets, "get_sector_for_symbol", mock_get_sector_for_symbol)
        monkeypatch.setattr(sockets, "scrape_news_for_quote", mock_scrape_news_for_quote)

        with patch("src.routes.sockets.validate_websocket", return_value=(True, {})):
            # Connect to the websocket
            with test_client.websocket_connect(f"/profile/{symbol}") as websocket:
                # Receive the response
                response = websocket.receive_json()

                # Verify the response structure
                assert "quote" in response
                assert "similar" in response
                assert "sectorPerformance" in response
                assert "news" in response

                # Verify quote data
                quote = response["quote"]
                assert isinstance(quote, dict)
                assert quote["symbol"] == symbol
                assert quote["name"] == "NVIDIA Corporation"
                assert quote["sector"] == "Technology"
                assert quote["industry"] == "Semiconductors"

                # Verify other fields are the expected types
                assert isinstance(response["similar"], list)
                assert response["news"] == []
                assert response["sectorPerformance"] is None

    async def test_websocket_quotes(self, test_client):
        """Test the websocket_quotes endpoint using the TestClient"""
        # Mock data
        symbols_str = "AAPL,NVDA,MSFT"
        mock_quotes = [
            SimpleQuote(
                symbol="AAPL",
                name="Apple Inc.",
                price="150.00",
                pre_market_price="150.50",
                after_hours_price="149.75",
                change="+1.25",
                percent_change="+0.84%",
                logo="https://logo.clearbit.com/apple.com",
            ),
            {
                "symbol": "NVDA",
                "name": "NVIDIA Corporation",
                "price": "300.00",
                "preMarketPrice": "305.00",
                "afterHoursPrice": "298.00",
                "change": "-2.00",
                "percentChange": "-0.67%",
                "logo": "https://logo.clearbit.com/nvidia.com",
            },
        ]
        metadata = {"rate_limit": 2000, "remaining_requests": 1999, "reset": 86400}

        with (
            patch("src.routes.sockets.get_simple_quotes", return_value=mock_quotes),
            patch("src.routes.sockets.validate_websocket", return_value=(True, metadata)),
        ):
            with test_client.websocket_connect("/quotes") as websocket:
                websocket.send_text(symbols_str)
                response = websocket.receive_json()

                # Verify response is a list with expected length
                assert isinstance(response, list)
                assert len(response) == 3

                # Check that metadata is the first item in the list
                assert response[0] == metadata

                # Check data structure of each quote (skip the metadata)
                for quote in response[1:]:
                    assert isinstance(quote, dict)
                    assert "symbol" in quote
                    assert "name" in quote
                    assert "price" in quote
                    assert "change" in quote
                    assert "percentChange" in quote

                    # Check data types
                    assert isinstance(quote["symbol"], str)
                    assert isinstance(quote["name"], str)
                    assert isinstance(quote["price"], str)
                    assert isinstance(quote["change"], str)
                    assert isinstance(quote["percentChange"], str)

                # Verify specific symbols are present
                symbols = [q.get("symbol") for q in response[1:]]
                assert "AAPL" in symbols
                assert "NVDA" in symbols

    async def test_websocket_market(self, test_client, monkeypatch):
        """Test the websocket_market endpoint"""
        # Mock data for each function that get_market_info calls
        mock_actives = [
            {
                "symbol": "AAPL",
                "name": "Apple Inc.",
                "price": 150.0,
                "change": 2.0,
                "percentChange": 1.35,
                "volume": 75000000,
            },
            {
                "symbol": "MSFT",
                "name": "Microsoft Corp",
                "price": 305.0,
                "change": 3.5,
                "percentChange": 1.16,
                "volume": 45000000,
            },
        ]

        mock_gainers = [
            {
                "symbol": "XYZ",
                "name": "XYZ Corp",
                "price": 45.0,
                "change": 5.0,
                "percentChange": 12.5,
                "volume": 3000000,
            },
            {
                "symbol": "ABC",
                "name": "ABC Tech",
                "price": 78.0,
                "change": 7.8,
                "percentChange": 11.1,
                "volume": 2500000,
            },
        ]

        mock_losers = [
            {
                "symbol": "DEF",
                "name": "DEF Inc",
                "price": 30.0,
                "change": -4.0,
                "percentChange": -11.76,
                "volume": 1800000,
            },
            {
                "symbol": "GHI",
                "name": "GHI Corp",
                "price": 25.0,
                "change": -3.0,
                "percentChange": -10.71,
                "volume": 1500000,
            },
        ]

        mock_indices = [
            {"symbol": "^GSPC", "name": "S&P 500", "price": 4500.0, "change": 25.0, "percentChange": 0.56},
            {"symbol": "^DJI", "name": "Dow Jones", "price": 35000.0, "change": 150.0, "percentChange": 0.43},
            {"symbol": "^IXIC", "name": "NASDAQ", "price": 14000.0, "change": 100.0, "percentChange": 0.72},
        ]

        mock_news = [
            {"title": "Market News 1", "link": "https://example.com/news1", "source": "Example News"},
            {"title": "Market News 2", "link": "https://example.com/news2", "source": "Example News"},
        ]

        mock_sectors = [
            {"name": "Technology", "percentChange": 1.2},
            {"name": "Healthcare", "percentChange": -0.3},
            {"name": "Financials", "percentChange": 0.8},
        ]

        # Mock functions
        async def mock_get_actives():
            return mock_actives

        async def mock_get_gainers():
            return mock_gainers

        async def mock_get_losers():
            return mock_losers

        async def mock_get_indices(*args, **kwargs):
            return mock_indices

        async def mock_scrape_general_news():
            return mock_news

        async def mock_get_sectors():
            return mock_sectors

        # Apply the monkeypatches
        monkeypatch.setattr(sockets, "get_actives", mock_get_actives)
        monkeypatch.setattr(sockets, "get_gainers", mock_get_gainers)
        monkeypatch.setattr(sockets, "get_losers", mock_get_losers)
        monkeypatch.setattr(sockets, "get_indices", mock_get_indices)
        monkeypatch.setattr(sockets, "scrape_general_news", mock_scrape_general_news)
        monkeypatch.setattr(sockets, "get_sectors", mock_get_sectors)

        with patch("src.routes.sockets.validate_websocket", return_value=(True, {})):
            # Connect to the websocket and test
            with test_client.websocket_connect("/market") as websocket:
                # Receive the response
                response = websocket.receive_json()

                # Verify the response structure matches what we expect
                assert "actives" in response
                assert "gainers" in response
                assert "losers" in response
                assert "indices" in response
                assert "headlines" in response
                assert "sectors" in response

                # Verify the data in the response matches our mocked data
                assert response["actives"] == mock_actives
                assert response["gainers"] == mock_gainers
                assert response["losers"] == mock_losers
                assert response["indices"] == mock_indices
                assert response["headlines"] == mock_news
                assert response["sectors"] == mock_sectors

    async def test_market_status_websocket(self, test_client, monkeypatch):
        """Test the market_status_websocket endpoint using TestClient"""
        # Mock market status data
        mock_status = "Closed"
        mock_reason = "Weekend"
        mock_timestamp = "2025-03-16T01:34:18.130233+00:00"

        # Mock function
        def mock_get_market_status(self, *args, **kwargs):
            return mock_status, mock_reason

        # Create a mock datetime
        class MockDateTime:
            @staticmethod
            def now(tz=None):
                from datetime import datetime

                return datetime.fromisoformat(mock_timestamp.replace("+00:00", "")).replace(tzinfo=UTC)

        from utils.dependencies import Schedule

        # Patches
        monkeypatch.setattr(Schedule, "get_market_status", mock_get_market_status)
        monkeypatch.setattr("src.routes.sockets.datetime", MockDateTime)

        with patch("src.routes.sockets.validate_websocket", return_value=(True, {})):
            # Connect to the websocket
            with test_client.websocket_connect("/hours") as websocket:
                # Receive the response
                response = websocket.receive_json()

                # Expected response
                expected_response = {"status": mock_status, "reason": mock_reason, "timestamp": mock_timestamp}

                # Verify the response
                assert response == expected_response

    def test_safe_convert_to_dict(self):
        """Test the safe_convert_to_dict function with various input types"""

        class SampleModel(BaseModel):
            name: str
            value: int

        # Test case 1: Empty list
        assert safe_convert_to_dict([]) == []

        # Test case 2: List with dictionaries
        input_dicts = [{"a": 1}, {"b": 2}]
        assert safe_convert_to_dict(input_dicts) == input_dicts

        # Test case 3: List with Pydantic models
        sample_objects = [SampleModel(name="test1", value=1), SampleModel(name="test2", value=2)]
        expected_dicts = [{"name": "test1", "value": 1}, {"name": "test2", "value": 2}]
        assert safe_convert_to_dict(sample_objects) == expected_dicts

        # Test case 4: Mixed list with dicts and Pydantic models
        mixed_list = [{"a": 1}, SampleModel(name="test", value=2)]
        expected_mixed = [{"a": 1}, {"name": "test", "value": 2}]
        assert safe_convert_to_dict(mixed_list) == expected_mixed

        # Test case 5: List with items that can't be converted
        non_convertible = [1, "string", True]
        assert safe_convert_to_dict(non_convertible) == [[], [], []]

        # Test case 6: Custom default value
        custom_default = {"default": True}
        assert safe_convert_to_dict([1, 2, 3], default=custom_default) == [
            custom_default,
            custom_default,
            custom_default,
        ]

        # Test case 7: None input
        assert safe_convert_to_dict(None) == []

        # Test case 8: Invalid input type (not iterable)
        assert safe_convert_to_dict(123) == []

    async def test_handle_disconnect(self):
        """Test the handle_websocket_connection function with a disconnect during receive_text"""
        # Mock dependencies
        websocket = AsyncMock(spec=WebSocket)

        # Create connection managers with proper attributes
        regular_connection_manager = AsyncMock()
        regular_connection_manager.active_connections = {}

        # Mock validate_websocket to return valid response
        with patch("src.routes.sockets.validate_websocket", return_value=(True, {})):
            # Mock data fetcher
            async def mock_data_fetcher():
                return {"data": "test"}

            # WebSocketDisconnect during receive_text
            websocket.receive_text.side_effect = WebSocketDisconnect()

            await handle_websocket_connection(
                websocket=websocket,
                channel="test_channel",
                data_fetcher=mock_data_fetcher,
                connection_manager=regular_connection_manager,
            )

            # Verify disconnect was called
            regular_connection_manager.disconnect.assert_called_once_with(websocket, "test_channel")

    async def test_handle_websocket_connection_invalid(self):
        """Test websocket connection with invalid authentication"""
        # Mock dependencies
        websocket = AsyncMock(spec=WebSocket)
        data_fetcher = AsyncMock(return_value={"data": "test"})
        connection_manager = AsyncMock()

        # Mock validate_websocket to return invalid response
        with patch("src.routes.sockets.validate_websocket", return_value=(False, {})):
            await handle_websocket_connection(
                websocket=websocket,
                channel="test_channel",
                data_fetcher=data_fetcher,
                connection_manager=connection_manager,
            )

            # Verify accept was not called
            websocket.accept.assert_not_called()
            # Verify no further processing happened
            connection_manager.connect.assert_not_called()

    async def test_handle_websocket_connection_with_metadata(self):
        """Test websocket connection with metadata processing"""
        # Mock dependencies
        websocket = AsyncMock(spec=WebSocket)
        connection_manager = AsyncMock()
        connection_manager.active_connections = {"test_channel": []}

        # Mock data with initial results
        initial_data = {"quote": "AAPL", "price": 150.0}

        # Mock metadata
        metadata = {"rate_limit": 2000, "remaining_requests": 1999, "reset": 86400}

        async def mock_data_fetcher():
            return initial_data

        # Simulate websocket disconnect after first message
        websocket.receive_text.side_effect = WebSocketDisconnect()

        # Mock validate_websocket to return valid response with metadata
        with patch("src.routes.sockets.validate_websocket", return_value=(True, metadata)):
            await handle_websocket_connection(
                websocket=websocket,
                channel="test_channel",
                data_fetcher=mock_data_fetcher,
                connection_manager=connection_manager,
            )

            # Verify metadata was merged with the result and sent
            # Metadata should update initial_data, becoming the response
            expected_data = metadata.copy()
            expected_data.update(initial_data)
            websocket.send_json.assert_called_once_with(expected_data)

    async def test_handle_websocket_connection_disconnect_in_fetch_data(self):
        """Test the except WebSocketDisconnect branch in fetch_data"""
        # Mock dependencies
        websocket = AsyncMock(spec=WebSocket)
        connection_manager = AsyncMock()
        connection_manager.active_connections = {"test_channel": []}

        # Create a data_fetcher that raises WebSocketDisconnect on second call
        call_count = 0

        async def mock_data_fetcher():
            nonlocal call_count
            call_count += 1
            if call_count > 1:
                raise WebSocketDisconnect()
            return {"data": "test"}

        # Mock the connect method to actually run fetch_data
        async def connect_and_run(*args, **kwargs):
            fetch_data_func = args[2]
            await fetch_data_func()  # This should eventually trigger the exception

        connection_manager.connect.side_effect = connect_and_run
        websocket.receive_text.side_effect = WebSocketDisconnect()

        # Patch sleep to avoid waiting and validate_websocket to return valid
        with patch("src.routes.sockets.validate_websocket", return_value=(True, {})), patch("asyncio.sleep", AsyncMock()):
            await handle_websocket_connection(
                websocket=websocket,
                channel="test_channel",
                data_fetcher=mock_data_fetcher,
                connection_manager=connection_manager,
            )

            # Verify disconnect was called from within fetch_data exception handler
            connection_manager.disconnect.assert_called_with(websocket, "test_channel")
