# Historical Prices

## GET /v1/historical

### Overview

**Purpose:** Historical stock price data retrieval  
**Response Format:** Time-series OHLCV (Open, High, Low, Close, Volume) data

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter  | Type    | Required | Description                           | Example |
|------------|---------|:--------:|---------------------------------------|---------|
| `symbol`   | string  |    ✓     | Stock ticker symbol                   | `NVDA`  |
| `range`    | string  |    ✓     | Historical time range                 | `1y`    |
| `interval` | string  |    ✓     | Data point frequency                  | `1d`    |
| `epoch`    | boolean |          | Use epoch timestamps (default: false) | `true`  |

#### Available Range Options
`1d`, `5d`, `1mo`, `3mo`, `6mo`, `ytd`, `1y`, `2y`, `5y`, `10y`, `max`

#### Available Interval Options
`1m`, `5m`, `15m`, `30m`, `1h`, `1d`, `1wk`, `1mo`

!!! warning "Interval and Range Compatibility"
    | Interval | Compatible Ranges                                   |
    |----------|-----------------------------------------------------|
    | `1m`     | `1d`, `5d` only                                     |
    | `5m`     | `1d`, `5d`, `1mo` only                              |
    | `15m`    | `1d`, `5d`, `1mo` only                              |
    | `30m`    | `1d`, `5d`, `1mo` only                              |
    | `1h`     | `1d`, `5d`, `1mo`, `3mo`, `6mo`, `ytd`, `1y` only   |
    | `1mo`    | Required for `max` range                            |
    
    Attempting incompatible combinations will result in a 400 Bad Request error.

**Responses:**

- **200 OK**  
  - **Content-Type:** `application/json`  
  - **Schema:** Object whose keys are dates (or epoch) and values are [`HistoricalData`](#historicaldata-schema) objects.  
  - **Example (200):**
    ```json
    {
      "2023-10-01": {
        "open": 300.0,
        "high": 305.0,
        "low": 295.0,
        "close": 302.0,
        "adjClose": 302.0,
        "volume": 1500000
      },
      "2023-10-02": {
        "open": 302.0,
        "high": 310.0,
        "low": 300.0,
        "close": 308.0,
        "adjClose": 308.0,
        "volume": 1600000
      }
    }
    ```

- **400 Bad Request**  
  ```json
  { "detail": "If interval is 1m, 5m, 15m or 30m, time period must be 1mo or less" }
  ```

- **404 Not Found**
  ```json
  { "detail": "Symbol not found" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "symbol": ["Field required"],
      "range": ["Field required"],
      "interval": ["Field required"]
    }
  }
  ```

## Schema References

### HistoricalData Schema

| Field      | Type    | Description            | Required |
|:-----------|:--------|:-----------------------|:--------:|
| `open`     | number  | Opening price          |    ✓     |
| `high`     | number  | Highest price          |    ✓     |
| `low`      | number  | Lowest price           |    ✓     |
| `close`    | number  | Closing price          |    ✓     |
| `adjClose` | number  | Adjusted closing price |          |
| `volume`   | integer | Volume traded          |    ✓     |
