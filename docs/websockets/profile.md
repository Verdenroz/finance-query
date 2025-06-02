# Profile WebSocket

The Profile WebSocket endpoint provides comprehensive real-time data for a single stock symbol, including detailed
quotes, similar stocks, sector performance, and relevant news.

## Use Case

Use this endpoint when you need extensive information **for a single stock** and want real-time updates. Perfect for:

- Stock detail pages
- Individual stock monitoring
- Comprehensive stock analysis dashboards

## Connection

### URL Format

- **Production**: `wss://your-domain.com/profile/AAPL`
- **Local**: `ws://localhost:8000/profile/AAPL`

### Parameters

- `symbol` (path parameter) - Stock symbol to monitor (e.g., "AAPL", "GOOGL", "TSLA")

## Usage Examples

### JavaScript

```javascript
const socket = new WebSocket('wss://finance-query.onrender.com/profile/AAPL');

socket.onopen = function (event) {
    console.log('Connected to profile stream for AAPL');
};

socket.onmessage = function (event) {
    const data = JSON.parse(event.data);
    console.log('Profile data:', data);
};

socket.onclose = function (event) {
    console.log('Profile stream closed');
};
```

## Responses

### Response Format

The WebSocket sends JSON responses with the following structure:

```json
{
  "quote": {
    "symbol": "AAPL",
    "name": "Apple Inc.",
    "price": 150.25,
    "change": 2.15,
    "percentChange": 1.45,
    "volume": 45678912,
    "marketCap": 2500000000000,
    "pe": 28.5,
    "eps": 5.26,
    "high": 152.00,
    "low": 148.50,
    "open": 149.00,
    "previousClose": 148.10,
    "fiftyTwoWeekHigh": 180.00,
    "fiftyTwoWeekLow": 120.00
  },
  "similar": [
    {
      "symbol": "MSFT",
      "name": "Microsoft Corporation",
      "price": 340.50,
      "change": -1.25,
      "percentChange": -0.37
    },
    {
      "symbol": "GOOGL",
      "name": "Alphabet Inc.",
      "price": 2650.00,
      "change": 15.30,
      "percentChange": 0.58
    }
  ],
  "sectorPerformance": {
    "sector": "Technology",
    "performance": 1.25,
    "description": "Technology sector performance"
  },
  "news": [
    {
      "title": "Apple Reports Strong Q4 Earnings",
      "url": "https://example.com/news/1",
      "publishedAt": "2024-01-15T10:30:00Z",
      "source": "Financial News",
      "summary": "Apple exceeded expectations..."
    },
    {
      "title": "iPhone Sales Drive Revenue Growth",
      "url": "https://example.com/news/2",
      "publishedAt": "2024-01-15T09:15:00Z",
      "source": "Tech Today",
      "summary": "Strong iPhone demand..."
    }
  ]
}
```

### Response Schema

#### Top-Level Structure

| Field             | Type   | Description                               | Required |
|-------------------|--------|-------------------------------------------|:--------:|
| metadata          | object | API rate limit and usage information      |          |
| quote             | object | Comprehensive quote data for the symbol   |    ✓     |
| similar           | array  | List of similar stocks with basic quotes  |    ✓     |
| sectorPerformance | object | Performance data for the stock's sector   |    ✓     |
| news              | array  | Recent news articles related to the stock |    ✓     |

#### Metadata Schema

!!! info
Only included in the first message after connection and if security is enabled.

| Field              | Type   | Description                             | Required |
|--------------------|--------|-----------------------------------------|:--------:|
| rate_limit         | number | Maximum requests allowed in time period |    ✓     |
| remaining_requests | number | Requests remaining in current period    |    ✓     |
| reset              | number | Seconds until rate limit resets         |    ✓     |

#### Quote Schema

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

#### Similar Quotes Schema

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

#### Sector Performance Schema

| Field           | Type   | Description                    | Required |
|-----------------|--------|--------------------------------|:--------:|
| sector          | string | Sector name                    |    ✓     |
| dayReturn       | string | 1-day sector return percentage |    ✓     |
| ytdReturn       | string | Year-to-date sector return     |    ✓     |
| yearReturn      | string | 1-year sector return           |    ✓     |
| threeYearReturn | string | 3-year sector return           |    ✓     |
| fiveYearReturn  | string | 5-year sector return           |    ✓     |

#### News Item Schema

| Field  | Type   | Description               | Required |
|:-------|:-------|:--------------------------|:--------:|
| title  | string | Title of the news article |    ✓     |
| link   | string | URL to the full article   |    ✓     |
| source | string | News source               |    ✓     |
| img    | string | URL to accompanying image |    ✓     |
| time   | string | Time relative to now      |    ✓     |

## Error Handling

- Invalid symbols will still establish connection but may return null quote data
- Network disconnections trigger automatic cleanup
- Authentication failures prevent connection establishment

### Channel Naming

- **Channel**: Uses `profile:{symbol}` format
- **Example**: For symbol "AAPL", the channel name is `profile:AAPL`
