import time
from collections.abc import Callable

from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware

from src.utils.logging import (
    get_logger,
    log_api_request,
    log_api_response,
    log_critical_system_failure,
    log_performance,
    log_route_error,
    log_route_request,
    log_route_success,
    set_request_id,
)

logger = get_logger(__name__)


class LoggingMiddleware(BaseHTTPMiddleware):
    """Middleware to log HTTP requests and responses."""

    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        # Set correlation ID for this request
        correlation_id = set_request_id()

        # Extract request info
        method = request.method
        path = request.url.path
        query_params = dict(request.query_params) if request.query_params else None

        # Extract route information for enhanced logging
        route_name = self._extract_route_name(request)
        route_params = self._extract_route_params(request)

        # Log incoming request (general HTTP logging)
        log_api_request(logger, method, path, query_params)

        # Log route-specific request if this is an API route
        if route_name and path.startswith("/v1/"):
            log_route_request(logger, route_name, route_params)

        # Process request and measure time
        start_time = time.perf_counter()
        try:
            response = await call_next(request)
            duration_ms = (time.perf_counter() - start_time) * 1000

            # Log performance metrics
            operation = f"{method} {path}"
            log_performance(logger, operation, duration_ms)

            # Log response (general HTTP logging)
            log_api_response(logger, method, path, response.status_code, duration_ms)

            # Log route-specific success if this is an API route
            if route_name and path.startswith("/v1/") and 200 <= response.status_code < 300:
                log_route_success(logger, route_name, route_params)

            # Add correlation ID to response headers
            response.headers["X-Correlation-ID"] = correlation_id

            return response

        except Exception as e:
            duration_ms = (time.perf_counter() - start_time) * 1000

            # Log performance even for failed requests
            operation = f"{method} {path}"
            log_performance(logger, operation, duration_ms)

            # Log route-specific error if this is an API route
            if route_name and path.startswith("/v1/"):
                log_route_error(logger, route_name, route_params, e)

            # Check if this is a system-level critical failure
            if isinstance(e, MemoryError | SystemExit | KeyboardInterrupt):
                log_critical_system_failure(logger, "request_processing", e, {"method": method, "path": path, "duration_ms": duration_ms})
            else:
                # General error logging for non-API routes or additional context
                if not route_name or not path.startswith("/v1/"):
                    logger.error(
                        "Request failed with exception",
                        extra={"method": method, "path": path, "duration_ms": duration_ms, "error": str(e), "error_type": type(e).__name__},
                        exc_info=True,
                    )
            raise

    def _extract_route_name(self, request: Request) -> str | None:
        """Extract a meaningful route name from the request."""
        try:
            if "route" in request.scope:
                route = request.scope["route"]
                if hasattr(route, "endpoint") and hasattr(route.endpoint, "__name__"):
                    return route.endpoint.__name__
                elif hasattr(route, "name") and route.name:
                    return route.name
        except (KeyError, AttributeError):
            pass
        return None

    def _extract_route_params(self, request: Request) -> dict:
        """Extract route parameters from path and query parameters."""
        params = {}

        # Add path parameters
        if hasattr(request, "path_params") and request.path_params:
            params.update(request.path_params)

        # Add query parameters
        if request.query_params:
            params.update(dict(request.query_params))

        return params
