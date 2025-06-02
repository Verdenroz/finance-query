# Hours WebSocket

The Hours WebSocket endpoint provides real-time market status information, indicating whether markets are currently open, closed, in pre-market, or after-hours trading sessions.

## Use Case

Use this endpoint when you need to **track market hours and status** for:

- Trading application status indicators
- Conditional data fetching based on market state
- User notifications about market status
- Trading hour restrictions
- Market countdown timers

## Connection

### URL Format
- **Production**: `wss://your-domain.com/hours`
- **Local**: `ws://localhost:8000/hours`

### Parameters
No parameters required - provides general market status.

## Usage Examples

### JavaScript

```javascript
const socket = new WebSocket('wss://finance-query.onrender.com/hours');

socket.onopen = function(event) {
    console.log('Connected to market hours stream');
};

socket.onmessage = function(event) {
    const statusData = JSON.parse(event.data);
    console.log('Market status:', statusData);
};

socket.onclose = function(event) {
    console.log('Market hours stream closed');
};
```

## Responses

### Response Format

The WebSocket sends JSON responses with market status information:

```json
{
    "status": "OPEN",
    "reason": "Regular trading hours",
    "timestamp": "2024-01-15T14:30:00.000Z"
}
```

### Response Schema

| Field     | Type   | Description                         | Required |
|-----------|--------|-------------------------------------|:--------:|
| status    | string | Market status indicator             |    ✓     |
| reason    | string | Description of current market state |    ✓     |
| timestamp | string | ISO 8601 timestamp of status update |    ✓     |

## Error Handling

- Connection errors return standard WebSocket error events

## Channel Naming

- **Channel**: Always uses the channel name `hours`
