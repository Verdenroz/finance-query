import time
from collections.abc import Callable

from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware

from src.utils.logging import get_logger, log_api_request, log_api_response, log_critical_system_failure, log_performance, set_request_id

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

        # Log incoming request
        log_api_request(logger, method, path, query_params)

        # Process request and measure time
        start_time = time.perf_counter()
        try:
            response = await call_next(request)
            duration_ms = (time.perf_counter() - start_time) * 1000

            # Log performance metrics
            operation = f"{method} {path}"
            log_performance(logger, operation, duration_ms)

            # Log response
            log_api_response(logger, method, path, response.status_code, duration_ms)

            # Add correlation ID to response headers
            response.headers["X-Correlation-ID"] = correlation_id

            return response

        except Exception as e:
            duration_ms = (time.perf_counter() - start_time) * 1000

            # Log performance even for failed requests
            operation = f"{method} {path}"
            log_performance(logger, operation, duration_ms)

            # Check if this is a system-level critical failure
            if isinstance(e, MemoryError | SystemExit | KeyboardInterrupt):
                log_critical_system_failure(logger, "request_processing", e, {"method": method, "path": path, "duration_ms": duration_ms})
            else:
                logger.error(
                    "Request failed with exception",
                    extra={"method": method, "path": path, "duration_ms": duration_ms, "error": str(e), "error_type": type(e).__name__},
                    exc_info=True,
                )
            raise
