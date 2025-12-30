# Server-Sent Events (SSE)

## GET /v1/stream/quotes

### Overview

**Purpose:** Stream real-time stock quotes via Server-Sent Events
**Response Format:** Text event stream with stock quote updates every 10 seconds

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                           | Example     |
|-----------|--------|:--------:|---------------------------------------|-------------|
| `symbols` | string |    ✓     | Comma-separated list of stock symbols | `NVDA,TSLA` |

**Responses:**

- **200 OK**
    - **Content-Type:** `text/event-stream`
    - **Example Event Stream:**
      ```
      quote: [
        {
          "symbol":"NVDA",
          "name":"NVIDIA Corporation",
          "price":"142.62",
          "change":"-4.60",
          "percentChange":"-3.12%",
          "logo":"https://img.logo.dev/nvidia.com?token=..."
        }
      ]

      quote: [
        {
          "symbol":"NVDA",
          "name":"NVIDIA Corporation",
          "price":"143.00",
          "change":"+0.38",
          "percentChange":"+0.27%",
          "logo":"https://img.logo.dev/nvidia.com?token=..."
        }
      ]
      ```

- **404 Not Found**
  ```json
  { "detail": "Symbol not found" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": { "symbols": ["Field required"] }
  }
  ```

## Schema References

### SimpleQuote Schema

| Field            | Type   | Description                | Required |
|------------------|--------|----------------------------|:--------:|
| symbol           | string | Stock symbol               |    ✓     |
| name             | string | Company name               |    ✓     |
| price            | string | Last traded price          |    ✓     |
| change           | string | Change in price            |    ✓     |
| percentChange    | string | Percentage change          |    ✓     |
| logo             | string | URL to company logo        |          |
