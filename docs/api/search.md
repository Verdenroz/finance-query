# Search

## Search Provider

This endpoint uses Algolia search with my personal credentials provided for your convenience:

1. **Shared credentials**: I've shared my personal Algolia credentials (free tier, search-only) for open use. Please use
   responsibly.
2. **Your own credentials**: You can configure your own Algolia credentials via environment variables:
    - `ALGOLIA_APP_ID`: Your Algolia application ID
    - `ALGOLIA_API_KEY`: Your Algolia search API key
3. **Yahoo Finance fallback**: The API falls back to Yahoo Finance search automatically on errors or on configured
   request parameter `yahoo`.

!!! info "Default Credentials"
    The API includes shared Algolia credentials for convenience. These are search-only and cannot modify data.
    If you need high-volume access, please configure your own credentials.

## GET /v1/search

### Overview

**Purpose:** Search for stocks by company name or symbol  
**Response Format:** Array of matching securities with basic information and metadata

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter     | Type    | Required | Description                                     | Example |
|---------------|---------|:--------:|-------------------------------------------------|---------|
| `query`       | string  |    ✓     | Partial or full company name or symbol          | `Apple` |
| `hits`        | integer |          | Number of results to return (default: 50)       | `25`    |
| `type`        | string  |          | Filter by security type (default: all)          | `stock` |
| `use_algolia` | boolean |          | Whether to use Algolia or Yahoo (default: true) | `false` |

!!! warning "Parameter Constraints"
    - `hits` parameter accepts values between 1 and 100
    - `type` parameter accepts: `stock`, `etf`, or `trust`

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** Array of [`SearchResult`](#searchresult-schema) objects
    - **Example (200):**
      ```json
      [
        {
          "name": "Apple Inc.",
          "symbol": "AAPL",
          "exchange": "NASDAQ",
          "type": "stock"
        }
      ]
      ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "query": ["Field required"],
      "hits": ["Input should be less than or equal to 100"],
      "type": ["Input should be 'stock', 'etf', or 'trust'"]
    }
  }
  ```

## Schema References

### SearchResult Schema

| Field    | Type   | Description                                     | Required |
|----------|--------|-------------------------------------------------|:--------:|
| name     | string | Full company name (e.g., "Apple Inc.")          |    ✓     |
| symbol   | string | Stock symbol (e.g., "AAPL")                     |    ✓     |
| exchange | string | Exchange where security trades (e.g., "NASDAQ") |    ✓     |
| type     | string | Security type (e.g., "stock", "etf", "trust")   |    ✓     |
