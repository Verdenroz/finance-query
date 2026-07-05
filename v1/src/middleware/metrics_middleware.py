import time
from collections.abc import Callable

from fastapi import Request, Response
from prometheus_client import Counter, Gauge, Histogram
from starlette.middleware.base import BaseHTTPMiddleware

from src.utils.logging import get_logger

logger = get_logger(__name__)

METRICS_PATH = "/v1/metrics"

# Label for requests that did not match any route; single bucket to avoid
# cardinality explosion from scanners probing arbitrary paths.
UNMATCHED_ENDPOINT = "unmatched"

DURATION_BUCKETS = (0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0)

HTTP_REQUESTS_TOTAL = Counter(
    "finance_query_v1_http_requests_total",
    "Total HTTP requests processed by the v1 API.",
    ["method", "endpoint", "status"],
)

HTTP_REQUEST_DURATION_SECONDS = Histogram(
    "finance_query_v1_http_request_duration_seconds",
    "HTTP request duration in seconds for the v1 API.",
    ["method", "endpoint"],
    buckets=DURATION_BUCKETS,
)

HTTP_REQUESTS_IN_FLIGHT = Gauge(
    "finance_query_v1_http_requests_in_flight",
    "Number of HTTP requests currently being processed by the v1 API.",
)

REDIS_CONNECTED = Gauge(
    "finance_query_v1_redis_connected",
    "Whether the v1 API holds a live Redis connection (1) or runs in-memory (0).",
)

REDIS_ERRORS_TOTAL = Counter(
    "finance_query_v1_redis_errors_total",
    "Total Redis errors observed by the v1 API.",
)


def record_redis_connected(connected: bool) -> None:
    """Set the Redis connection gauge. Never raises (passive instrumentation)."""
    try:
        REDIS_CONNECTED.set(1 if connected else 0)
    except Exception:
        logger.debug("Failed to record Redis connection gauge", exc_info=True)


def record_redis_error() -> None:
    """Increment the Redis error counter. Never raises (passive instrumentation)."""
    try:
        REDIS_ERRORS_TOTAL.inc()
    except Exception:
        logger.debug("Failed to record Redis error counter", exc_info=True)


def _endpoint_label(request: Request) -> str:
    """Resolve the route template (e.g. "/v1/quotes") to keep label cardinality bounded."""
    # FastAPI >=0.136 nests included routers, so scope["route"].path lacks the
    # router prefix; the effective route context carries the full template.
    fastapi_scope = request.scope.get("fastapi")
    if isinstance(fastapi_scope, dict):
        context = fastapi_scope.get("effective_route_context")
        path = getattr(context, "path_format", None)
        if isinstance(path, str):
            return path
    route = request.scope.get("route")
    path = getattr(route, "path", None)
    return path if isinstance(path, str) else UNMATCHED_ENDPOINT


def _observe_in_flight(delta: int) -> None:
    try:
        HTTP_REQUESTS_IN_FLIGHT.inc(delta)
    except Exception:
        logger.debug("Failed to record in-flight gauge", exc_info=True)


def _observe_request(request: Request, status: int, duration_seconds: float) -> None:
    try:
        method = request.method
        endpoint = _endpoint_label(request)
        HTTP_REQUESTS_TOTAL.labels(method=method, endpoint=endpoint, status=str(status)).inc()
        HTTP_REQUEST_DURATION_SECONDS.labels(method=method, endpoint=endpoint).observe(duration_seconds)
    except Exception:
        logger.debug("Failed to record request metrics", exc_info=True)


class MetricsMiddleware(BaseHTTPMiddleware):
    """
    Passive Prometheus instrumentation for HTTP requests.

    Metric recording is fully wrapped so it can never raise into the request
    path; exceptions from the application itself are recorded as status 500
    and re-raised untouched. Non-HTTP scopes (websockets, lifespan) bypass
    dispatch entirely via BaseHTTPMiddleware.
    """

    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        # Scraping the metrics endpoint should not observe itself.
        if request.url.path == METRICS_PATH:
            return await call_next(request)

        _observe_in_flight(1)
        start_time = time.perf_counter()
        try:
            response = await call_next(request)
        except Exception:
            _observe_request(request, 500, time.perf_counter() - start_time)
            raise
        finally:
            _observe_in_flight(-1)

        _observe_request(request, response.status_code, time.perf_counter() - start_time)
        return response
