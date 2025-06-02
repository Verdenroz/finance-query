# News

## GET /v1/news

### Overview

**Purpose:** Get financial news  
**Response Format:** Array of news articles with metadata

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                                     | Example |
|-----------|--------|:--------:|-------------------------------------------------|:-------:|
| `symbol`  | string |          | Ticker symbol (returns general news if omitted) | `AAPL`  |

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** Array of [`News`](#news-schema) objects.
    - **Example (200):**
      ```json
      [
        {
          "title": "New iPhone Released!",
          "link": "https://www.example.com/new-iphone",
          "source": "TechCrunch",
          "img": "https://www.example.com/image.jpg",
          "time": "1 day ago"
        }
      ]
      ```

- **404 Not Found**
  ```json
  { "detail": "No news found for the given symbol" }
  ```

- **422 Unprocessable Entity**
  ```json
  { "detail": "Invalid request" }
  ```

## Schema References

### News Schema

| Field  | Type   | Description               | Required |
|:-------|:-------|:--------------------------|:--------:|
| title  | string | Title of the news article |    ✓     |
| link   | string | URL to the full article   |    ✓     |
| source | string | News source               |    ✓     |
| img    | string | URL to accompanying image |    ✓     |
| time   | string | Time relative to now      |    ✓     |
