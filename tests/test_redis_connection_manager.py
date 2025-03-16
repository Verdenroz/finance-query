from unittest.mock import AsyncMock, MagicMock


async def test_redis_connection_manager_connect(redis_connection_manager, mock_websocket):
    """Test Redis connection manager connect method"""
    channel = "test_channel"
    task = AsyncMock()

    await redis_connection_manager.connect(mock_websocket, channel, task)

    assert mock_websocket in redis_connection_manager.active_connections[channel]
    assert channel in redis_connection_manager.listen_tasks
    assert len(redis_connection_manager.active_connections[channel]) == 1


async def test_redis_connection_manager_disconnect(redis_connection_manager, mock_websocket):
    """Test Redis connection manager disconnect method"""
    channel = "test_channel"
    task = AsyncMock()

    # First connect
    await redis_connection_manager.connect(mock_websocket, channel, task)

    # Then disconnect
    await redis_connection_manager.disconnect(mock_websocket, channel)

    assert channel not in redis_connection_manager.active_connections
    assert channel not in redis_connection_manager.listen_tasks


async def test_redis_connection_manager_publish(redis_connection_manager):
    """Test Redis connection manager publish method"""
    channel = "test_channel"
    message = {"test": "data"}

    # Replace the redis.publish with a regular Mock instead of AsyncMock
    redis_connection_manager.redis.publish = MagicMock()

    # Publish message
    await redis_connection_manager.publish(message, channel)

    # Check that the message was published to the Redis channel
    redis_connection_manager.redis.publish.assert_called_once_with(
        channel,
        b'{"test":"data"}',
    )


async def test_redis_connection_manager_broadcast(redis_connection_manager, mock_websocket):
    """Test Redis connection manager broadcast method"""
    channel = "test_channel"
    task = AsyncMock()
    message = {"test": "data"}

    # Connect a websocket
    await redis_connection_manager.connect(mock_websocket, channel, task)

    # Broadcast message
    await redis_connection_manager._broadcast(channel, message)

    mock_websocket.send_json.assert_called_once_with(message)


async def test_redis_connection_manager_close(redis_connection_manager, mock_websocket):
    """Test Redis connection manager close method"""
    channel = "test_channel"
    task = AsyncMock()

    # Connect a websocket
    await redis_connection_manager.connect(mock_websocket, channel, task)

    # Close manager
    await redis_connection_manager.close()

    assert len(redis_connection_manager.active_connections) == 0
    assert len(redis_connection_manager.listen_tasks) == 0
