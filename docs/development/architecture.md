# FinanceQuery API Architecture

## 1. Overview

FinanceQuery is a FastAPI-based application designed to provide financial data by scraping web sources and interacting with Yahoo Finance's unofficial API. It offers RESTful endpoints for various financial data points and WebSocket support for real-time data streaming. The architecture emphasizes modularity, asynchronous operations, and resilience.

## 2. Core Components

### 2.1. FastAPI Application (`main.py`)

The core of the application is a FastAPI instance.
- **Initialization**: Sets up metadata like title, version, description, server URLs, contact, and license information.

- **Lifespan Management**: A `lifespan` asynchronous context manager handles the initialization and cleanup of shared resources:
    - `fastapi_injectable` registration.
    - Shared `curl_cffi.requests.Session` for HTTP requests.
    - `YahooAuthManager` instance for managing Yahoo Finance authentication.
    - Optional proxy setup (`PROXY_URL`, `USE_PROXY`).
    - Optional Redis client (`REDIS_URL`) for caching and WebSocket message brokering.
    - WebSocket `ConnectionManager` (defaults to in-memory, or `RedisConnectionManager` if Redis is enabled).
    - Primes Yahoo authentication on startup for faster initial user requests.
    - Ensures cleanup of resources (exit stacks, connection manager, proxy whitelist, Redis connection) on shutdown.

- **Middleware**:
    - `CORSMiddleware`: Configured to allow all origins, methods, and headers, and exposes rate limit headers.
    - `RequestContextMiddleware`: Makes the `Request` or `WebSocket` object accessible throughout the application via a context variable. This is crucial for dependency injection and accessing request-specific state.
    - `RateLimitMiddleware`: (Optional, enabled by `USE_SECURITY` env var) Provides rate limiting capabilities.
  
- **Error Handling**:
    - A custom exception handler for `RequestValidationError` formats Pydantic validation errors into a more user-friendly JSON structure.
  
- **Routing**:
    - Includes various routers for different API functionalities (e.g., quotes, historical prices, news, WebSockets). Routers are typically prefixed with `/v1`. Each router groups related endpoints and utilizes Pydantic models for request and response validation and serialization.
  
- **Deployment**:
    - `Mangum` adapter is used to allow deployment as an AWS Lambda function.

### 2.2. Health Checks (`main.py`)

- **`/ping`**: A simple endpoint to check if the server is alive and responding. Returns status and current timestamp.
- **`/health`**: A detailed health check endpoint that verifies:
    - Basic API health.
    - Redis connectivity and latency (if configured).
    - Status of various internal services by making sample calls (e.g., fetching indices, market movers, news, quotes). Reports success/failure for each service.

### 2.3. Models and Routers

#### Pydantic Models (`src/models`)

Pydantic models are extensively used throughout the application, primarily defined in `src/models.py` (or similar modules within a `models` directory). Their roles include:

-   **Data Validation**: Defining the expected structure, data types, and validation rules for incoming request data (path parameters, query parameters, request bodies). FastAPI automatically uses these models to validate incoming data.
-   **Data Serialization**: Defining the structure and data types for outgoing API responses. FastAPI uses these models to serialize response data into JSON.
-   **API Contract**: Serving as a clear and enforceable contract for API inputs and outputs.
-   **OpenAPI Schema Generation**: FastAPI leverages these Pydantic models to automatically generate the OpenAPI (Swagger) documentation, providing interactive API docs.
-   **Error Reporting**: When validation fails, FastAPI (in conjunction with the custom `RequestValidationError` handler) uses the model definitions to provide detailed error messages.

#### Routers (`src/routes`)

The API's functionality is organized into modular routers, typically with each file in the `src/routes/` directory representing a distinct set of related endpoints or a microservice.

-   **Modular Design**: Each router (e.g., `quotes_router.py`, `historical_prices_router.py`) encapsulates the logic for a specific feature (e.g., fetching quotes, retrieving historical data).
-   **Endpoint Definition**: Routers define API path operations (e.g., `@router.get("/symbols")`) along with their associated HTTP methods.
-   **Model Integration**:
    -   Path operation functions use Pydantic models as type hints for request bodies, query parameters, and path parameters. This enables FastAPI's automatic validation.
    -   The `response_model` parameter in path operation decorators is set to a Pydantic model to define the structure of the response, ensuring consistent output and enabling automatic serialization and documentation.
-   **Dependency Injection**: Routers utilize FastAPI's dependency injection system to obtain necessary services, clients (like `FinanceClient`), and other resources.
-   **Inclusion in Main App**: These individual routers are then included in the main FastAPI application instance in `main.py` (e.g., `app.include_router(quotes_router)`), often with a common prefix (like `/v1`).

## 3. Routes, Models, and Services Architecture

### 3.1. Models (`src/models`)

The application's data structures are defined as Pydantic models, which serve multiple purposes:

- **API Contract Definition**: Models like `Quote`, `SimpleQuote`, `MarketIndex` define the structure of API responses, ensuring consistency and type safety.
- **Input Validation**: Models validate incoming request parameters.
- **Documentation Generation**: FastAPI uses these models to generate OpenAPI documentation.
- **Serialization/Deserialization**: Converting between Python objects and JSON for API interactions.

### 3.2. Routes (`src/routes`)

Routes are organized into logical modules, each responsible for a specific domain:

- **`quotes_router`**: Endpoints for retrieving stock quotes
- **`historical_prices_router`**: Historical price data
- **`indices_router`**: Market indices data
- **`sectors_router`**: Industry sector performance
- **`movers_router`**: Market movers (gainers/losers)
- **`finance_news_router`**: Financial news
- **`search_router`**: Symbol search functionality
- **`hours_router`**: Market hours status
- **`similar_quotes_router`**: Similar securities
- **`stream_router`**: Server-sent events
- **`sockets_router`**: WebSocket connections

Each router defines endpoints with:
- Path and HTTP method
- Parameter and response type definitions
- Documentation and examples
- Security requirements
- Error handling specifications

### 3.3. Services (`src/services`)

Services implement the business logic for the API, acting as an intermediary between routes and data sources:

#### Service Organization

Services are organized by domain (quotes, news, sectors, etc.), with each service implementing a specific data retrieval function:

!!! info
    Example directory structure:
    ```
    src/services/
    ├── quotes/
    │   ├── fetchers/
    │   │   ├── quote_api.py    # Primary API-based fetchers
    │   │   └── quote_scraper.py  # Fallback web scraping implementation
    │   ├── get_quotes.py       # Main service functions with retry logic
    │   └── utils.py           # Helper functions for fetchers
    ├── news/
    ├── sectors/
    └── ...
    ```

#### Multi-Source Strategy Pattern

Many services implement a dual-fetching strategy:

1. **Primary Source** (usually Yahoo Finance API):
   - Faster and more reliable when available
   - Used as the first attempt

2. **Fallback Source** (usually web scraping):
   - More resilient to API changes
   - Used when primary source fails

For example, quote data has:
- `fetch_quotes()`: Primary API-based implementation in `quote_api.py`
- `scrape_quotes()`: Fallback web scraping implementation in `quote_scraper.py`

The main service function (e.g., `get_quotes()`) uses the `@retry` decorator to attempt the primary source first and fall back to web scraping if needed.

### 3.4. Caching Strategy (`src/utils/cache.py`)

The application implements a flexible caching system:

- **Cache Handlers**:
    - `RedisCacheHandler`: Distributed caching when Redis is available.
    - `MemCacheHandler`: Local in-memory caching when Redis is unavailable.

- **Smart Expiration**:
    - Standard TTL for normal market hours.
    - Extended TTL when market is closed (`market_closed_expire`).
    - Uses `MarketSchedule` to determine market open/closed status.

- **Cache Key Generation**:
    - Generated from function name and SHA-256 hash of arguments.
    - Excludes non-serializable objects like HTTP clients.

- **Type-Aware Deserialization**:
    - Maintains Pydantic model types after retrieval.
    - Special handling for collections (lists, dicts).

- **Environment Controls**:
    - `BYPASS_CACHE` environment variable to disable caching.
    - Redis connection driven by `REDIS_URL` availability.

### 3.5. Retry Mechanism (`src/utils/retry.py`)

A sophisticated retry decorator provides resilience for data fetching:

- **Configurable Retries**:
    - Attempts primary function up to specified number of times
    - Falls back to alternative implementation after exhausting retries

- **Intelligent Fallback**:
    - Analyzes function signatures to pass only compatible parameters
    - Logs exceptions and retry attempts

- **Error Handling**:
    - Propagates `HTTPException` directly (don't retry client errors)
    - Captures other exceptions for retry logic


**Usage Pattern**:
  ```python
  @retry(fallback=scrape_quotes, retries=2)
  async def get_quotes(finance_client, symbols):
      # Primary implementation using Yahoo Finance API
      return await fetch_quotes(finance_client, symbols)
  ```

## 4. Dependency Injection (`src/utils/dependencies.py`)

The application heavily utilizes `fastapi_injectable` for managing dependencies. This promotes modularity and testability by decoupling components.

- **Shared Resources**: Provides injectable dependencies for:
    - `RequestContext`: The current `Request` or `WebSocket` object, made available via `RequestContextMiddleware`.
    - `WebsocketConnectionManager`: The active WebSocket connection manager, which can be either `ConnectionManager` (in-memory) or `RedisConnectionManager` (if Redis is enabled).
    - `RedisClient`: The shared Redis client instance, available if `REDIS_URL` is configured.
    - `Session`: The shared `curl_cffi.requests.Session` for making HTTP requests, initialized during application lifespan.
    - `Proxy`: The proxy URL string, configured via environment variables.
    - `AuthManager`: The shared `YahooAuthManager` instance for handling Yahoo Finance authentication.
    - `YahooAuth`: A tuple of `(cookies, crumb)` obtained from `YahooAuthManager`, representing the current authentication credentials.
    - `MarketSchedule`: Provides information about market open/closed status and upcoming holidays.

- **Client Abstractions**:
    - `FetchClient`: An instance of `CurlFetchClient`, a general-purpose asynchronous HTTP client built on `curl_cffi`.
    - `FinanceClient`: An instance of `YahooFinanceClient`, a specialized client for interacting with the Yahoo Finance API, pre-configured with authentication details.

- **Utility Functions as Dependencies**:
    - `fetch`: A generic async function to perform HTTP requests using the `FetchClient`, including built-in retry logic. This is injectable and used by various services.
    - `get_logo`: A utility to fetch company logos, attempting `logo.dev` first and falling back to domain icons.

- **How it Works**:
    - Dependencies are defined as functions decorated with `@injectable`.
    - FastAPI automatically resolves and injects these dependencies into route handlers and other dependent functions based on type hints.
    - For example, a route handler can request a `FinanceClient` by simply type-hinting a parameter: `async def my_route(client: FinanceClient): ...`

## 5. Yahoo Authentication (`src/utils/yahoo_auth.py`)

Authentication with Yahoo Finance is crucial for accessing their unofficial API. The `YahooAuthManager` class handles this process:

- **Cookie and Crumb**: Yahoo Finance API requests require a valid cookie and a "crumb" (a type of CSRF token).
- **`YahooAuthManager`**:
    - Manages fetching and caching of the cookie and crumb.
    - A single instance is created at application startup and shared.
    - Uses an `asyncio.Lock` to prevent concurrent refresh attempts.
    - **`refresh()` method**: Fetches a new cookie/crumb pair. It first tries a direct crumb endpoint. If that fails (e.g., due to consent requirements), it navigates the Yahoo consent flow to obtain a CSRF token and session ID, then uses these to get a valid crumb.
    - **`get_or_refresh()` method**: Provides the cached cookie/crumb if still valid (within `_MIN_REFRESH_INTERVAL`), otherwise triggers a refresh. This is the primary method used by dependencies.
- **Error Handling**: Raises `YahooAuthError` if it fails to obtain a valid cookie/crumb after attempting all methods.
- **Integration**: The `YahooAuth` dependency in `dependencies.py` uses `YahooAuthManager` to provide the auth details to `YahooFinanceClient`.

## 6. HTTP Clients

The application uses `curl_cffi` for HTTP requests, wrapped in custom client classes for better organization and specific functionalities.

### 6.1. `CurlFetchClient` (`src/clients/fetch_client.py`)

This is a general-purpose asynchronous HTTP client.

- **Core Functionality**:
    - Wraps `curl_cffi.requests.Session`.
    - Provides `request()` (synchronous) and `fetch()` (asynchronous) methods.
    - Handles common HTTP methods (GET, POST, etc.).
    - Manages default headers (e.g., User-Agent) which can be overridden.
    - Supports proxy configuration.
    - Includes basic error handling, raising `HTTPException` on request failures.
- **Asynchronous Operations**: The `fetch()` method uses `asyncio.to_thread` to run synchronous `curl_cffi` calls in a separate thread, making them non-blocking for the asyncio event loop.

### 6.2. `YahooFinanceClient` (`src/clients/yahoo_client.py`)

This client inherits from `CurlFetchClient` and is specialized for Yahoo Finance API interactions.

- **Initialization**: Takes Yahoo cookies and crumb as arguments, which are automatically injected via the `FinanceClient` dependency.
- **`_yahoo_request()`**: A private helper method that:
    - Adds the `crumb` to request parameters.
    - Sets a specific User-Agent.
    - Handles Yahoo-specific HTTP error codes (401 for auth failure, 404 for not found, 429 for rate limits) by raising appropriate `HTTPException`.
- **`_json()`**: A helper to make a request using `_yahoo_request()` and parse the JSON response, with error handling for parsing failures.
- **API-Specific Methods**: Provides methods like `get_quote()`, `get_simple_quotes()`, `get_chart()`, `search()`, and `get_similar_quotes()`, each corresponding to a specific Yahoo Finance API endpoint.

## 7. WebSocket Connection Management

The API supports real-time data streaming via WebSockets. Connection management is handled by classes in `src/connections/`.

### 7.1. `ConnectionManager` (`src/connections/connection_manager.py`)

This is the default in-memory WebSocket connection manager.

- **`active_connections`**: A dictionary mapping channel names (e.g., a stock symbol) to a list of active `WebSocket` objects subscribed to that channel.
- **`tasks`**: A dictionary mapping channel names to `asyncio.Task` objects that are responsible for fetching and broadcasting data for that channel.
- **`connect()`**:
    - Adds a new WebSocket to the specified channel.
    - If it's the first connection for a channel, it creates and starts a new data-fetching task for that channel.
- **`disconnect()`**:
    - Removes a WebSocket from a channel.
    - If a channel has no more active connections, it cancels the associated data-fetching task.
- **`broadcast()`**: Sends a message (JSON) to all WebSockets connected to a specific channel.
- **`close()`**: Cleans up all connections and cancels all tasks, typically called during application shutdown.

### 7.2. `RedisConnectionManager` (`src/connections/redis_connection_manager.py`)

If Redis is configured (`REDIS_URL` is set), this manager is used to enable multi-instance WebSocket support.

- **Leverages Redis Pub/Sub**:
    - **`active_connections`**: Similar to `ConnectionManager`, but stores local connections for the current instance.
    - **`pubsub`**: A dictionary mapping channel names to Redis `PubSub` objects.
    - **`listen_tasks`**: Tasks that listen for messages on Redis Pub/Sub channels.
    - **`tasks`**: Data-fetching tasks, similar to `ConnectionManager`. One instance of the application will typically run the data fetching task for a given channel.
- **`connect()`**:
    - Subscribes the local WebSocket.
    - If not already listening, creates a `_listen_to_channel` task that subscribes to the Redis channel.
    - If no data-fetching task exists for this channel (across all instances, implicitly coordinated or by convention), it starts one.
- **`disconnect()`**:
    - Removes local WebSocket.
    - If no local connections remain for a channel, cancels the Redis listener task and the data-fetching task for that channel *on this instance*.
    - Unsubscribes from the Redis channel.
- **`_listen_to_channel()`**: Continuously polls the Redis Pub/Sub channel for messages and uses `_broadcast()` to send them to locally connected WebSockets.
- **`_broadcast()`**: Sends messages to WebSockets connected to the channel *on the current instance*.
- **`publish()`**: Publishes a message to a Redis channel. The data-fetching task uses this method to send data, which is then picked up by `_listen_to_channel()` on all instances (including the publishing one) that have subscribers for that channel.
- **`close()`**: Cleans up local connections, tasks, and Redis Pub/Sub subscriptions.

This architecture allows multiple instances of the application to share WebSocket load. A data-fetching task for a symbol might run on one instance, publish its data to Redis, and then all instances with clients interested in that symbol will receive the data via Redis Pub/Sub and forward it to their respective clients.

## 8. Rate Limiting and Security (`src/security/`)

The application implements rate limiting and basic API key security.

### 8.1. `SecurityConfig` (`src/security/rate_limit_manager.py`)

- Defines constants for security settings:
    - `ADMIN_API_KEY`: An API key that bypasses rate limits.
    - `RATE_LIMIT`: Default requests per day for normal users.
    - `HEALTH_CHECK_INTERVAL`: Cooldown period for the `/health` endpoint per IP.
    - `OPEN_PATHS`: A set of paths (like `/docs`, `/ping`) that bypass all security checks.

### 8.2. `RateLimitManager` (`src/security/rate_limit_manager.py`)

This class manages rate limit counts and health check access, stored in memory.

- **Storage**: Uses dictionaries (`rate_limits`, `health_checks`) to store `RateLimitEntry` (count, expiration) and health check timestamps per IP.
- **`_clean_expired()`**: Periodically removes stale entries.
- **`get_rate_limit_info()`**: Returns current rate limit status for an IP (count, remaining, reset time).
- **`get_health_check_info()`**: Returns whether an IP can access `/health` and when the cooldown resets.
- **`check_health_rate_limit()`**: Enforces the cooldown for the `/health` endpoint. Admin key bypasses this.
- **`increment_and_check()`**: Increments the request count for an IP and checks if the limit is exceeded. Admin key bypasses this. Returns `(is_allowed, rate_limit_info)`.
- **`validate_websocket()`**: Checks rate limits for WebSocket connections. If exceeded, the connection is closed.
- **`cleanup()`**: Clears all stored rate limit data.

### 8.3. `RateLimitMiddleware` (`src/security/rate_limit_middleware.py`)

This FastAPI middleware enforces the rate limits.

- **Dispatch Logic**:
    - Skips security for paths in `SecurityConfig.OPEN_PATHS`.
    - For `/health` path: Uses `check_health_rate_limit`. If denied, returns 429. Otherwise, adds `X-RateLimit-Reset` header for health check cooldown.
    - For other paths: Uses `increment_and_check`. If denied, returns 429. Otherwise, adds `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers to the response.
- **API Key**: Reads `x-api-key` header to identify admin users.
- **Client IP**: Uses `request.client.host` for rate limiting.

This setup provides a basic but effective way to protect the API from abuse while allowing administrative access and open access to documentation.

## 9. Authentication (`src/utils/yahoo_auth.py`)

Authentication with Yahoo Finance is crucial for accessing their unofficial API. The `YahooAuthManager` class handles this process:

- **Cookie and Crumb**: Yahoo Finance API requests require a valid cookie and a "crumb" (a type of CSRF token).
- **`YahooAuthManager`**:
    - Manages fetching and caching of the cookie and crumb.
    - A single instance is created at application startup and shared.
    - Uses an `asyncio.Lock` to prevent concurrent refresh attempts.
    - **`refresh()` method**: Fetches a new cookie/crumb pair. It first tries a direct crumb endpoint. If that fails (e.g., due to consent requirements), it navigates the Yahoo consent flow to obtain a CSRF token and session ID, then uses these to get a valid crumb.
    - **`get_or_refresh()` method**: Provides the cached cookie/crumb if still valid (within `_MIN_REFRESH_INTERVAL`), otherwise triggers a refresh. This is the primary method used by dependencies.
- **Error Handling**: Raises `YahooAuthError` if it fails to obtain a valid cookie/crumb after attempting all methods.
- **Integration**: The `YahooAuth` dependency in `dependencies.py` uses `YahooAuthManager` to provide the auth details to `YahooFinanceClient`.
