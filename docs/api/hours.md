# Market Hours

## GET /v1/hours

### Overview

**Purpose:** Retrieve the current market status  
**Response Format:** JSON object with market status, reason, and timestamp

### Authentication

Optional authentication via `x-api-key` header token

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Example (200):**
      ```json
      {
        "status": "open",
        "reason": "Regular trading hours.",
        "timestamp": "2021-09-22T14:00:00.000Z"
      }
      ```

## Schema References

### MarketStatus Schema

| Field      | Type   | Description                                     | Required |
|------------|--------|-------------------------------------------------|----------|
| status     | string | Current market status ("open", "closed", etc.)  | ✓        |
| reason     | string | Explanation for the current status              | ✓        |
| timestamp  | string | ISO timestamp of the status check               | ✓        |
