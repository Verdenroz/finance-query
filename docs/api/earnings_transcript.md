# Earnings Transcript API

The Earnings Transcript API provides access to earnings call transcripts in JSON format.

## Overview

This API fetches earnings call transcripts using the `defeatbeta-api` package and returns the raw transcript data in structured JSON format.

## Authentication

All endpoints require an API key passed in the `x-api-key` header.

## Endpoints

### GET /v1/earnings-transcript/{symbol}

Get earnings call transcript for a stock symbol in JSON format.

**Parameters:**
- `symbol` (path): Stock ticker symbol (e.g., AAPL, TSLA, MSFT)
- `quarter` (query, optional): Specific quarter (Q1, Q2, Q3, Q4)
- `year` (query, optional): Specific year (e.g., 2024, 2023)

**Example Request:**
```bash
curl -X GET "https://your-api-domain.com/v1/earnings-transcript/AAPL?quarter=Q3&year=2024" \
  -H "x-api-key: your-api-key"
```

**Example Response:**
```json
{
  "symbol": "AAPL",
  "transcripts": [
    {
      "symbol": "AAPL",
      "quarter": "Q3",
      "year": 2024,
      "date": "2024-10-15T00:00:00",
      "transcript": "Full transcript text...",
      "participants": ["CEO", "CFO", "Analysts"],
      "metadata": {"source": "defeatbeta-api"}
    }
  ],
  "metadata": {
    "total_transcripts": 1,
    "filters_applied": {
      "quarter": "Q3",
      "year": 2024
    },
    "retrieved_at": "2024-10-15T12:00:00"
  }
}
```

### POST /v1/earnings-transcript/analyze

Get earnings transcript with custom parameters via POST request.

**Request Body:**
```json
{
  "symbol": "AAPL",
  "quarter": "Q3",
  "year": 2024
}
```

### GET /v1/earnings-transcript/{symbol}/latest

Get the most recent earnings call transcript for a stock symbol.

**Parameters:**
- `symbol` (path): Stock ticker symbol

**Example Request:**
```bash
curl -X GET "https://your-api-domain.com/v1/earnings-transcript/AAPL/latest" \
  -H "x-api-key: your-api-key"
```

## Response Structure

All endpoints return earnings transcript data with the following structure:

- **symbol**: Stock ticker symbol
- **transcripts**: Array of transcript objects containing:
  - **symbol**: Stock symbol
  - **quarter**: Quarter (e.g., "Q3")
  - **year**: Year (e.g., 2024)
  - **date**: Date of the earnings call
  - **transcript**: Full transcript text
  - **participants**: List of call participants
  - **metadata**: Additional metadata about the transcript
- **metadata**: Request metadata including:
  - **total_transcripts**: Number of transcripts returned
  - **filters_applied**: Applied filters (quarter, year)
  - **retrieved_at**: Timestamp when data was retrieved

## Error Responses

- `404`: Symbol not found or no transcripts available
- `500`: Internal server error (service unavailable)

## Rate Limits

Standard API rate limits apply.

## Dependencies

- `defeatbeta-api`: For fetching earnings transcript data
