# Quotes WebSocket

The Quotes WebSocket endpoint provides real-time simple quote data for multiple stock symbols simultaneously. This
endpoint is optimized for high-performance scenarios where you need basic quote information for many stocks.

## Use Case

Use this endpoint when you need basic quote information **for multiple stocks** with minimal bandwidth usage. Perfect
for:

- Stock watchlists
- Portfolio monitoring
- Market overview displays
- Trading dashboards

## Connection

### URL Format

- **Production**: `wss://your-domain.com/quotes`
- **Local**: `ws://localhost:8000/quotes`

!!! warning "Two-Step Connection Required"
Unlike other WebSocket endpoints, the quotes endpoint requires a two-step connection process:

1. **Establish Connection** - Connect to the WebSocket endpoint
2. **Send Symbols** - Send a comma-separated list of stock symbols

## Usage Examples

### JavaScript

```javascript
const socket = new WebSocket('wss://finance-query.onrender.com/quotes');

socket.onopen = function (event) {
    console.log('Connected to quotes stream');

    // Send the symbols you want to monitor
    socket.send('AAPL,GOOGL,MSFT,TSLA,AMZN');
};

socket.onmessage = function (event) {
    const quotes = JSON.parse(event.data);
    console.log('Quotes update:', quotes);
};

socket.onclose = function (event) {
    console.log('Quotes stream closed');
};
```

## Symbol Format

### Sending Symbols

- **Format**: Comma-separated string
- **Case**: Symbols are automatically converted to uppercase
- **Duplicates** Symbols are automatically deduplicated

## Responses

### Response Format

The WebSocket sends JSON arrays with simplified quote objects:

```json
[
  {
    "symbol": "AAPL",
    "name": "Apple Inc.",
    "price": "150.25",
    "change": 2.15,
    "percentChange": 1.45,
    "preMarketPrice": "151.00",
    "afterHoursPrice": "149.80",
    "logo": "https://example.com/logos/aapl.png"
  },
  {
    "symbol": "GOOGL",
    "name": "Alphabet Inc.",
    "price": "2650.00",
    "change": -15.30,
    "percentChange": -0.57,
    "logo": "https://example.com/logos/googl.png"
  },
  {
    "symbol": "MSFT",
    "name": "Microsoft Corporation",
    "price": "340.50",
    "change": 5.25,
    "percentChange": 1.56,
    "preMarketPrice": "341.00"
  }
]
```

### Response Schema

| Field           | Type   | Description               | Required |
|-----------------|--------|---------------------------|:--------:|
| symbol          | string | Stock ticker symbol       |    ✓     |
| name            | string | Company name              |    ✓     |
| price           | string | Current stock price       |    ✓     |
| change          | number | Absolute price change     |    ✓     |
| percentChange   | number | Percentage change         |    ✓     |
| preMarketPrice  | string | Pre-market trading price  |          |
| afterHoursPrice | string | After-hours trading price |          |
| logo            | string | Company logo URL          |          |

## Error Handling

- Invalid symbols are silently ignored
- Valid symbols in the list are still processed
- No error messages for invalid symbols

## Channel Naming

- **Channel**: Uses the exact symbol list as channel name
- **Example**: `"AAPL,GOOGL,MSFT"` becomes channel `AAPL,GOOGL,MSFT`

