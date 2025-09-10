# FinanceQuery API Architecture

## Overview

FinanceQuery is a FastAPI-based financial data API that aggregates data from Yahoo Finance API and web scraping. The architecture follows a service-oriented design with clear separation of concerns, emphasizing modularity, asynchronous operations, and resilience.

## Core Components

### FastAPI Application (`src/main.py`)

The main application handles:

- **Lifespan Management**: Initializes shared resources (HTTP sessions, Redis, authentication, proxy setup)
- **Middleware Stack**: CORS, request context, logging, and optional rate limiting
- **Error Handling**: Custom validation error formatting
- **Routing**: API endpoints under `/v1/` prefix with Pydantic validation
- **Health Checks**: `/ping` (basic) and `/health` (comprehensive service validation)
- **AWS Lambda Support**: Via Mangum adapter

### Data Models (`src/models/`)

Pydantic models provide:
- **API Contract Definition**: Consistent request/response structures
- **Input Validation**: Automatic parameter validation
- **Documentation Generation**: OpenAPI schema generation
- **Type Safety**: Runtime type checking and IDE support

### Routing System (`src/routes/`)

Modular routers organize endpoints by domain:
- `quotes` - Stock quotes and company information
- `historical_prices` - Historical price data
- `indices` - Market indices (S&P 500, NASDAQ, DOW)
- `sectors` - Industry sector performance
- `movers` - Market gainers/losers/most active
- `finance_news` - Financial news aggregation
- `search` - Symbol search functionality
- `hours` - Market hours status
- `similar` - Similar securities lookup
- `stream` - Server-sent events
- `sockets` - WebSocket connections

## Logging & Middleware Architecture

### Logging System (`src/utils/logging.py`)

Comprehensive logging with configurable levels and formats:

- **Environment Configuration**:
  - `LOG_LEVEL`: DEBUG/INFO/WARNING/ERROR/CRITICAL
  - `LOG_FORMAT`: JSON (production) or text (development)
  - `PERFORMANCE_THRESHOLD_MS`: Slow operation warning threshold (default: 2000ms)
  - `DISABLE_LOGO_FETCHING`: Disable logo fetching entirely (default: false)
  - `LOGO_TIMEOUT_SECONDS`: Logo request timeout (default: 1s)
  - `LOGO_CIRCUIT_BREAKER_THRESHOLD`: Failures before circuit breaker opens (default: 5)
  - `LOGO_CIRCUIT_BREAKER_TIMEOUT`: Circuit breaker timeout in seconds (default: 300)

- **Correlation Tracking**: Every request gets a unique correlation ID for distributed tracing
- **Structured Logging**: JSON format with contextual metadata
- **Performance Monitoring**: Automatic slow operation detection and logging
- **External Library Filtering**: Reduced noise from urllib3, curl_cffi, Redis

- **Specialized Logging Functions**:
  - `log_api_request/response()` - HTTP request/response logging
  - `log_route_request/success/error()` - Route-specific logging
  - `log_cache_operation()` - Cache hit/miss tracking
  - `log_external_api_call()` - Third-party API monitoring
  - `log_performance()` - Operation timing with threshold warnings
  - `log_critical_system_failure()` - System-level error handling

### Middleware Stack

#### LoggingMiddleware (`src/middleware/logging_middleware.py`)

- **Request Correlation**: Sets unique correlation ID for each request
- **Comprehensive Logging**: Logs all HTTP requests/responses with timing
- **Route-Aware Logging**: Enhanced logging for API routes vs. static content
- **Performance Tracking**: Automatic operation timing and slow request warnings
- **Error Context**: Detailed error logging with stack traces and request context
- **Response Headers**: Adds `X-Correlation-ID` header for client tracking

#### RateLimitMiddleware (`src/middleware/rate_limit_middleware.py`)

- **IP-Based Limiting**: Configurable daily request limits per client IP
- **API Key Bypass**: Admin keys bypass all rate limits
- **Open Paths**: Documentation and health endpoints exempt from limits
- **Health Check Throttling**: Special rate limiting for `/health` endpoint
- **Response Headers**: Rate limit status in `X-RateLimit-*` headers

## Service Layer Architecture

### Service Organization (`src/services/`)

Services implement business logic with a consistent pattern:

```
src/services/
├── quotes/
│   ├── fetchers/
│   │   ├── quote_api.py      # Primary Yahoo Finance API
│   │   └── quote_scraper.py  # Fallback web scraping
│   ├── get_quotes.py         # Main service with retry logic
│   └── utils.py             # Helper functions
├── indicators/
│   └── core/                # Cython-compiled calculations
└── ...
```

### Multi-Source Strategy

Each service implements dual-fetching for resilience:

1. **Primary Source** (Yahoo Finance API): Fast, reliable when available
2. **Fallback Source** (Web scraping): Resilient to API changes

The `@retry` decorator automatically falls back from API to scraping on failure.

### Technical Indicators (`src/services/indicators/core/`)

High-performance numerical computations using Cython:

- **Cython Modules**: `moving_averages.pyx`, `oscillators.pyx`, `trends.pyx`, `utils.pyx`
- **Build Requirement**: `python setup.py build_ext --inplace` compiles .pyx → .so files
- **Performance**: Significant speed improvements for mathematical calculations
- **NumPy Integration**: Optimized array operations with NumPy C API

## Development Workflow

### Package Management

- **Primary**: `uv` for fast dependency resolution and virtual environment management
- **Commands**: `uv sync --all-groups` for development setup
- **Fallback**: Traditional pip with requirements.txt files

### Build Process

Essential for technical indicators:
```bash
make build                              # Recommended
python setup.py build_ext --inplace   # Manual
```

### Development Commands

```bash
make help        # Show all commands
make install-dev # Install dependencies + pre-commit hooks
make serve       # Start development server
make test        # Run tests with coverage
make lint        # Run pre-commit hooks (ruff check/format)
make docs        # Serve documentation
make clean       # Clean build artifacts
```

## Data Persistence & Caching

### Caching Strategy (`src/utils/cache.py`)

Flexible caching with market-aware expiration:

- **Cache Handlers**:
  - `RedisCacheHandler`: Distributed caching (when Redis available)
  - `MemCacheHandler`: Local in-memory fallback

- **Smart Expiration**:
  - Standard TTL during market hours
  - Extended TTL when markets closed
  - Market schedule awareness via `MarketSchedule`

- **Cache Key Generation**: SHA-256 hash of function name + arguments
- **Type Preservation**: Maintains Pydantic model types after retrieval

### Retry Mechanism (`src/utils/retry.py`)

Sophisticated retry with fallback capability:

- **Configurable Retries**: Attempts primary function with exponential backoff
- **Intelligent Fallback**: Analyzes function signatures for parameter compatibility
- **Error Propagation**: Preserves HTTP exceptions while retrying network errors

## Authentication & HTTP Clients

### Yahoo Authentication (`src/utils/yahoo_auth.py`)

Manages Yahoo Finance API access:

- **Cookie/Crumb Management**: Handles CSRF tokens and session cookies
- **Automatic Refresh**: Maintains valid authentication credentials
- **Consent Flow Handling**: Navigates Yahoo's consent requirements
- **Thread Safety**: Async lock prevents concurrent refresh attempts

### HTTP Client Architecture

#### FetchClient (`src/clients/fetch_client.py`)

General-purpose async HTTP client:
- **curl_cffi Integration**: Browser impersonation for web scraping
- **Proxy Support**: Configurable proxy routing
- **Async/Await**: Non-blocking operations via `asyncio.to_thread`

#### YahooFinanceClient (`src/clients/yahoo_client.py`)

Specialized Yahoo Finance API client:
- **Auto-Authentication**: Injects cookies/crumb automatically
- **Error Handling**: Yahoo-specific HTTP status code handling
- **API Methods**: `get_quote()`, `get_chart()`, `search()`, etc.

## Real-Time Data Architecture

### WebSocket Connection Management

#### ConnectionManager (`src/connections/connection_manager.py`)

In-memory WebSocket management:
- **Channel-Based**: Groups connections by symbol/topic
- **Task Management**: Per-channel data fetching tasks
- **Auto-Cleanup**: Removes inactive connections and cancels unused tasks

#### RedisConnectionManager (`src/connections/redis_connection_manager.py`)

Distributed WebSocket support:
- **Redis Pub/Sub**: Multi-instance message broadcasting
- **Load Distribution**: Data fetching tasks distributed across instances
- **Scalability**: Supports horizontal scaling of WebSocket connections

## Security & Rate Limiting

### Security Configuration (`src/security/rate_limit_manager.py`)

- **Admin API Keys**: `ADMIN_API_KEY` bypasses all rate limits
- **Daily Limits**: Configurable requests per IP per day
- **Open Paths**: `/docs`, `/ping`, `/health` exempt from limits
- **Health Check Throttling**: Separate rate limiting for health checks
- **Security Toggle**: `USE_SECURITY` environment variable enables/disables rate limiting

### Implementation

- **In-Memory Storage**: Rate limit counters with automatic cleanup
- **IP-Based Tracking**: Client identification via request IP
- **Header Communication**: Rate limit status via response headers

## Dependency Injection (`src/utils/dependencies.py`)

FastAPI-injectable dependencies provide:

- **Shared Resources**: Session objects, Redis clients, authentication
- **Client Abstractions**: Pre-configured HTTP clients
- **Utility Functions**: Reusable async operations
- **Context Management**: Request-specific state and correlation

## Deployment Architecture

- **AWS Lambda**: Mangum adapter for serverless deployment
- **Docker**: Multi-stage builds with optimized images  
- **Environment Configuration**: 12-factor app principles with comprehensive environment variables
- **Health Monitoring**: Comprehensive health checks for all services
- **Logging Integration**: Structured logs for observability platforms

### Key Environment Variables

| Variable | Purpose | Default | Required | Docker Config |
|----------|---------|---------|----------|---------------|
| `REDIS_URL` | Redis connection for caching/WebSockets | None | No | Runtime |
| `USE_SECURITY` | Enable rate limiting and API authentication | False | No | Runtime |
| `ADMIN_API_KEY` | Admin key bypassing rate limits | None | No | Runtime |
| `USE_PROXY` | Enable proxy for web scraping | False | No | Runtime |
| `PROXY_URL` | Proxy server URL | None | No | Runtime |
| `PROXY_TOKEN` | Proxy authentication token | None | No | Runtime |
| `LOG_LEVEL` | Logging level | INFO | No | Build + Runtime |
| `LOG_FORMAT` | Log format (json/text) | json | No | Build + Runtime |
| `BYPASS_CACHE` | Disable caching | False | No | Runtime |
| `ALGOLIA_APP_ID` | Algolia search app ID | Public default | No | Runtime |
| `ALGOLIA_API_KEY` | Algolia search API key | Public default | No | Runtime |
| `DISABLE_LOGO_FETCHING` | Disable logo fetching | false | No | Build + Runtime |
| `LOGO_TIMEOUT_SECONDS` | Logo request timeout | 1 | No | Build + Runtime |
| `LOGO_CIRCUIT_BREAKER_THRESHOLD` | Circuit breaker failure threshold | 5 | No | Build + Runtime |
| `LOGO_CIRCUIT_BREAKER_TIMEOUT` | Circuit breaker timeout (seconds) | 300 | No | Build + Runtime |
| `PERFORMANCE_THRESHOLD_MS` | Slow operation warning threshold | 2000 | No | Build + Runtime |

### Docker Configuration

Both `Dockerfile` and `Dockerfile.aws` support all environment variables:

- **Build-time**: Logo fetching and logging configs can be baked into image
- **Runtime**: All variables can be overridden when running containers
- **Compose Ready**: All variables work with docker-compose configurations