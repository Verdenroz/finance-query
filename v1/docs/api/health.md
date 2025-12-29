# Health Endpoints

## GET /v1/health

### Overview

**Purpose:** Detailed health check of the API and its dependencies
**Response Format:** JSON object with service statuses and metrics

### Authentication

None required


**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Example (200):**
      ```json
      {
        "status": "healthy",
        "timestamp": "2025-05-13T19:35:38.383240",
        "redis": {
            "status": "healthy",
            "latency_ms": 18.45
        },
        "services": {
            "status": "28/28 succeeded",
            "Indices": {
                "status": "succeeded"
            },
            "Market Actives": {
                "status": "succeeded"
            },
            "Market Losers": {
                "status": "succeeded"
            },
            "Market Gainers": {
                "status": "succeeded"
            },
            "Market Sectors": {
                "status": "succeeded"
            },
            "Sector for a symbol": {
                "status": "succeeded"
            },
            "Detailed Sector": {
                "status": "succeeded"
            },
            "General News": {
                "status": "succeeded"
            },
            "News for equity": {
                "status": "succeeded"
            },
            "News for ETF": {
                "status": "succeeded"
            },
            "Full Quotes": {
                "status": "succeeded"
            },
            "Simple Quotes": {
                "status": "succeeded"
            },
            "Similar Equities": {
                "status": "succeeded"
            },
            "Similar ETFs": {
                "status": "succeeded"
            },
            "Historical day prices": {
                "status": "succeeded"
            },
            "Historical month prices": {
                "status": "succeeded"
            },
            "Historical year prices": {
                "status": "succeeded"
            },
            "Historical five year prices": {
                "status": "succeeded"
            },
            "Search": {
                "status": "succeeded"
            },
            "Technical Indicators": {
                "status": "succeeded"
            },
            "Market Hours": {
                "status": "succeeded"
            },
            "Annual Income Statement": {
                "status": "succeeded"
            },
            "Quarterly Balance Sheet": {
                "status": "succeeded"
            },
            "Institutional Holders": {
                "status": "succeeded"
            },
            "Analyst Recommendations": {
                "status": "succeeded"
            },
            "Price Targets": {
                "status": "succeeded"
            },
            "Earnings Calls List": {
                "status": "succeeded"
            },
            "Earnings Transcript": {
                "status": "succeeded"
            }
        }
      }
      ```

## GET /v1/ping

### Overview

**Purpose:** Simple connectivity check
**Response Format:** Basic JSON health status with timestamp

### Authentication

None required

### Request Parameters

None

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Example (200):**
      ```json
      {
        "status": "healthy",
        "timestamp": "2023-10-01T12:34:56.789Z"
      }
      ```

## Schema References

### HealthStatus Schema

| Field     | Type   | Description                            | Required |
|-----------|--------|----------------------------------------|:--------:|
| status    | string | Overall health status of the API       |    ✓     |
| timestamp | string | ISO timestamp of the health check      |    ✓     |
| redis     | object | Redis service status and metrics       |          |
| services  | object | Individual service statuses and checks |          |

### PingStatus Schema

| Field     | Type   | Description                       | Required |
|-----------|--------|-----------------------------------|:--------:|
| status    | string | Health status of the API          |    ✓     |
| timestamp | string | ISO timestamp of the ping request |    ✓     |
