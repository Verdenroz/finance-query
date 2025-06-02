# Sectors

## GET /v1/sectors

### Overview

**Purpose:** Retrieve summary performance data for all market sectors  
**Response Format:** Array of sector performance summaries with returns over multiple time periods

### Authentication

Optional authentication via `x-api-key` header token

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** Array of [`MarketSector`](#marketsector-schema) objects
  - **Example (200):**
    ```json
    [
      {
        "sector": "Technology",
        "dayReturn": "-0.69%",
        "ytdReturn": "-2.36%",
        "yearReturn": "+24.00%",
        "threeYearReturn": "+50.20%",
        "fiveYearReturn": "+158.41%"
      },
      {
        "sector": "Healthcare",
        "dayReturn": "+0.87%",
        "ytdReturn": "+7.45%",
        "yearReturn": "+4.04%",
        "threeYearReturn": "+7.59%",
        "fiveYearReturn": "+44.74%"
      }
    ]
    ```

## GET /v1/sectors/symbol/{symbol}

### Overview

**Purpose:** Retrieve sector performance summary for a specific stock symbol  
**Response Format:** Single sector performance object with returns over multiple time periods

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description           | Example |
|-----------|--------|:--------:|-----------------------|---------|
| `symbol`  | string |    ✓     | Stock symbol          | `AAPL`  |

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** [`MarketSector`](#marketsector-schema) object
  - **Example (200):**
    ```json
    {
      "sector": "Technology",
      "dayReturn": "-0.46%",
      "ytdReturn": "-2.13%",
      "yearReturn": "+24.28%",
      "threeYearReturn": "+50.55%",
      "fiveYearReturn": "+159.00%"
    }
    ```

- **404 Not Found**
  ```json
  { "detail": "Sector for {symbol} not found" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": { "symbol": ["Field required"] }
  }
  ```

## GET /v1/sectors/details/{sector}

### Overview

**Purpose:** Retrieve comprehensive details for a specific market sector  
**Response Format:** Detailed sector information including performance, composition, and top holdings

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter | Type   | Required | Description    | Example      |
|-----------|--------|:--------:|----------------|--------------|
| `sector`  | string |    ✓     | Sector name    | `Technology` |

**Valid sector names:**
- Basic Materials
- Communication Services
- Consumer Cyclical
- Consumer Defensive
- Energy
- Financial Services
- Healthcare
- Industrials
- Real Estate
- Technology
- Utilities

**Responses:**

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** [`MarketSectorDetails`](#marketsectordetails-schema) object
  - **Example (200):**
    ```json
    {
      "sector": "Technology",
      "dayReturn": "+0.97%",
      "ytdReturn": "+3.35%",
      "yearReturn": "+32.59%",
      "threeYearReturn": "+66.92%",
      "fiveYearReturn": "+179.23%",
      "marketCap": "20.196T",
      "marketWeight": "29.28%",
      "industries": 12,
      "companies": 815,
      "topIndustries": [
        "Semiconductors: 29.04%",
        "Software - Infrastructure: 26.44%",
        "Consumer Electronics: 16.60%",
        "Software - Application: 13.92%",
        "Information Technology Services: 4.53%"
      ],
      "topCompanies": ["NVDA","AAPL","MSFT","AVGO","ORCL","CRM","CSCO","NOW","ACN","IBM"]
    }
    ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "sector": [
        "Input should be 'Basic Materials', 'Communication Services', 'Consumer Cyclical', 'Consumer Defensive', 'Energy', 'Financial Services', 'Healthcare', 'Industrials', 'Real Estate', 'Technology' or 'Utilities'"
      ]
    }
  }
  ```

## Schema References

### MarketSector Schema

| Field           | Type   | Description                            | Required |
|-----------------|--------|----------------------------------------|:--------:|
| sector          | string | Sector name (e.g., "Technology")       |    ✓     |
| dayReturn       | string | Day change percentage (e.g., "-0.69%") |    ✓     |
| ytdReturn       | string | Year-to-date return (e.g., "-2.36%")   |    ✓     |
| yearReturn      | string | 1-year return (e.g., "+24.00%")        |    ✓     |
| threeYearReturn | string | 3-year return (e.g., "+50.20%")        |    ✓     |
| fiveYearReturn  | string | 5-year return (e.g., "+158.41%")       |    ✓     |

### MarketSectorDetails Schema

| Field           | Type     | Description                                        | Required |
|-----------------|----------|----------------------------------------------------|:--------:|
| sector          | string   | Sector name (e.g., "Technology")                   |    ✓     |
| dayReturn       | string   | Day change percentage (e.g., "+0.97%")             |    ✓     |
| ytdReturn       | string   | Year-to-date return (e.g., "+3.35%")               |    ✓     |
| yearReturn      | string   | 1-year return (e.g., "+32.59%")                    |    ✓     |
| threeYearReturn | string   | 3-year return (e.g., "+66.92%")                    |    ✓     |
| fiveYearReturn  | string   | 5-year return (e.g., "+179.23%")                   |    ✓     |
| marketCap       | string   | Total market capitalization (e.g., "20.196T")      |    ✓     |
| marketWeight    | string   | Sector's weight in overall market (e.g., "29.28%") |    ✓     |
| industries      | integer  | Number of industries in the sector                 |    ✓     |
| companies       | integer  | Number of companies in the sector                  |    ✓     |
| topIndustries   | string[] | List of top industries with percentages            |    ✓     |
| topCompanies    | string[] | List of top company symbols in the sector          |    ✓     |
