import logging
import logging.config
import os
import sys
import uuid
from contextvars import ContextVar
from typing import Any, Dict, Optional

from pythonjsonlogger.json import JsonFormatter

# Context variable to store request correlation ID
request_id: ContextVar[Optional[str]] = ContextVar("request_id", default=None)


class CorrelationFilter(logging.Filter):
    """Add correlation ID to log records."""
    
    def filter(self, record: logging.LogRecord) -> bool:
        record.correlation_id = request_id.get() or "system"
        return True


class CustomJSONFormatter(JsonFormatter):
    """Custom JSON formatter with additional context."""
    
    def add_fields(self, log_record: Dict[str, Any], record: logging.LogRecord, message_dict: Dict[str, Any]) -> None:
        super().add_fields(log_record, record, message_dict)
        log_record["level"] = record.levelname
        log_record["logger"] = record.name
        log_record["module"] = record.module
        if hasattr(record, "correlation_id"):
            log_record["correlation_id"] = record.correlation_id


def configure_logging() -> None:
    """Configure logging based on environment variables."""
    log_level = os.getenv("LOG_LEVEL", "INFO").upper()
    log_format = os.getenv("LOG_FORMAT", "json").lower()
    
    # Validate log level
    numeric_level = getattr(logging, log_level, None)
    if not isinstance(numeric_level, int):
        numeric_level = logging.INFO
        print(f"Warning: Invalid log level '{log_level}', defaulting to INFO")
    
    # Configure formatters
    if log_format == "json":
        formatter = CustomJSONFormatter(
            "%(asctime)s %(level)s %(logger)s %(module)s %(correlation_id)s %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )
    else:
        formatter = logging.Formatter(
            "%(asctime)s - %(name)s - %(levelname)s - [%(correlation_id)s] - %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )
    
    # Create console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(numeric_level)
    console_handler.setFormatter(formatter)
    console_handler.addFilter(CorrelationFilter())
    
    # Configure root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(numeric_level)
    
    # Clear existing handlers
    root_logger.handlers.clear()
    root_logger.addHandler(console_handler)
    
    # Set specific logger levels
    logging.getLogger("uvicorn.access").setLevel(logging.WARNING)
    logging.getLogger("uvicorn.error").setLevel(logging.INFO)
    
    # Disable debug logging for external libraries unless specifically requested
    if numeric_level > logging.DEBUG:
        logging.getLogger("urllib3").setLevel(logging.WARNING)
        logging.getLogger("curl_cffi").setLevel(logging.WARNING)
        logging.getLogger("redis").setLevel(logging.WARNING)


def get_logger(name: str) -> logging.Logger:
    """Get a configured logger instance."""
    return logging.getLogger(name)


def set_request_id(correlation_id: Optional[str] = None) -> str:
    """Set correlation ID for the current request context."""
    if correlation_id is None:
        correlation_id = str(uuid.uuid4())[:8]
    request_id.set(correlation_id)
    return correlation_id


def get_request_id() -> Optional[str]:
    """Get the current request correlation ID."""
    return request_id.get()


def log_performance(logger: logging.Logger, operation: str, duration_ms: float, threshold_ms: float = 2000) -> None:
    """Log performance metrics, with warnings for slow operations."""
    if duration_ms > threshold_ms:
        logger.warning(
            "Slow operation detected",
            extra={
                "operation": operation,
                "duration_ms": duration_ms,
                "threshold_ms": threshold_ms
            }
        )
    else:
        logger.debug(
            "Operation completed",
            extra={
                "operation": operation,
                "duration_ms": duration_ms
            }
        )


def log_api_request(logger: logging.Logger, method: str, path: str, query_params: Optional[Dict[str, Any]] = None) -> None:
    """Log incoming API request."""
    logger.info(
        "API request received",
        extra={
            "method": method,
            "path": path,
            "query_params": query_params or {}
        }
    )


def log_api_response(logger: logging.Logger, method: str, path: str, status_code: int, duration_ms: float) -> None:
    """Log API response."""
    log_level = logging.INFO if status_code < 400 else logging.ERROR
    logger.log(
        log_level,
        "API response sent",
        extra={
            "method": method,
            "path": path,
            "status_code": status_code,
            "duration_ms": duration_ms
        }
    )


def log_cache_operation(logger: logging.Logger, operation: str, key: str, hit: Optional[bool] = None) -> None:
    """Log cache operations."""
    extra = {"operation": operation, "cache_key": key}
    
    if operation == "get":
        if hit is True:
            message = "Cache HIT - Data retrieved from cache"
            extra["cache_hit"] = True
        elif hit is False:
            message = "Cache MISS - Data not found in cache"
            extra["cache_hit"] = False
        else:
            message = "Cache GET operation"
    elif operation == "set":
        message = "Cache SET - Data stored in cache"
    else:
        message = f"Cache {operation.upper()} operation"
    
    if hit is not None and operation != "get":
        extra["cache_hit"] = hit
    
    logger.debug(message, extra=extra)


def log_external_api_call(logger: logging.Logger, service: str, endpoint: str, duration_ms: float, success: bool = True) -> None:
    """Log external API calls."""
    log_level = logging.INFO if success else logging.ERROR
    logger.log(
        log_level,
        "External API call",
        extra={
            "service": service,
            "endpoint": endpoint,
            "duration_ms": duration_ms,
            "success": success
        }
    )


def log_route_request(logger: logging.Logger, route_name: str, params: dict) -> None:
    """Log incoming route request."""
    logger.info(f"Processing {route_name} request", extra={"route": route_name, "params": params})


def log_route_success(logger: logging.Logger, route_name: str, params: dict, result_info: dict = None) -> None:
    """Log successful route response."""
    extra = {"route": route_name, "params": params}
    if result_info:
        extra.update(result_info)
    logger.info(f"{route_name} request completed successfully", extra=extra)


def log_route_error(logger: logging.Logger, route_name: str, params: dict, error: Exception) -> None:
    """Log route error."""
    logger.error(
        f"{route_name} request failed",
        extra={
            "route": route_name,
            "params": params,
            "error": str(error),
            "error_type": type(error).__name__
        },
        exc_info=True
    )


def log_critical_system_failure(logger: logging.Logger, operation: str, error: Exception, context: dict = None) -> None:
    """Log critical system failures that indicate the application cannot continue."""
    extra = {
        "operation": operation,
        "error": str(error),
        "error_type": type(error).__name__
    }
    if context:
        extra.update(context)
    
    logger.critical(
        f"CRITICAL SYSTEM FAILURE: {operation}",
        extra=extra,
        exc_info=True
    )