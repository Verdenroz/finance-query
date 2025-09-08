import logging
import os
from io import StringIO
from unittest.mock import patch, MagicMock

from src.utils.logging import (
    CorrelationFilter,
    CustomJSONFormatter,
    configure_logging,
    get_logger,
    get_request_id,
    log_api_request,
    log_api_response,
    log_cache_operation,
    log_critical_system_failure,
    log_external_api_call,
    log_performance,
    log_route_error,
    log_route_request,
    log_route_success,
    request_id,
    set_request_id,
)


class TestCorrelationFilter:
    def test_filter_with_request_id(self):
        """Test filter adds correlation ID when request_id is set."""
        set_request_id("test-123")
        filter_instance = CorrelationFilter()
        record = logging.LogRecord(
            name="test", level=logging.INFO, pathname="", lineno=0, msg="test", args=(), exc_info=None
        )
        
        result = filter_instance.filter(record)
        
        assert result is True
        assert record.correlation_id == "test-123"

    def test_filter_without_request_id(self):
        """Test filter uses 'system' when no request_id is set."""
        # Clear any existing request_id
        request_id.set(None)
        filter_instance = CorrelationFilter()
        record = logging.LogRecord(
            name="test", level=logging.INFO, pathname="", lineno=0, msg="test", args=(), exc_info=None
        )
        
        result = filter_instance.filter(record)
        
        assert result is True
        assert record.correlation_id == "system"


class TestCustomJSONFormatter:
    def test_add_fields(self):
        """Test CustomJSONFormatter adds required fields."""
        formatter = CustomJSONFormatter()
        record = logging.LogRecord(
            name="test.module", level=logging.INFO, pathname="/path/to/module.py", lineno=0, msg="test message", args=(), exc_info=None
        )
        record.module = "module"  # Set module attribute explicitly
        record.correlation_id = "test-correlation"
        
        log_record = {}
        message_dict = {}
        
        formatter.add_fields(log_record, record, message_dict)
        
        assert log_record["level"] == "INFO"
        assert log_record["logger"] == "test.module"
        assert log_record["module"] == "module"
        assert log_record["correlation_id"] == "test-correlation"

    def test_add_fields_without_correlation_id(self):
        """Test CustomJSONFormatter without correlation_id attribute."""
        formatter = CustomJSONFormatter()
        record = logging.LogRecord(
            name="test.module", level=logging.ERROR, pathname="/path/to/module.py", lineno=0, msg="test message", args=(), exc_info=None
        )
        record.module = "module"  # Set module attribute explicitly
        
        log_record = {}
        message_dict = {}
        
        formatter.add_fields(log_record, record, message_dict)
        
        assert log_record["level"] == "ERROR"
        assert log_record["logger"] == "test.module"
        assert log_record["module"] == "module"
        assert "correlation_id" not in log_record


class TestConfigureLogging:
    @patch('src.utils.logging.logging.getLogger')
    @patch('src.utils.logging.sys.stdout')
    def test_configure_logging_json_format(self, mock_stdout, mock_get_logger):
        """Test configure_logging with JSON format."""
        mock_root_logger = MagicMock()
        mock_get_logger.return_value = mock_root_logger
        
        with patch.dict(os.environ, {"LOG_LEVEL": "DEBUG", "LOG_FORMAT": "json"}):
            configure_logging()
        
        assert mock_root_logger.setLevel.called
        assert mock_root_logger.handlers.clear.called
        assert mock_root_logger.addHandler.called

    @patch('src.utils.logging.logging.getLogger')
    @patch('src.utils.logging.sys.stdout')
    def test_configure_logging_text_format(self, mock_stdout, mock_get_logger):
        """Test configure_logging with text format."""
        mock_root_logger = MagicMock()
        mock_get_logger.return_value = mock_root_logger
        
        with patch.dict(os.environ, {"LOG_LEVEL": "INFO", "LOG_FORMAT": "text"}):
            configure_logging()
        
        assert mock_root_logger.setLevel.called
        assert mock_root_logger.handlers.clear.called
        assert mock_root_logger.addHandler.called

    @patch('src.utils.logging.logging.getLogger')
    @patch('src.utils.logging.sys.stdout')
    @patch('builtins.print')
    def test_configure_logging_invalid_level(self, mock_print, mock_stdout, mock_get_logger):
        """Test configure_logging with invalid log level."""
        mock_root_logger = MagicMock()
        mock_get_logger.return_value = mock_root_logger
        
        with patch.dict(os.environ, {"LOG_LEVEL": "INVALID", "LOG_FORMAT": "json"}):
            configure_logging()
        
        mock_print.assert_called_with("Warning: Invalid log level 'INVALID', defaulting to INFO")

    @patch('src.utils.logging.logging.getLogger')
    @patch('src.utils.logging.sys.stdout')
    def test_configure_logging_external_libs_debug(self, mock_stdout, mock_get_logger):
        """Test external library logging configuration when DEBUG level."""
        mock_root_logger = MagicMock()
        mock_urllib3_logger = MagicMock()
        mock_curl_logger = MagicMock()
        mock_redis_logger = MagicMock()
        mock_uvicorn_access_logger = MagicMock()
        mock_uvicorn_error_logger = MagicMock()
        
        def get_logger_side_effect(name=None):
            if name == "" or name is None:
                return mock_root_logger
            elif name == "urllib3":
                return mock_urllib3_logger
            elif name == "curl_cffi":
                return mock_curl_logger
            elif name == "redis":
                return mock_redis_logger
            elif name == "uvicorn.access":
                return mock_uvicorn_access_logger
            elif name == "uvicorn.error":
                return mock_uvicorn_error_logger
            return MagicMock()
        
        mock_get_logger.side_effect = get_logger_side_effect
        
        with patch.dict(os.environ, {"LOG_LEVEL": "DEBUG", "LOG_FORMAT": "json"}):
            configure_logging()
        
        # External libraries should NOT be set to WARNING when DEBUG level
        mock_urllib3_logger.setLevel.assert_not_called()
        mock_curl_logger.setLevel.assert_not_called()
        mock_redis_logger.setLevel.assert_not_called()

    @patch('src.utils.logging.logging.getLogger')
    @patch('src.utils.logging.sys.stdout')
    def test_configure_logging_external_libs_info(self, mock_stdout, mock_get_logger):
        """Test external library logging configuration when INFO level."""
        mock_root_logger = MagicMock()
        mock_urllib3_logger = MagicMock()
        mock_curl_logger = MagicMock()
        mock_redis_logger = MagicMock()
        mock_uvicorn_access_logger = MagicMock()
        mock_uvicorn_error_logger = MagicMock()
        
        def get_logger_side_effect(name=None):
            if name == "" or name is None:
                return mock_root_logger
            elif name == "urllib3":
                return mock_urllib3_logger
            elif name == "curl_cffi":
                return mock_curl_logger
            elif name == "redis":
                return mock_redis_logger
            elif name == "uvicorn.access":
                return mock_uvicorn_access_logger
            elif name == "uvicorn.error":
                return mock_uvicorn_error_logger
            return MagicMock()
        
        mock_get_logger.side_effect = get_logger_side_effect
        
        with patch.dict(os.environ, {"LOG_LEVEL": "INFO", "LOG_FORMAT": "json"}):
            configure_logging()
        
        # External libraries should be set to WARNING when INFO level
        mock_urllib3_logger.setLevel.assert_called_with(logging.WARNING)
        mock_curl_logger.setLevel.assert_called_with(logging.WARNING)
        mock_redis_logger.setLevel.assert_called_with(logging.WARNING)


class TestUtilityFunctions:
    def test_get_logger(self):
        """Test get_logger returns a logger instance."""
        logger = get_logger("test.module")
        assert isinstance(logger, logging.Logger)
        assert logger.name == "test.module"

    def test_set_request_id_with_id(self):
        """Test set_request_id with provided ID."""
        correlation_id = set_request_id("custom-id")
        assert correlation_id == "custom-id"
        assert get_request_id() == "custom-id"

    def test_set_request_id_without_id(self):
        """Test set_request_id generates UUID when no ID provided."""
        correlation_id = set_request_id()
        assert correlation_id is not None
        assert len(correlation_id) == 8  # UUID[:8]
        assert get_request_id() == correlation_id

    def test_get_request_id_none(self):
        """Test get_request_id returns None when no ID set."""
        request_id.set(None)
        assert get_request_id() is None


class TestLogPerformance:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.performance")
        self.logger.handlers = []  # Clear handlers
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_performance_fast_operation(self):
        """Test log_performance for fast operation (debug level)."""
        log_performance(self.logger, "test_operation", 100.5)
        
        output = self.stream.getvalue()
        assert "Operation completed - test_operation (100.5ms)" in output

    def test_log_performance_slow_operation_default_threshold(self):
        """Test log_performance for slow operation with default threshold."""
        log_performance(self.logger, "slow_operation", 3000.0)
        
        output = self.stream.getvalue()
        assert "Slow operation detected - slow_operation (3000.0ms)" in output

    def test_log_performance_slow_operation_custom_threshold(self):
        """Test log_performance for slow operation with custom threshold."""
        log_performance(self.logger, "slow_operation", 1500.0, threshold_ms=1000.0)
        
        output = self.stream.getvalue()
        assert "Slow operation detected - slow_operation (1500.0ms)" in output

    @patch.dict(os.environ, {"PERFORMANCE_THRESHOLD_MS": "5000"})
    def test_log_performance_env_threshold(self):
        """Test log_performance uses environment variable threshold."""
        log_performance(self.logger, "test_operation", 3000.0)
        
        output = self.stream.getvalue()
        assert "Operation completed - test_operation (3000.0ms)" in output


class TestAPILogging:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.api")
        self.logger.handlers = []
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_api_request(self):
        """Test log_api_request."""
        log_api_request(self.logger, "GET", "/api/test", {"param": "value"})
        
        output = self.stream.getvalue()
        assert "API request received" in output

    def test_log_api_request_no_params(self):
        """Test log_api_request without query parameters."""
        log_api_request(self.logger, "POST", "/api/test")
        
        output = self.stream.getvalue()
        assert "API request received" in output

    def test_log_api_response_success(self):
        """Test log_api_response for successful response."""
        log_api_response(self.logger, "GET", "/api/test", 200, 150.5)
        
        output = self.stream.getvalue()
        assert "API response sent - GET /api/test (200) [150.5ms]" in output

    def test_log_api_response_error(self):
        """Test log_api_response for error response."""
        log_api_response(self.logger, "GET", "/api/test", 500, 250.0)
        
        output = self.stream.getvalue()
        assert "API response sent - GET /api/test (500) [250.0ms]" in output


class TestCacheLogging:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.cache")
        self.logger.handlers = []
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_cache_operation_hit(self):
        """Test log_cache_operation for cache hit."""
        log_cache_operation(self.logger, "get", "test_key", hit=True)
        
        output = self.stream.getvalue()
        assert "Cache HIT - Data retrieved from cache" in output

    def test_log_cache_operation_miss(self):
        """Test log_cache_operation for cache miss."""
        log_cache_operation(self.logger, "get", "test_key", hit=False)
        
        output = self.stream.getvalue()
        assert "Cache MISS - Data not found in cache" in output

    def test_log_cache_operation_get_no_hit(self):
        """Test log_cache_operation for get operation without hit specified."""
        log_cache_operation(self.logger, "get", "test_key")
        
        output = self.stream.getvalue()
        assert "Cache GET operation" in output

    def test_log_cache_operation_set(self):
        """Test log_cache_operation for set operation."""
        log_cache_operation(self.logger, "set", "test_key")
        
        output = self.stream.getvalue()
        assert "Cache SET - Data stored in cache" in output

    def test_log_cache_operation_other(self):
        """Test log_cache_operation for other operations."""
        log_cache_operation(self.logger, "delete", "test_key")
        
        output = self.stream.getvalue()
        assert "Cache DELETE operation" in output

    def test_log_cache_operation_set_with_hit(self):
        """Test log_cache_operation for set operation with hit parameter."""
        log_cache_operation(self.logger, "set", "test_key", hit=True)
        
        output = self.stream.getvalue()
        assert "Cache SET - Data stored in cache" in output


class TestExternalAPILogging:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.external")
        self.logger.handlers = []
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_external_api_call_success(self):
        """Test log_external_api_call for successful call."""
        log_external_api_call(self.logger, "yahoo", "/api/quotes", 500.0, success=True)
        
        output = self.stream.getvalue()
        assert "External API SUCCESS - yahoo /api/quotes (500.0ms)" in output

    def test_log_external_api_call_failure(self):
        """Test log_external_api_call for failed call."""
        log_external_api_call(self.logger, "yahoo", "/api/quotes", 1000.0, success=False)
        
        output = self.stream.getvalue()
        assert "External API FAILED - yahoo /api/quotes (1000.0ms)" in output

    def test_log_external_api_call_default_success(self):
        """Test log_external_api_call with default success parameter."""
        log_external_api_call(self.logger, "yahoo", "/api/quotes", 300.0)
        
        output = self.stream.getvalue()
        assert "External API SUCCESS - yahoo /api/quotes (300.0ms)" in output


class TestRouteLogging:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.route")
        self.logger.handlers = []
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_route_request(self):
        """Test log_route_request."""
        log_route_request(self.logger, "get_quotes", {"symbol": "AAPL"})
        
        output = self.stream.getvalue()
        assert "Processing get_quotes request" in output

    def test_log_route_success(self):
        """Test log_route_success without result info."""
        log_route_success(self.logger, "get_quotes", {"symbol": "AAPL"})
        
        output = self.stream.getvalue()
        assert "get_quotes request completed successfully" in output

    def test_log_route_success_with_result_info(self):
        """Test log_route_success with result info."""
        log_route_success(self.logger, "get_quotes", {"symbol": "AAPL"}, {"count": 1})
        
        output = self.stream.getvalue()
        assert "get_quotes request completed successfully" in output

    def test_log_route_error(self):
        """Test log_route_error."""
        error = ValueError("Test error")
        log_route_error(self.logger, "get_quotes", {"symbol": "INVALID"}, error)
        
        output = self.stream.getvalue()
        assert "get_quotes request failed" in output


class TestCriticalLogging:
    def setup_method(self):
        """Setup test logger."""
        self.logger = logging.getLogger("test.critical")
        self.logger.handlers = []
        self.stream = StringIO()
        handler = logging.StreamHandler(self.stream)
        handler.setLevel(logging.DEBUG)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.DEBUG)

    def test_log_critical_system_failure(self):
        """Test log_critical_system_failure without context."""
        error = Exception("System failure")
        log_critical_system_failure(self.logger, "database_connection", error)
        
        output = self.stream.getvalue()
        assert "CRITICAL SYSTEM FAILURE: database_connection" in output

    def test_log_critical_system_failure_with_context(self):
        """Test log_critical_system_failure with context."""
        error = Exception("System failure")
        context = {"host": "localhost", "port": 5432}
        log_critical_system_failure(self.logger, "database_connection", error, context)
        
        output = self.stream.getvalue()
        assert "CRITICAL SYSTEM FAILURE: database_connection" in output