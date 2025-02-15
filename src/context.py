from contextvars import ContextVar

from fastapi import Request
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp, Receive, Scope, Send
from starlette.websockets import WebSocket

request_context: ContextVar[Request | WebSocket] = ContextVar("request_context")


class RequestContextMiddleware(BaseHTTPMiddleware):
    """
    Middleware to set the request context for FastAPI and Starlette apps.
    We can use this to access the request object in any part of the app.
    """
    def __init__(self, app: ASGIApp):
        super().__init__(app)

    async def dispatch(self, request: Request, call_next):
        request_context.set(request)
        response = await call_next(request)
        return response

    async def __call__(self, scope: Scope, receive: Receive, send: Send):
        if scope["type"] == "http":
            request = Request(scope, receive=receive, send=send)
            request_context.set(request)
        elif scope["type"] == "websocket":
            websocket = WebSocket(scope, receive=receive, send=send)
            request_context.set(websocket)
        await super().__call__(scope, receive, send)
