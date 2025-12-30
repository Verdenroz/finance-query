# WebSockets

Finance Query provides real-time financial data through WebSocket connections. All WebSocket endpoints automatically refresh data every 5 seconds and support both individual connections and Redis-based pub/sub scaling.

## Available WebSocket Routes

Finance Query currently implements four WebSocket endpoints:

- **[Profile](websockets/profile.md)** - Stream comprehensive data for a single symbol including quotes, similar stocks, sector performance, and news
- **[Quotes](websockets/quotes.md)** - Stream simple quote data for multiple symbols simultaneously
- **[Market](websockets/market.md)** - Stream market overview including movers, indices, news, and sector performances
- **[Hours](websockets/hours.md)** - Stream current market status (open, closed, pre-market, after-hours)

## Connection Requirements

### Authentication
All WebSocket connections require validation if the `USE_SECURITY` environment variable is set to `True`. This ensures that only authenticated users can access the data streams.

### URL Format
WebSocket URLs follow this pattern:

- **Production/HTTPS**: `wss://your-domain.com/{endpoint}`
- **Local Development**: `ws://localhost:8000/{endpoint}`

## Deployment Considerations

### AWS Lambda Limitations
WebSockets are **not supported** when deploying Finance Query to AWS Lambda due to Lambda's stateless nature and lack of persistent connections.

### Recommended Deployment
For full WebSocket functionality, deploy to platforms that support persistent connections:

- **Render** (recommended)
- **Railway**
- **Fly.io**
- **DigitalOcean App Platform**
- **Traditional VPS/servers**

## Connection Management

### Redis Scaling
Finance Query supports Redis-based connection management for horizontal scaling:

- Multiple server instances can share WebSocket state
- Pub/sub mechanism ensures all connected clients receive updates
- Automatic fallback to in-memory management if Redis is unavailable

### Connection Lifecycle
1. **Validation** - Each connection is validated before acceptance (if `USE_SECURITY` is `True`)
2. **Initial Data** - Immediate data fetch and send upon connection
3. **Periodic Updates** - Data refreshed every 5 seconds
4. **Graceful Disconnection** - Proper cleanup on client disconnect

## Error Handling
- Invalid authentication results in connection rejection
- Failed data fetches don't terminate the connection
- Default null values provided for failed operations

## Connection Management

### Channel Naming
- **Connection Pooling**: Multiple clients may share data channel
- **Redis Support**: Horizontal scaling via Redis pub/sub

### Connection Sharing
- Reduces server load for popular symbol combinations
- Redis pub/sub enables scaling across multiple server instances


## Getting Started

1. **Choose your endpoint** based on data requirements
2. **Establish connection** using appropriate URL format
3. **Handle initial data** sent immediately upon connection
4. **Process periodic updates** sent every 5 seconds
5. **Implement reconnection logic** for production applications

For detailed usage examples and response formats, see the individual endpoint documentation pages.
