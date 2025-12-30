# Market Movers

## GET /v1/actives

### Overview

**Purpose:** Get list of most actively traded stocks by volume
**Response Format:** Array of most active stocks with price movement data

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                                 | Example |
|-----------|--------|:--------:|---------------------------------------------|:-------:|
| `count`   | string |          | Number of actives to retrieve (default: 50) |  `25`   |

#### Available Count Options

`25`, `50`, `100`

```bash
curl -X GET "https://finance-query.onrender.com/v1/actives?count=25" \
     -H "x-api-key: your_api_key_here"
```

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** Array of [`MarketMover`](#marketmover-schema) objects.
    - **Example (200):**
      ```json
      [
        {
          "symbol": "AAPL",
          "name": "Apple Inc.",
          "price": "145.86",
          "change": "+1.00",
          "percentChange": "+0.69%"
        }
      ]
      ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "count": ["Input should be '25', '50' or '100'"]
    }
  }
  ```

## GET /v1/gainers

### Overview

**Purpose:** Get list of stocks with the highest price increase
**Response Format:** Array of top gaining stocks with price movement data

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                                 | Example |
|-----------|--------|:--------:|---------------------------------------------|:-------:|
| `count`   | string |          | Number of gainers to retrieve (default: 50) |  `25`   |

#### Available Count Options

`25`, `50`, `100`

```bash
curl -X GET "https://finance-query.onrender.com/v1/gainers?count=25" \
     -H "x-api-key: your_api_key_here"
```

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** Array of [`MarketMover`](#marketmover-schema) objects.
    - **Example (200):**
      ```json
      [
        {
          "symbol": "TSLA",
          "name": "Tesla Inc.",
          "price": "900.00",
          "change": "+25.00",
          "percentChange": "+2.86%"
        }
      ]
      ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "count": ["Input should be '25', '50' or '100'"]
    }
  }
  ```

## GET /v1/losers

### Overview

**Purpose:** Get list of stocks with the highest price decrease
**Response Format:** Array of top losing stocks with price movement data

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                                | Example |
|-----------|--------|:--------:|--------------------------------------------|:-------:|
| `count`   | string |          | Number of losers to retrieve (default: 50) |  `25`   |

#### Available Count Options

`25`, `50`, `100`

```bash
curl -X GET "https://finance-query.onrender.com/v1/losers?count=25" \
     -H "x-api-key: your_api_key_here"
```

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** Array of [`MarketMover`](#marketmover-schema) objects.
    - **Example (200):**
      ```json
      [
        {
          "symbol": "AMZN",
          "name": "Amazon.com Inc.",
          "price": "110.00",
          "change": "-5.00",
          "percentChange": "-4.35%"
        }
      ]
      ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "count": ["Input should be '25', '50' or '100'"]
    }
  }
  ```

## Schema References

### MarketMover Schema

| Field           | Type   | Description       | Required |
|:----------------|:-------|:------------------|:--------:|
| `symbol`        | string | Stock symbol      |    ✓     |
| `name`          | string | Company name      |    ✓     |
| `price`         | string | Last traded price |    ✓     |
| `change`        | string | Change in price   |    ✓     |
| `percentChange` | string | Percentage change |    ✓     |
