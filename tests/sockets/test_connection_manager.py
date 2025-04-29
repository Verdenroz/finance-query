from unittest.mock import AsyncMock

from starlette.websockets import WebSocket, WebSocketDisconnect


class TestConnectionManager:
    async def test_connection_manager_connect(self, connection_manager, mock_websocket):
        """Test the connect method of ConnectionManager."""
        mock_task = AsyncMock()
        channel = "test_channel"

        await connection_manager.connect(mock_websocket, channel, mock_task)

        assert mock_websocket in connection_manager.active_connections[channel]
        assert channel in connection_manager.tasks
        assert len(connection_manager.active_connections[channel]) == 1

    async def test_connection_manager_disconnect(self, connection_manager, mock_websocket):
        """Test the disconnect method of ConnectionManager."""
        channel = "test_channel"
        mock_task = AsyncMock()

        # First connect the websocket
        await connection_manager.connect(mock_websocket, channel, mock_task)
        assert mock_websocket in connection_manager.active_connections[channel]

        # Then disconnect
        await connection_manager.disconnect(mock_websocket, channel)
        assert channel not in connection_manager.active_connections

    async def test_connection_manager_broadcast(self, connection_manager):
        """Test the broadcast method of ConnectionManager."""
        channel = "test_channel"
        mock_task = AsyncMock()

        # Create multiple mock websockets
        mock_ws1 = AsyncMock(spec=WebSocket)
        mock_ws2 = AsyncMock(spec=WebSocket)
        mock_ws3 = AsyncMock(spec=WebSocket)

        # Connect them all
        await connection_manager.connect(mock_ws1, channel, mock_task)
        await connection_manager.connect(mock_ws2, channel, mock_task)
        await connection_manager.connect(mock_ws3, channel, mock_task)

        # Simulate one of them raising an exception during broadcast
        mock_ws2.send_json.side_effect = WebSocketDisconnect()

        # Test broadcast
        test_message = {"test": "message"}
        await connection_manager.broadcast(channel, test_message)

        # Verify ws1 and ws3 received the message
        mock_ws1.send_json.assert_called_once_with(test_message)
        mock_ws3.send_json.assert_called_once_with(test_message)

        # Verify ws2 was disconnected due to the exception
        assert mock_ws2 not in connection_manager.active_connections[channel]

    async def test_connection_manager_close(self, connection_manager):
        """Test the close method of ConnectionManager."""
        channel = "test_channel"
        mock_task = AsyncMock()

        # Create multiple mock websockets
        mock_ws1 = AsyncMock(spec=WebSocket)
        mock_ws2 = AsyncMock(spec=WebSocket)

        # Connect them
        await connection_manager.connect(mock_ws1, channel, mock_task)
        await connection_manager.connect(mock_ws2, channel, mock_task)

        # Close the connection manager
        await connection_manager.close()

        # Verify websockets were closed
        mock_ws1.close.assert_called_once()
        mock_ws2.close.assert_called_once()

        # Verify all channels are cleared
        assert not connection_manager.active_connections

        # Verify all tasks are cleared
        assert not connection_manager.tasks
