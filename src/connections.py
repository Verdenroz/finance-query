import asyncio
from typing import Dict, List

from orjson import orjson
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.utils import r


class RedisConnectionManager:
    """
    Manages active websocket connections and listens to Redis channels to broadcast messages to clients.
    """

    def __init__(self):
        self.active_connections: Dict[str, List[WebSocket]] = {}
        self.pubsub = r.pubsub()
        self.task = None

    async def connect(self, websocket: WebSocket, channel: str):
        """
        Connects a websocket to a channel and listens for messages.
        :param websocket: the client websocket connection
        :param channel: the channel to subscribe to
        :return:
        """
        if channel not in self.active_connections:
            self.active_connections[channel] = []

        self.active_connections[channel].append(websocket)

        self.task = asyncio.create_task(self._listen_to_channel(channel))

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
            await self.pubsub.unsubscribe(channel)

    async def _listen_to_channel(self, channel: str):
        """
        Listens to a Redis channel and broadcasts messages to clients.
        :param channel: the channel to subscribe to with Redis PubSub
        :return:
        """
        await self.pubsub.subscribe(channel)
        try:
            while True:
                message = await self.pubsub.get_message()
                if message and message['type'] == 'message':
                    data = orjson.loads(message['data'])
                    await self._broadcast(channel, data)
        except RuntimeError:
            # When channel name changes, RuntimeError is raised
            self.task.cancel()

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

    @staticmethod
    async def publish(message: dict | list, channel: str):
        """
        Publishes a message to a Redis channel.
        :param message: the json message to publish
        :param channel: the channel to publish to
        :return:
        """
        await r.publish(channel, orjson.dumps(message))
