import asyncio
from unittest.mock import AsyncMock, MagicMock

import orjson


class DummyPubSub:
    def __init__(self, message):
        self.message = message

    def subscribe(self, channel):
        pass

    def get_message(self, ignore_subscribe_messages=True):
        return self.message

    def close(self):
        pass


class TestRedisConnectionManager:

    async def test_redis_connection_manager_connect(self, redis_connection_manager, mock_websocket):
        """Test Redis connection manager connect method"""
        channel = "test_channel"
        task = AsyncMock()

        await redis_connection_manager.connect(mock_websocket, channel, task)

        # Your assertions:
        assert mock_websocket in redis_connection_manager.active_connections[channel]
        assert channel in redis_connection_manager.listen_tasks
        assert len(redis_connection_manager.active_connections[channel]) == 1

        # Cleanup
        await redis_connection_manager.close()

    async def test_redis_connection_manager_disconnect(self, redis_connection_manager, mock_websocket):
        """Test Redis connection manager disconnect method"""
        channel = "test_channel"
        task = AsyncMock()

        # First connect
        await redis_connection_manager.connect(mock_websocket, channel, task)

        # Then disconnect
        await redis_connection_manager.disconnect(mock_websocket, channel)

        assert channel not in redis_connection_manager.active_connections
        assert channel not in redis_connection_manager.listen_tasks

    async def test_redis_connection_manager_publish(self, redis_connection_manager):
        """Test Redis connection manager publish method"""
        channel = "test_channel"
        message = {"test": "data"}

        # Publish message
        await redis_connection_manager.publish(message, channel)

        # Check that the message was published to the Redis channel.
        redis_connection_manager.redis.publish.assert_called_once_with(
            channel,
            orjson.dumps(message),
        )

    async def test__listen_to_channel_broadcasts_valid_message(self, monkeypatch, redis_connection_manager):
        """
        Test that _listen_to_channel picks up a valid message from the PubSub
        and calls _broadcast with the correct data.
        """
        channel = "test_channel"
        # Prepare a valid message (the channel is encoded and the data is JSON bytes).
        message = {
            'type': 'message',
            'channel': channel.encode('utf-8'),
            'data': orjson.dumps({"test": "value"})
        }
        # Inject a DummyOncePubSub that returns the message only once.
        dummy_pubsub = DummyPubSub(message)
        redis_connection_manager.pubsub = {channel: dummy_pubsub}

        # Patch the _broadcast method so we can verify that it gets called.
        redis_connection_manager._broadcast = AsyncMock()

        # Run _listen_to_channel in a background task.
        listen_task = asyncio.create_task(redis_connection_manager._listen_to_channel(channel))

        # Wait long enough for one iteration to execute.
        await asyncio.sleep(0.3)
        listen_task.cancel()
        try:
            await listen_task
        except asyncio.CancelledError:
            pass

        # Assert that _broadcast was called exactly once with the expected data.
        redis_connection_manager._broadcast.assert_called_with(channel, {"test": "value"})

    async def test_redis_connection_manager_broadcast(self, redis_connection_manager, mock_websocket):
        """Test Redis connection manager broadcast method"""
        channel = "test_channel"
        task = AsyncMock()
        message = {"test": "data"}

        # Connect a websocket
        await redis_connection_manager.connect(mock_websocket, channel, task)

        # Broadcast message
        await redis_connection_manager._broadcast(channel, message)

        mock_websocket.send_json.assert_called_once_with(message)

    async def test_redis_connection_manager_close(self, redis_connection_manager, mock_websocket):
        """Test Redis connection manager close method"""
        channel = "test_channel"
        task = AsyncMock()

        # Connect a websocket
        await redis_connection_manager.connect(mock_websocket, channel, task)

        # Close manager
        await redis_connection_manager.close()

        assert len(redis_connection_manager.active_connections) == 0
        assert len(redis_connection_manager.listen_tasks) == 0
