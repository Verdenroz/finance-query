# Financials

## GET /v1/financials/{symbol}

### Overview

**Purpose:** Retrieve financial statements for a given stock symbol.  
**Response Format:** A financial statement object containing the data for the requested type and frequency.

### Authentication

Optional authentication via `x-api-key` header token

### Path Parameters

| Parameter | Type   | Required | Description                     | Example |
|-----------|--------|:--------:|---------------------------------|---------|
| `symbol`  | string |    ✓     | The stock ticker symbol         | `AAPL`  |

### Query Parameters

| Parameter   | Type          | Required | Description                               | Example    |
|-------------|---------------|:--------:|-------------------------------------------|------------|
| `statement` | StatementType |    ✓     | The type of statement (`income`, `balance`, `cashflow`) | `income`   |
| `frequency` | Frequency     |          | The frequency of the report (`annual`, `quarterly`). Defaults to `annual`. | `annual`   |

---

### Responses

- **200 OK**  
  - **Content-Type:** `application/json`  
  - **Schema:** [`FinancialStatement`](#financialstatement-schema)
  - **Example (200):**
    ```json
    {
      "symbol": "AAPL",
      "statement_type": "income",
      "frequency": "annual",
      "statement": {
        "2023-09-30": {
          "Total Revenue": 383285000000,
          "Operating Revenue": 383285000000,
          "Cost Of Revenue": 214137000000,
          "Gross Profit": 169148000000,
          "Operating Expense": 54847000000,
          "Operating Income": 114301000000,
          "Net Income": 96995000000
        },
        "2022-09-30": {
          "Total Revenue": 394328000000,
          "Operating Revenue": 394328000000,
          "Cost Of Revenue": 223546000000,
          "Gross Profit": 170782000000,
          "Operating Expense": 51345000000,
          "Operating Income": 119437000000,
          "Net Income": 99803000000
        }
      }
    }
    ```

- **404 Not Found**  
  ```json
  { "detail": "No data found for SYMBOL" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "errors": {
        "statement": [
            "Input should be 'income', 'balance' or 'cashflow'"
        ]
    }
  }
  ```

---

### Schema References

#### FinancialStatement Schema

| Field          | Type                             | Description                                                                    | Required |
|----------------|----------------------------------|--------------------------------------------------------------------------------|:--------:|
| symbol         | string                           | Stock symbol (e.g., "AAPL")                                                    |    ✓     |
| statement_type | [StatementType](#statementtype)  | Type of financial statement                                                    |    ✓     |
| frequency      | [Frequency](#frequency)          | Frequency of the financial statement                                           |    ✓     |
| statement      | object                           | Financial statement data, with metrics as keys and a dictionary of dates and values |    ✓     |

#### StatementType

An `Enum` with the following possible string values:
- `income`
- `balance`
- `cashflow`

#### Frequency

An `Enum` with the following possible string values:
- `annual`
- `quarterly`
