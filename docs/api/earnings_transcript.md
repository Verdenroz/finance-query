# Earnings Transcript

## GET /v1/earnings-transcript/{symbol}

### Overview

**Purpose:** Retrieve earnings call transcripts for a given stock symbol.
**Response Format:** A transcript data object containing the transcript content and metadata.

### Authentication

Optional authentication via `x-api-key` header token

### Path Parameters

| Parameter | Type   | Required | Description                     | Example |
|-----------|--------|:--------:|---------------------------------|---------|
| `symbol`  | string |    ✓     | The stock ticker symbol         | `AAPL`  |

### Query Parameters

| Parameter | Type    | Required | Description                               | Example |
|-----------|---------|:--------:|-------------------------------------------|---------|
| `quarter` | string  |          | Specific quarter filter (Q1, Q2, Q3, Q4) | `Q3`    |
| `year`    | integer |          | Specific year filter                      | `2024`  |

---

### Responses

- **200 OK**
  - **Content-Type:** `application/json`
  - **Schema:** [`EarningsTranscript`](#earningstranscript-schema)
  - **Example (200):**
    ```json
    {
      "symbol": "AAPL",
      "transcripts": [
        {
          "symbol": "AAPL",
          "quarter": "Q3",
          "year": 2024,
          "date": "2024-10-15T00:00:00",
          "transcript": "AAPL Q3 2024 Earnings Call Transcript\n\nCORPORATE PARTICIPANTS:\n- CEO: Thank you for joining us today...",
          "participants": [
            "CEO - Chief Executive Officer",
            "CFO - Chief Financial Officer",
            "Analyst 1 - Investment Research"
          ],
          "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-10-15T12:00:00",
            "transcripts_id": 12345
          }
        }
      ]
    }
    ```

- **404 Not Found**
  ```json
  { "detail": "No earnings transcripts found for symbol INVALID" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "errors": {
      "quarter": [
        "Invalid quarter format. Use Q1, Q2, Q3, or Q4"
      ]
    }
  }
  ```

---

### Schema References

#### EarningsTranscript Schema

| Field       | Type                                    | Description                          | Required |
|-------------|-----------------------------------------|--------------------------------------|:--------:|
| symbol      | string                                  | Stock symbol (e.g., "AAPL")         |    ✓     |
| transcripts | [TranscriptItem[]](#transcriptitem)     | List of transcript objects           |    ✓     |

#### TranscriptItem

| Field        | Type     | Description                          | Required |
|--------------|----------|--------------------------------------|:--------:|
| symbol       | string   | Stock symbol                         |    ✓     |
| quarter      | string   | Quarter (e.g., "Q3")                 |    ✓     |
| year         | integer  | Year (e.g., 2024)                    |    ✓     |
| date         | datetime | Date of the earnings call            |    ✓     |
| transcript   | string   | Full transcript text content         |    ✓     |
| participants | string[] | List of call participants            |    ✓     |
| metadata     | object   | Additional transcript metadata       |    ✓     |

---

### Usage Examples

#### Get Latest Earnings Transcript
```bash
curl -X GET "https://finance-query.onrender.com/v1/earnings-transcript/AAPL" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Get Specific Quarter and Year
```bash
curl -X GET "https://finance-query.onrender.com/v1/earnings-transcript/TSLA?quarter=Q3&year=2024" \
     -H "x-api-key: YOUR_API_KEY"
```

#### Filter by Year Only
```bash
curl -X GET "https://finance-query.onrender.com/v1/earnings-transcript/MSFT?year=2023" \
     -H "x-api-key: YOUR_API_KEY"
```
