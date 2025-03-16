import asyncio
from starlette.websockets import WebSocket, WebSocketDisconnect


class ConnectionManager:
    """
    Manages active websocket connections and broadcasts messages to clients.
    """

    def __init__(self):
        self.active_connections: dict[str, list[WebSocket]] = {}
        self.tasks: dict[str, asyncio.Task] = {}

    async def connect(self, websocket: WebSocket, channel: str, task: callable):
        """
        Connects a websocket to a channel and starts a task.
        :param websocket: the client websocket connection
        :param channel: the channel to subscribe to
        :param task: the continuous function to fetch data and publish to the channel
        :return:
        """
        if channel not in self.active_connections:
            self.active_connections[channel] = []

        self.active_connections[channel].append(websocket)

        if channel not in self.tasks:
            self.tasks[channel] = asyncio.create_task(task())

    async def disconnect(self, websocket: WebSocket, channel: str):
        """
        Disconnects a websocket from a channel.
        :param websocket: the client websocket connection to disconnect
        :param channel: the channel to disconnect from
        :return:
        """
        if channel in self.active_connections:
            self.active_connections[channel].remove(websocket)
            if not self.active_connections[channel]:
                del self.active_connections[channel]

                if channel in self.tasks:
                    self.tasks[channel].cancel()
                    del self.tasks[channel]

    async def broadcast(self, channel: str, message: dict):
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

    async def close(self):
        """
        Clean up all connections and tasks.
        """
        # Create a copy of channels to avoid modifying dict during iteration
        channels = list(self.active_connections.keys())

        for channel in channels:
            # Create a copy of connections to avoid modifying list during iteration
            connections = self.active_connections[channel].copy()
            for connection in connections:
                await connection.close()  # Close the websocket connection
                await self.disconnect(connection, channel)

        # Cancel all remaining tasks
        for task in self.tasks.values():
            task.cancel()

        # Clear any remaining data
        self.active_connections.clear()
        self.tasks.clear()
