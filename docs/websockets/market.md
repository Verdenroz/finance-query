# Market WebSocket

The Market WebSocket endpoint provides a comprehensive real-time overview of the financial markets, including market movers, major indices, general market news, and sector performance data.

## Use Case

Use this endpoint when you need a **complete market overview** for dashboards or market analysis applications. Perfect for:

- Market overview dashboards
- Trading platform home pages  
- Financial news applications
- Market analysis tools
- Portfolio context information

## Connection

### URL Format
- **Production**: `wss://your-domain.com/market`
- **Local**: `ws://localhost:8000/market`

## Usage Examples

### JavaScript
```javascript
const socket = new WebSocket('wss://finance-query.onrender.com/market');

socket.onopen = function(event) {
    console.log('Connected to market overview stream');
};

socket.onmessage = function(event) {
    const marketData = JSON.parse(event.data);
    console.log('Market data update:', marketData);
};

socket.onclose = function(event) {
    console.log('Market stream closed');
};
```

## Responses

### Response Format

The WebSocket sends comprehensive market data in the following JSON structure:

```json
{
    "actives": [
        {
            "symbol": "AAPL",
            "name": "Apple Inc.",
            "price": 150.25,
            "change": 2.15,
            "percentChange": 1.45,
            "volume": 45678912
        },
        {
            "symbol": "TSLA",
            "name": "Tesla Inc.",
            "price": 850.30,
            "change": -15.75,
            "percentChange": -1.82,
            "volume": 32145678
        }
    ],
    "gainers": [
        {
            "symbol": "NVDA",
            "name": "NVIDIA Corporation",
            "price": 420.50,
            "change": 25.30,
            "percentChange": 6.40,
            "volume": 15234567
        }
    ],
    "losers": [
        {
            "symbol": "NFLX",
            "name": "Netflix Inc.",
            "price": 380.25,
            "change": -18.75,
            "percentChange": -4.70,
            "volume": 8765432
        }
    ],
    "indices": [
        {
            "symbol": "^GSPC",
            "name": "S&P 500",
            "price": 4250.15,
            "change": 12.50,
            "percentChange": 0.29
        },
        {
            "symbol": "^DJI", 
            "name": "Dow Jones",
            "price": 34150.25,
            "change": -45.30,
            "percentChange": -0.13
        },
        {
            "symbol": "^IXIC",
            "name": "NASDAQ",
            "price": 13500.75,
            "change": 85.20,
            "percentChange": 0.63
        }
    ],
    "headlines": [
        {
            "title": "Federal Reserve Maintains Interest Rates",
            "url": "https://example.com/news/fed-rates",
            "publishedAt": "2024-01-15T14:30:00Z",
            "source": "Financial Times",
            "summary": "The Federal Reserve decided to keep rates unchanged..."
        },
        {
            "title": "Tech Stocks Rally on AI Optimism",
            "url": "https://example.com/news/tech-rally",
            "publishedAt": "2024-01-15T13:15:00Z",
            "source": "MarketWatch",
            "summary": "Technology stocks surged following positive AI developments..."
        }
    ],
    "sectors": [
        {
            "sector": "Technology",
            "performance": 1.25,
            "change": 0.35,
            "description": "Technology sector showing strong performance"
        },
        {
            "sector": "Healthcare",
            "performance": -0.45,
            "change": -0.12,
            "description": "Healthcare sector declining amid regulatory concerns"
        },
        {
            "sector": "Energy",
            "performance": 2.15,
            "change": 0.89,
            "description": "Energy sector boosted by oil price increases"
        }
    ]
}
```

### Response Schema

#### Top-Level Structure

| Field     | Type  | Description                            | Required |
|-----------|-------|----------------------------------------|:--------:|
| actives   | array | Most actively traded stocks by volume  |    ✓     |
| gainers   | array | Top performing stocks by percentage    |    ✓     |
| losers    | array | Worst performing stocks by percentage  |    ✓     |
| indices   | array | Major market indices                   |    ✓     |
| headlines | array | General financial and economic news    |    ✓     |
| sectors   | array | Performance of major market sectors    |    ✓     |

#### Market Mover Item Schema

| Field         | Type   | Description                    | Required |
|---------------|--------|--------------------------------|:--------:|
| symbol        | string | Stock ticker symbol            |    ✓     |
| name          | string | Company name                   |    ✓     |
| price         | number | Current stock price            |    ✓     |
| change        | number | Absolute price change          |    ✓     |
| percentChange | number | Percentage price change        |    ✓     |
| volume        | number | Trading volume                 |    ✓     |

#### Index Item Schema

| Field         | Type   | Description                    | Required |
|---------------|--------|--------------------------------|:--------:|
| symbol        | string | Index symbol                   |    ✓     |
| name          | string | Index name                     |    ✓     |
| price         | number | Current index value            |    ✓     |
| change        | number | Absolute value change          |    ✓     |
| percentChange | number | Percentage value change        |    ✓     |

#### Headline Item Schema

| Field       | Type   | Description                    | Required |
|-------------|--------|--------------------------------|:--------:|
| title       | string | News article title             |    ✓     |
| url         | string | Link to full article           |    ✓     |
| publishedAt | string | Publication timestamp (ISO)    |    ✓     |
| source      | string | News source name               |    ✓     |
| summary     | string | Brief article summary          |    ✓     |

#### Sector Item Schema

| Field       | Type   | Description                    | Required |
|-------------|--------|--------------------------------|:--------:|
| sector      | string | Sector name                    |    ✓     |
| performance | number | Overall sector performance     |    ✓     |
| change      | number | Day-to-day change              |    ✓     |
| description | string | Brief sector status summary    |    ✓     |

## Error Handling

- Individual data sources may fail independently
- Failed sources return empty arrays
- Successful sources continue to provide data

## Channel Naming

- **Channel**: Always uses the channel name `hours`
