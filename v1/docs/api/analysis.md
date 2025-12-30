# Analysis API

## GET /v1/analysis/{symbol}

### Overview

**Purpose:** Retrieve comprehensive stock analysis data including analyst recommendations, upgrades/downgrades, price targets, earnings/revenue estimates, sustainability scores, and earnings history for a given stock symbol.
**Response Format:** Analysis data object containing the requested analysis type and associated data.

### Authentication

Optional authentication via `x-api-key` header token

### Path Parameters

| Parameter | Type   | Required | Description                     | Example |
|-----------|--------|:--------:|---------------------------------|---------|
| `symbol`  | string |    ✓     | The stock ticker symbol         | `AAPL`  |

### Query Parameters

| Parameter      | Type   | Required | Description                               | Example                    |
|----------------|--------|:--------:|-------------------------------------------|----------------------------|
| `analysis_type`| string |    ✓     | Type of analysis data to retrieve         | `recommendations`          |

#### Available Analysis Types

| Analysis Type        | Description                                    | Example |
|----------------------|------------------------------------------------|---------|
| `recommendations`    | Analyst recommendations (strong buy, buy, hold, sell, strong sell) | `recommendations` |
| `upgrades_downgrades`| Recent analyst upgrades and downgrades         | `upgrades_downgrades` |
| `price_targets`      | Analyst price targets (current, mean, median, high, low) | `price_targets` |
| `earnings_estimate`  | Earnings estimates for upcoming periods       | `earnings_estimate` |
| `revenue_estimate`  | Revenue estimates for upcoming periods        | `revenue_estimate` |
| `earnings_history`  | Historical earnings data                      | `earnings_history` |
| `sustainability`    | ESG (Environmental, Social, Governance) scores | `sustainability` |

---

### Responses

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** [`AnalysisData`](#analysisdata-schema)
  - **Example - Recommendations (200):**
    ```json
    {
      "symbol": "AAPL",
      "analysis_type": "recommendations",
      "recommendations": [
        {
          "period": "3m",
          "strong_buy": 5,
          "buy": 10,
          "hold": 3,
          "sell": 1,
          "strong_sell": 0
        }
      ]
    }
    ```

  - **Example - Price Targets (200):**
    ```json
    {
      "symbol": "AAPL",
      "analysis_type": "price_targets",
      "price_targets": {
        "current": 150.0,
        "mean": 160.0,
        "median": 155.0,
        "low": 140.0,
        "high": 180.0
      }
    }
    ```

  - **Example - Earnings Estimate (200):**
    ```json
    {
      "symbol": "AAPL",
      "analysis_type": "earnings_estimate",
      "earnings_estimate": {
        "estimates": {
          "2024-12-31": {
            "avg": 6.5,
            "low": 6.0,
            "high": 7.0
          },
          "2025-12-31": {
            "avg": 7.2,
            "low": 6.8,
            "high": 7.6
          }
        }
      }
    }
    ```

  - **Example - Sustainability (200):**
    ```json
    {
      "symbol": "AAPL",
      "analysis_type": "sustainability",
      "sustainability": {
        "scores": {
          "environmentScore": 75,
          "socialScore": 80,
          "governanceScore": 85,
          "totalEsg": 80
        }
      }
    }
    ```

- **404 Not Found**
  ```json
  { "detail": "Symbol not found or no analysis data available for the specified type" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": [
      {
        "type": "enum",
        "loc": ["query", "analysis_type"],
        "msg": "Input should be 'recommendations', 'upgrades_downgrades', 'price_targets', 'earnings_estimate', 'revenue_estimate', 'earnings_history', or 'sustainability'",
        "input": "invalid_type"
      }
    ]
  }
  ```

---

### Schema References

#### AnalysisData Schema

| Field           | Type                                    | Description                          | Required |
|-----------------|-----------------------------------------|--------------------------------------|:--------:|
| symbol          | string                                  | Stock symbol (e.g., "AAPL")         |    ✓     |
| analysis_type   | string                                  | Type of analysis data requested      |    ✓     |
| recommendations | [RecommendationData[]](#recommendationdata) | Analyst recommendations data        |          |
| upgrades_downgrades | [UpgradeDowngrade[]](#upgradedowngrade) | Analyst upgrades/downgrades data    |          |
| price_targets   | [PriceTarget](#pricetarget)            | Analyst price targets data           |          |
| earnings_estimate | [EarningsEstimate](#earningsestimate) | Earnings estimates data              |          |
| revenue_estimate | [RevenueEstimate](#revenueestimate)   | Revenue estimates data                |          |
| earnings_history | [EarningsHistoryItem[]](#earningshistoryitem) | Historical earnings data         |          |
| sustainability  | [SustainabilityScores](#sustainabilityscores) | ESG sustainability scores data |          |

#### RecommendationData

| Field       | Type   | Description                          | Required |
|-------------|--------|--------------------------------------|:--------:|
| period      | string | Time period (e.g., "3m", "1m")      |    ✓     |
| strong_buy  | int    | Number of strong buy recommendations |          |
| buy         | int    | Number of buy recommendations        |          |
| hold        | int    | Number of hold recommendations       |          |
| sell        | int    | Number of sell recommendations       |          |
| strong_sell | int    | Number of strong sell recommendations|          |

#### UpgradeDowngrade

| Field      | Type     | Description                          | Required |
|------------|----------|--------------------------------------|:--------:|
| firm       | string   | Firm providing the upgrade/downgrade |          |
| to_grade   | string   | New grade/recommendation             |          |
| from_grade | string   | Previous grade/recommendation         |          |
| action     | string   | Action taken (upgrade, downgrade)    |          |
| date       | datetime | Date of the action                   |          |

#### PriceTarget

| Field  | Type   | Description                          | Required |
|--------|--------|--------------------------------------|:--------:|
| current| float  | Current stock price                  |          |
| mean   | float  | Mean analyst price target           |          |
| median | float  | Median analyst price target         |          |
| low    | float  | Lowest analyst price target          |          |
| high   | float  | Highest analyst price target         |          |

#### EarningsEstimate

| Field     | Type  | Description                          | Required |
|-----------|-------|--------------------------------------|:--------:|
| estimates | object| Earnings estimates by period         |    ✓     |

#### RevenueEstimate

| Field     | Type  | Description                          | Required |
|-----------|-------|--------------------------------------|:--------:|
| estimates | object| Revenue estimates by period          |    ✓     |

#### EarningsHistoryItem

| Field          | Type     | Description                          | Required |
|----------------|----------|--------------------------------------|:--------:|
| date           | datetime | Date of earnings report              |          |
| eps_actual     | float    | Actual earnings per share            |          |
| eps_estimate   | float    | Estimated earnings per share         |          |
| surprise       | float    | Earnings surprise amount              |          |
| surprise_percent| float  | Earnings surprise percentage         |          |

#### SustainabilityScores

| Field  | Type  | Description                          | Required |
|--------|-------|--------------------------------------|:--------:|
| scores | object| ESG scores and metrics               |    ✓     |

---

### Usage Examples

#### Get Analyst Recommendations
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/AAPL?analysis_type=recommendations" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Price Targets
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/MSFT?analysis_type=price_targets" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Earnings Estimates
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/GOOGL?analysis_type=earnings_estimate" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Sustainability Scores
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/TSLA?analysis_type=sustainability" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Earnings History
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/NVDA?analysis_type=earnings_history" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Revenue Estimates
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/AMZN?analysis_type=revenue_estimate" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Upgrades/Downgrades
```bash
curl -X GET "https://finance-query.onrender.com/v1/analysis/META?analysis_type=upgrades_downgrades" \
     -H "x-api-key: YOUR_API_KEY"
```

---

### Data Sources

This endpoint uses the `yfinance` library to fetch real-time analysis data from Yahoo Finance, providing comprehensive analyst insights and market data for informed investment decisions.

### Error Handling

- **Graceful Degradation**: Missing data fields return `null` rather than errors
- **Symbol Validation**: Invalid symbols return 404 Not Found
- **Type Validation**: Invalid analysis types return 422 Unprocessable Entity
- **Server Errors**: Unexpected errors return 500 Internal Server Error
