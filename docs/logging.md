# Logging Configuration

FinanceQuery includes a comprehensive logging system designed for both development and production environments. The logging system provides structured output, performance monitoring, request tracing, and external API tracking.

## Overview

The logging system features:

- **Configurable log levels** (DEBUG, INFO, WARNING, ERROR, CRITICAL)
- **Multiple output formats** (JSON for production, text for development)  
- **Performance monitoring** with configurable thresholds
- **Request correlation IDs** for distributed tracing
- **External API tracking** with timing and success metrics
- **Cache operation logging** for performance optimization
- **Automatic middleware** for HTTP request/response logging with route-specific tracking
- **Response correlation headers** (`X-Correlation-ID`) for client tracing

## Environment Variables

### LOG_LEVEL

Controls the verbosity of log output.

```env
LOG_LEVEL=INFO
```

**Available Options:**

| Level | Description | Use Case |
|-------|-------------|----------|
| `DEBUG` | Detailed diagnostic information | Development, troubleshooting |
| `INFO` | General operational messages | Production (default) |
| `WARNING` | Warning messages and slow operations | Production monitoring |
| `ERROR` | Error conditions | Always enabled |
| `CRITICAL` | System failures requiring immediate attention | Always enabled |

### LOG_FORMAT

Determines the output format of log messages.

```env
LOG_FORMAT=json
```

**Available Options:**

=== "JSON Format (`json`)"
    ```json
    {
      "asctime": "2025-01-15 14:30:22",
      "level": "INFO",
      "logger": "src.routes.quotes",
      "module": "quotes",
      "correlation_id": "a1b2c3d4",
      "message": "Processing quotes request",
      "route": "quotes",
      "params": {"symbols": ["AAPL", "TSLA"]}
    }
    ```
    
    **Benefits:**
    
    - Structured data for log analysis tools
    - Easy parsing by monitoring systems
    - Consistent field extraction
    - Machine-readable format

=== "Text Format (`text`)"
    ```
    2025-01-15 14:30:22 - src.routes.quotes - INFO - [a1b2c3d4] - Processing quotes request
    ```
    
    **Benefits:**
    
    - Human-readable format
    - Easier debugging during development
    - Familiar log format
    - Compact output

### PERFORMANCE_THRESHOLD_MS

Sets the threshold (in milliseconds) for slow operation warnings.

```env
PERFORMANCE_THRESHOLD_MS=2000
```

**Recommended Values:**

| Environment | Threshold | Reasoning |
|-------------|-----------|-----------|
| Development | `500ms` | Strict performance monitoring for optimization |
| Staging | `1000ms` | Moderate monitoring for pre-production testing |
| Production | `2000ms` | Balanced monitoring (default) |
| High-Load | `5000ms` | Relaxed monitoring for systems under heavy load |

## Logging Features

### Request Correlation IDs

Every request gets a unique correlation ID for tracing across the entire request lifecycle:

```
2025-01-15 14:30:22 - src.middleware.logging_middleware - INFO - [a1b2c3d4] - API request received
2025-01-15 14:30:22 - src.middleware.logging_middleware - INFO - [a1b2c3d4] - Processing quotes request  
2025-01-15 14:30:22 - src.clients.yahoo_client - INFO - [a1b2c3d4] - External API SUCCESS - Yahoo Finance quote (234.1ms)
2025-01-15 14:30:22 - src.middleware.logging_middleware - INFO - [a1b2c3d4] - quotes request completed successfully
2025-01-15 14:30:22 - src.middleware.logging_middleware - INFO - [a1b2c3d4] - API response sent - GET /quotes (200) [347.8ms]
```

The correlation ID is also added to HTTP response headers as `X-Correlation-ID`, allowing clients to correlate their requests with server logs.

### Automatic Route Logging

The `LoggingMiddleware` automatically handles logging for API routes without requiring manual logging calls in route handlers:

- **General HTTP logging**: All requests/responses with timing and status codes
- **Route-specific logging**: Enhanced logging for `/v1/` API endpoints with route names and parameters
- **Error handling**: Automatic error logging with stack traces and context
- **Performance tracking**: Slow operation warnings based on configurable thresholds

Route handlers are kept clean and focused on business logic, while the middleware handles all logging concerns centrally.

### Performance Monitoring

Automatic performance logging for all HTTP requests:

```
# Fast operation (DEBUG level)
Operation completed - GET /quotes (347.8ms)

# Slow operation (WARNING level) 
Slow operation detected - GET /historical (2134.7ms)
```

### External API Tracking

Monitor all external service calls with timing and success metrics:

```
External API SUCCESS - Yahoo Finance quote (234.1ms)
External API SUCCESS - Algolia search (89.3ms)
External API FAILED - Logo.dev ticker (5002.4ms)
```

### Cache Operations

Track cache performance for optimization:

```
Cache HIT - Data retrieved from cache
Cache MISS - Data not found in cache  
Cache SET - Data stored in cache
```

## Configuration Examples

### Development Setup

```env
LOG_LEVEL=DEBUG
LOG_FORMAT=text
PERFORMANCE_THRESHOLD_MS=500
```

**Output:**
```
2025-01-15 14:30:22 - src.routes.quotes - DEBUG - [a1b2c3d4] - Operation completed - GET /quotes (123.4ms)
2025-01-15 14:30:22 - src.utils.cache - DEBUG - [a1b2c3d4] - Cache HIT - Data retrieved from cache
```

### Production Setup

```env
LOG_LEVEL=INFO
LOG_FORMAT=json
PERFORMANCE_THRESHOLD_MS=2000
```

**Output:**
```json
{
  "asctime": "2025-01-15 14:30:22",
  "level": "INFO", 
  "logger": "src.middleware.logging_middleware",
  "correlation_id": "a1b2c3d4",
  "message": "API response sent - GET /quotes (200) [347.8ms]",
  "method": "GET",
  "path": "/quotes",
  "status_code": 200,
  "duration_ms": 347.8,
  "api_response": true
}
```

### Docker Configuration

=== "Build Time"
    ```bash
    docker build \
      --build-arg LOG_LEVEL=INFO \
      --build-arg LOG_FORMAT=json \
      --build-arg PERFORMANCE_THRESHOLD_MS=2000 \
      -t financequery .
    ```

=== "Runtime"
    ```bash
    docker run \
      -e LOG_LEVEL=WARNING \
      -e LOG_FORMAT=json \
      -e PERFORMANCE_THRESHOLD_MS=5000 \
      financequery
    ```

## Monitoring Integration

### ELK Stack (Elasticsearch, Logstash, Kibana)

With `LOG_FORMAT=json`, logs can be easily ingested by Logstash:

```ruby
# logstash.conf
input {
  docker {
    type => "financequery"
  }
}

filter {
  if [type] == "financequery" {
    json {
      source => "message"
    }
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
  }
}
```

### Prometheus Integration

Create custom metrics from structured logs:

```yaml
# prometheus.yml
- job_name: 'financequery-logs'
  static_configs:
    - targets: ['financequery:8000']
  metrics_path: '/metrics'
```

### CloudWatch Logs

For AWS deployments, structured JSON logs integrate seamlessly:

```json
{
  "logGroup": "/aws/lambda/financequery",
  "logStream": "2025/01/15/[$LATEST]abc123",
  "timestamp": 1642263022000,
  "message": "{\"level\":\"INFO\",\"correlation_id\":\"a1b2c3d4\",\"duration_ms\":347.8}"
}
```

## Troubleshooting

### Common Issues

#### High Log Volume

**Problem:** Too many DEBUG logs in production.
**Solution:** Set `LOG_LEVEL=INFO` or higher.

```env
LOG_LEVEL=INFO  # Reduces log volume
```

#### Missing Performance Warnings

**Problem:** Not seeing slow operation warnings.
**Solution:** Lower the performance threshold.

```env
PERFORMANCE_THRESHOLD_MS=1000  # More sensitive monitoring
```

#### Parsing Issues with Log Aggregators

**Problem:** Logs not parsing correctly in monitoring systems.
**Solution:** Ensure JSON format is enabled.

```env
LOG_FORMAT=json  # Structured output
```

### Log Filtering

#### View Only Errors

```bash
# Text format
grep "ERROR\|CRITICAL" app.log

# JSON format  
jq 'select(.level == "ERROR" or .level == "CRITICAL")' app.log
```

#### Monitor Performance Issues

```bash
# Text format
grep "Slow operation detected" app.log

# JSON format
jq 'select(.message | contains("Slow operation"))' app.log
```

#### Track External API Issues

```bash
# Text format  
grep "External API FAILED" app.log

# JSON format
jq 'select(.message | contains("External API FAILED"))' app.log
```

## Best Practices

### Development

- Use `LOG_LEVEL=DEBUG` for detailed diagnostics
- Use `LOG_FORMAT=text` for human-readable output
- Set low `PERFORMANCE_THRESHOLD_MS=500` for optimization

### Production

- Use `LOG_LEVEL=INFO` to balance detail and volume
- Use `LOG_FORMAT=json` for structured monitoring
- Set appropriate `PERFORMANCE_THRESHOLD_MS` based on expected load
- Monitor correlation IDs for request tracing
- Set up log rotation to prevent disk space issues

### Monitoring

- Create alerts on ERROR and CRITICAL log levels
- Monitor slow operation trends
- Track external API failure rates
- Set up dashboards for performance metrics
- Use correlation IDs for distributed tracing

## Log Rotation

For production deployments, configure log rotation to manage disk space:

```bash
# logrotate configuration
/var/log/financequery/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 financequery financequery
}
```
