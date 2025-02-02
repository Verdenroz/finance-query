import asyncio

from orjson import orjson
from redis.client import PubSub
from starlette.websockets import WebSocket, WebSocketDisconnect


class RedisConnectionManager:
    """
    Manages active websocket connections and listens to Redis channels to broadcast messages to clients.
    """

    def __init__(self, redis):
        self.active_connections: dict[str, list[WebSocket]] = {}
        self.pubsub: dict[str, PubSub] = {}
        self.listen_tasks: dict[str, asyncio.Task] = {}
        self.tasks: dict[str, asyncio.Task] = {}
        self.redis = redis

    async def connect(self, websocket: WebSocket, channel: str, task: callable):
        """
        Connects a websocket to a channel and listens for messages.
        :param websocket: the client websocket connection
        :param channel: the channel to subscribe to
        :param task: the continuous function to fetch data and publish to the channel
        :return:
        """
        if channel not in self.active_connections:
            self.active_connections[channel] = []
            self.pubsub[channel] = self.redis.pubsub()

        self.active_connections[channel].append(websocket)

        if channel not in self.listen_tasks:
            self.listen_tasks[channel] = asyncio.create_task(self._listen_to_channel(channel))
        if channel not in self.tasks:
            self.tasks[channel] = asyncio.create_task(task())

    async def disconnect(self, websocket: WebSocket, channel: str):
        """
        Disconnects a websocket from a channel.
        :param websocket: the client websocket connection to disconnect
        :param channel: the channel to disconnect from
        :return:
        """
        self.active_connections[channel].remove(websocket)
        if not self.active_connections[channel]:
            del self.active_connections[channel]

            self.listen_tasks[channel].cancel()
            del self.listen_tasks[channel]

            self.tasks[channel].cancel()
            del self.tasks[channel]

            await self.pubsub[channel].unsubscribe(channel)
            await self.pubsub[channel].close()
            del self.pubsub[channel]

    async def _listen_to_channel(self, channel: str):
        """
        Listens to a Redis channel and broadcasts messages to clients.
        :param channel: the channel to subscribe to with Redis PubSub
        :return:
        """
        await self.pubsub[channel].subscribe(channel)
        while True:
            message = await self.pubsub[channel].get_message(ignore_subscribe_messages=True)
            if message and message['type'] == 'message':
                message_channel = message['channel'].decode('utf-8')
                if message_channel == channel:
                    data = orjson.loads(message['data'])
                    await self._broadcast(channel, data)

    async def _broadcast(self, channel: str, message: dict):
        """
        Broadcasts a message to all clients connected to a channel.
        :param channel: the channel to broadcast to
        :param message: the message json to broadcast
        :return:
        """
        if channel in self.active_connections:
            disconnected_clients = []
            for connection in self.active_connections[channel]:
                try:
                    await connection.send_json(message)
                except WebSocketDisconnect:
                    disconnected_clients.append(connection)
            for client in disconnected_clients:
                await self.disconnect(client, channel)

    async def publish(self, message: dict | list, channel: str):
        """
        Publishes a message to a Redis channel.
        :param message: the json message to publish
        :param channel: the channel to publish to
        :return:
        """
        await self.redis.publish(channel, orjson.dumps(message))

    async def close(self):
        """
        Closes all connections and tasks.
        :return:
        """
        for channel in self.active_connections:
            for connection in self.active_connections[channel]:
                await self.disconnect(connection, channel)

        for pubsub in self.pubsub.values():
            pubsub.close()

        for task in self.tasks.values():
            task.cancel()
