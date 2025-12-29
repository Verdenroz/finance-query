# Quotes

## GET /v1/quotes

### Overview

**Purpose:** Retrieve comprehensive quote data for multiple stocks
**Response Format:** Detailed stock information with all available fields

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                     | Example          |
|-----------|--------|:--------:|---------------------------------|------------------|
| `symbols` | string |    ✓     | Comma-separated list of tickers | `AAPL,MSFT,GOOG` |

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** Array of [`Quote`](#quote-schema) objects
  - **Example (200):**
    ```json
    [
      {
        "symbol": "AAPL",
        "name": "Apple Inc.",
        "price": "145.00",
        "preMarketPrice": "145.50",
        "afterHoursPrice": "145.50",
        "change": "+1.00",
        "percentChange": "+0.69%",
        "open": "144.00",
        "high": "146.00",
        "low": "143.00",
        "yearHigh": "150.00",
        "yearLow": "100.00",
        "volume": 1000000,
        "avgVolume": 2000000,
        "marketCap": "2.5T",
        "beta": 1.23,
        "pe": "30.00",
        "eps": "4.50",
        "dividend": "0.82",
        "yield": "1.3%",
        "exDividend": "Feb 5, 2024",
        "netAssets": "10.5B",
        "nav": "100.00",
        "expenseRatio": "0.05%",
        "category": "Large Growth",
        "lastCapitalGain": "10.00",
        "morningstarRating": "★★",
        "morningstarRiskRating": "Low",
        "holdingsTurnover": "5.00%",
        "earningsDate": "Apr 23, 2024",
        "lastDividend": "0.82",
        "inceptionDate": "Jan 1, 2020",
        "sector": "Technology",
        "industry": "Consumer Electronics",
        "about": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide.",
        "employees": "150,000",
        "fiveDaysReturn": "-19.35%",
        "oneMonthReturn": "-28.48%",
        "threeMonthReturn": "-14.02%",
        "sixMonthReturn": "36.39%",
        "ytdReturn": "+10.00%",
        "yearReturn": "+20.00%",
        "threeYearReturn": "+30.00%",
        "fiveYearReturn": "+40.00%",
        "tenYearReturn": "2,005.31%",
        "maxReturn": "22,857.89%",
        "logo": "https://img.logo.dev/apple.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"
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

## GET /v1/simple-quotes

### Overview

**Purpose:** Retrieve simplified quote data for multiple stocks
**Response Format:** Basic stock information including symbols, names, prices, and changes

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description                     | Example          |
|-----------|--------|:--------:|---------------------------------|------------------|
| `symbols` | string |    ✓     | Comma-separated list of tickers | `AAPL,MSFT,GOOG` |

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** Array of [`SimpleQuote`](#simplequote-schema) objects.
  - **Example (200):**
    ```json
    [
      {
        "symbol": "AAPL",
        "name": "Apple Inc.",
        "price": "145.00",
        "change": "+1.00",
        "percentChange": "+0.69%",
        "logo": "https://img.logo.dev/apple.com?token=…"
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

## GET /v1/similar

### Overview

**Purpose:** Find stocks similar to a specific ticker
**Response Format:** List of comparable stocks with simplified quote data

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type    | Required | Description                   | Example  |
|-----------|---------|:--------:|-------------------------------|----------|
| `symbol`  | string  |    ✓     | Base stock for comparison     | `AAPL`   |
| `limit`   | integer |          | Maximum results (default: 10) | `15`     |

**Note:** Limit parameter accepts values between 1 and 20.

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** Array of [`SimpleQuote`](#simplequote-schema) objects.
  - **Example (200):**
    ```json
    [
      {
        "symbol": "AAPL",
        "name": "Apple Inc.",
        "price": "146.06",
        "change": "-0.11",
        "percentChange": "-0.11%",
        "logo": "https://img.logo.dev/apple.com?token=…"
      }
    ]
    ```

- **404 Not Found**
  ```json
  { "detail": "No similar stocks found or invalid symbol" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "limit": ["Input should be greater than or equal to 1 and less than or equal to 20"]
    }
  }
  ```

## Schema References

### Quote Schema

| Field                 | Type    | Description                                | Required |
|-----------------------|---------|--------------------------------------------|:--------:|
| symbol                | string  | Stock symbol (e.g., "AAPL")                |    ✓     |
| name                  | string  | Company name (e.g., "Apple Inc.")          |    ✓     |
| price                 | string  | Last traded price (e.g., "145.00")         |    ✓     |
| preMarketPrice        | string  | Pre-market price (if available)            |          |
| afterHoursPrice       | string  | After-hours price (if available)           |          |
| change                | string  | Change in price (e.g., "+1.00")            |    ✓     |
| percentChange         | string  | Percentage change (e.g., "+0.69%")         |    ✓     |
| open                  | string  | Opening price of the stock                 |          |
| high                  | string  | Highest price of the trading day           |          |
| low                   | string  | Lowest price of the trading day            |          |
| yearHigh              | string  | 52-week high price                         |          |
| yearLow               | string  | 52-week low price                          |          |
| volume                | integer | Volume traded                              |          |
| avgVolume             | integer | Average volume                             |          |
| marketCap             | string  | Market capitalization                      |          |
| beta                  | string  | Beta of the stock                          |          |
| pe                    | string  | Price-to-earnings ratio                    |          |
| eps                   | string  | Earnings per share                         |          |
| dividend              | string  | Dividend yield                             |          |
| yield                 | string  | Dividend yield in percentage               |          |
| exDividend            | string  | Ex-dividend date                           |          |
| netAssets             | string  | Net assets (for funds)                     |          |
| nav                   | string  | Net asset value (for funds)                |          |
| expenseRatio          | string  | Expense ratio (for funds)                  |          |
| category              | string  | Fund category (e.g., "Large Growth")       |          |
| lastCapitalGain       | string  | Last capital gain distribution (for funds) |          |
| morningstarRating     | string  | Morningstar rating (for funds)             |          |
| morningstarRiskRating | string  | Morningstar risk rating (for funds)        |          |
| holdingsTurnover      | string  | Holdings turnover (for funds)              |          |
| earningsDate          | string  | Next earnings date (if available)          |          |
| lastDividend          | string  | Last dividend distribution                 |          |
| inceptionDate         | string  | Inception date (for funds)                 |          |
| sector                | string  | Sector of the company                      |          |
| industry              | string  | Industry of the company                    |          |
| about                 | string  | Company description                        |          |
| employees             | string  | Number of employees                        |          |
| fiveDaysReturn        | string  | 5-day return                               |          |
| oneMonthReturn        | string  | 1-month return                             |          |
| threeMonthReturn      | string  | 3-month return                             |          |
| sixMonthReturn        | string  | 6-month return                             |          |
| ytdReturn             | string  | Year-to-date return                        |          |
| yearReturn            | string  | 1-year return                              |          |
| threeYearReturn       | string  | 3-year return                              |          |
| fiveYearReturn        | string  | 5-year return                              |          |
| tenYearReturn         | string  | 10-year return                             |          |
| maxReturn             | string  | All-time maximum return                    |          |
| logo                  | string  | URL to company logo                        |          |

### SimpleQuote Schema

| Field           | Type   | Description                | Required |
|-----------------|--------|----------------------------|:--------:|
| symbol          | string | Stock symbol               |    ✓     |
| name            | string | Company name               |    ✓     |
| price           | string | Last traded price          |    ✓     |
| preMarketPrice  | string | Pre-market price (if any)  |          |
| afterHoursPrice | string | After-hours price (if any) |          |
| change          | string | Change in price            |    ✓     |
| percentChange   | string | Percentage change          |    ✓     |
| logo            | string | URL to company logo        |          |
