# Holders

## GET /v1/holders/{symbol}

### Overview

**Purpose:** Retrieve holders information for a given stock symbol.  
**Response Format:** A holders data object containing the requested holder type information.

### Authentication

Optional authentication via `x-api-key` header token

### Path Parameters

| Parameter | Type   | Required | Description                     | Example |
|-----------|--------|:--------:|---------------------------------|---------|
| `symbol`  | string |    ✓     | The stock ticker symbol         | `AAPL`  |

### Query Parameters

| Parameter     | Type       | Required | Description                               | Example         |
|---------------|------------|:--------:|-------------------------------------------|-----------------|
| `holder_type` | HolderType |    ✓     | The type of holders data to retrieve      | `institutional` |

---

### Responses

- **200 OK**  
  - **Content-Type:** `application/json`  
  - **Schema:** [`HoldersData`](#holdersdata-schema)
  - **Example (200) - Institutional Holders:**
    ```json
    {
      "symbol": "AAPL",
      "holder_type": "institutional",
      "major_breakdown": null,
      "institutional_holders": [
        {
          "holder": "Vanguard Group Inc",
          "shares": 1311658000,
          "date_reported": "2024-03-31T00:00:00",
          "percent_out": 8.44,
          "value": 224234567000
        },
        {
          "holder": "BlackRock Inc.",
          "shares": 1038225000,
          "date_reported": "2024-03-31T00:00:00",
          "percent_out": 6.68,
          "value": 177456789000
        }
      ],
      "mutualfund_holders": null,
      "insider_transactions": null,
      "insider_purchases": null,
      "insider_roster": null
    }
    ```

  - **Example (200) - Major Holders Breakdown:**
    ```json
    {
      "symbol": "AAPL",
      "holder_type": "major",
      "major_breakdown": {
        "breakdown_data": {
          "institutionsPercentHeld": 0.595,
          "insidersPercentHeld": 0.0007,
          "institutionsFloatPercentHeld": 0.596,
          "institutionsCount": 5743
        }
      },
      "institutional_holders": null,
      "mutualfund_holders": null,
      "insider_transactions": null,
      "insider_purchases": null,
      "insider_roster": null
    }
    ```

  - **Example (200) - Insider Transactions:**
    ```json
    {
      "symbol": "AAPL",
      "holder_type": "insider_transactions",
      "major_breakdown": null,
      "institutional_holders": null,
      "mutualfund_holders": null,
      "insider_transactions": [
        {
          "start_date": "2024-02-01T00:00:00",
          "insider": "COOK TIMOTHY D",
          "position": "Chief Executive Officer",
          "transaction": "Sale at price 191.12 - 195.33 per share.",
          "shares": 511000,
          "value": 98620000,
          "ownership": "D"
        }
      ],
      "insider_purchases": null,
      "insider_roster": null
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
        "holder_type": [
            "Input should be 'major', 'institutional', 'mutualfund', 'insider_transactions', 'insider_purchases' or 'insider_roster'"
        ]
    }
  }
  ```

---

### Schema References

#### HoldersData Schema

| Field                | Type                                              | Description                                          | Required |
|----------------------|---------------------------------------------------|------------------------------------------------------|:--------:|
| symbol               | string                                            | Stock symbol (e.g., "AAPL")                         |    ✓     |
| holder_type          | [HolderType](#holdertype)                         | Type of holders data                                 |    ✓     |
| major_breakdown      | [MajorHoldersBreakdown](#majorholdersbreakdown)   | Major holders breakdown (when holder_type=major)    |          |
| institutional_holders| [InstitutionalHolder[]](#institutionalholder)     | List of institutional holders                        |          |
| mutualfund_holders   | [MutualFundHolder[]](#mutualfundholder)           | List of mutual fund holders                          |          |
| insider_transactions | [InsiderTransaction[]](#insidertransaction)       | List of insider transactions                         |          |
| insider_purchases    | [InsiderPurchase](#insiderpurchase)               | Insider purchase activity summary                    |          |
| insider_roster       | [InsiderRosterMember[]](#insiderrostermember)      | List of insider roster members                       |          |

#### HolderType

An `Enum` with the following possible string values:
- `major` - Major holders breakdown
- `institutional` - Institutional holders list
- `mutualfund` - Mutual fund holders list
- `insider_transactions` - Insider transaction history
- `insider_purchases` - Insider purchase activity summary
- `insider_roster` - Current insider roster

#### MajorHoldersBreakdown

| Field          | Type   | Description                              | Required |
|----------------|--------|------------------------------------------|:--------:|
| breakdown_data | object | Key-value pairs of major holder metrics |    ✓     |

#### InstitutionalHolder

| Field          | Type     | Description                           | Required |
|----------------|----------|---------------------------------------|:--------:|
| holder         | string   | Institution name                      |    ✓     |
| shares         | integer  | Number of shares held                 |    ✓     |
| date_reported  | datetime | Date of last report                   |    ✓     |
| percent_out    | number   | Percentage of outstanding shares      |          |
| value          | integer  | Value of holdings                     |          |

#### MutualFundHolder

| Field          | Type     | Description                           | Required |
|----------------|----------|---------------------------------------|:--------:|
| holder         | string   | Fund name                             |    ✓     |
| shares         | integer  | Number of shares held                 |    ✓     |
| date_reported  | datetime | Date of last report                   |    ✓     |
| percent_out    | number   | Percentage of outstanding shares      |          |
| value          | integer  | Value of holdings                     |          |

#### InsiderTransaction

| Field       | Type     | Description                         | Required |
|-------------|----------|-------------------------------------|:--------:|
| start_date  | datetime | Transaction start date              |    ✓     |
| insider     | string   | Insider name                        |    ✓     |
| position    | string   | Insider position/relation           |    ✓     |
| transaction | string   | Transaction description             |    ✓     |
| shares      | integer  | Number of shares                    |          |
| value       | integer  | Transaction value                   |          |
| ownership   | string   | Ownership type (direct/indirect)    |          |

#### InsiderPurchase

| Field                        | Type   | Description                             | Required |
|------------------------------|--------|-----------------------------------------|:--------:|
| period                       | string | Time period for the data                |    ✓     |
| purchases_shares             | integer| Shares purchased                        |          |
| purchases_transactions       | integer| Number of purchase transactions         |          |
| sales_shares                 | integer| Shares sold                             |          |
| sales_transactions           | integer| Number of sale transactions             |          |
| net_shares                   | integer| Net shares purchased/sold               |          |
| net_transactions             | integer| Net transactions                        |          |
| total_insider_shares         | integer| Total insider shares held               |          |
| net_percent_insider_shares   | number | Net % of insider shares                 |          |
| buy_percent_insider_shares   | number | % buy shares                            |          |
| sell_percent_insider_shares  | number | % sell shares                           |          |

#### InsiderRosterMember

| Field                     | Type     | Description                          | Required |
|---------------------------|----------|--------------------------------------|:--------:|
| name                      | string   | Insider name                         |    ✓     |
| position                  | string   | Position/relation                    |    ✓     |
| most_recent_transaction   | string   | Most recent transaction              |          |
| latest_transaction_date   | datetime | Latest transaction date              |          |
| shares_owned_directly     | integer  | Shares owned directly                |          |
| shares_owned_indirectly   | integer  | Shares owned indirectly              |          |
| position_direct_date      | datetime | Position direct date                 |          |

---

### Usage Examples

#### Get Institutional Holders
```bash
curl -X GET "https://finance-query.onrender.com/v1/holders/AAPL?holder_type=institutional" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Major Holders Breakdown
```bash
curl -X GET "https://finance-query.onrender.com/v1/holders/MSFT?holder_type=major" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Mutual Fund Holders
```bash
curl -X GET "https://finance-query.onrender.com/v1/holders/GOOGL?holder_type=mutualfund" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Insider Transactions
```bash
curl -X GET "https://finance-query.onrender.com/v1/holders/TSLA?holder_type=insider_transactions" \
     -H "x-api-key: YOUR_API_KEY"
```
