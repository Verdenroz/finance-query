# Rate Limiting Configuration

FinanceQuery includes built-in rate limiting to prevent abuse and ensure fair usage of the API. This document covers how to configure and understand the rate limiting system.

## Overview

The rate limiting system provides:

- **Daily request limits** per IP address
- **Configurable limits** via environment variables
- **Admin bypass** for authenticated admin users
- **Real-time headers** showing current usage
- **Health check protection** with separate limits

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `USE_SECURITY` | `False` | Enable/disable rate limiting middleware |
| `APP_RATE_LIMIT_PER_DAY` | `8000` | Daily request limit per IP address |
| `ADMIN_API_KEY` | `None` | Admin API key (bypasses rate limits) |

### Basic Setup

Enable rate limiting by setting the security environment variable:

```bash
# Enable rate limiting with default 8000 requests/day
USE_SECURITY=True

# Set custom daily limit
APP_RATE_LIMIT_PER_DAY=5000

# Optional: Set admin key for unlimited access
ADMIN_API_KEY=your-secret-admin-key
```

## Deployment Configuration

### Docker

Configure rate limiting when running the Docker container:

```bash
# Basic rate limiting
docker run -p 8000:8000 \
  -e USE_SECURITY=True \
  -e APP_RATE_LIMIT_PER_DAY=10000 \
  financequery

# With admin access
docker run -p 8000:8000 \
  -e USE_SECURITY=True \
  -e APP_RATE_LIMIT_PER_DAY=5000 \
  -e ADMIN_API_KEY=your-secret-key \
  financequery
```

### AWS Lambda

Set environment variables in your Lambda function configuration:

```bash
# Build with custom rate limit
docker build -f Dockerfile.aws \
  --build-arg APP_RATE_LIMIT_PER_DAY=15000 \
  -t financequery-lambda .

# Or set at runtime
docker run -e APP_RATE_LIMIT_PER_DAY=15000 \
  -e USE_SECURITY=True \
  financequery-lambda
```

### Environment File

Add to your `.env` file:

```env
# Enable security and rate limiting
USE_SECURITY=True

# Set daily rate limit (default: 8000)
APP_RATE_LIMIT_PER_DAY=8000

# Optional admin key for unlimited access
ADMIN_API_KEY=your-secret-admin-key
```

## API Behavior

### Rate Limit Headers

When rate limiting is active, all API responses include headers showing current usage:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 8000
X-RateLimit-Remaining: 7999
X-RateLimit-Reset: 86399
Content-Type: application/json
```

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Total requests allowed per day |
| `X-RateLimit-Remaining` | Requests remaining in current period |
| `X-RateLimit-Reset` | Seconds until limit resets |

### Rate Limit Exceeded

When the daily limit is exceeded, the API returns:

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{
  "detail": "Rate limit exceeded",
  "rate_limit_info": {
    "count": 8000,
    "remaining": 0,
    "reset_in": 43200,
    "limit": 8000
  }
}
```

### Admin Access

Requests with a valid admin API key bypass rate limits:

```bash
# Unlimited requests with admin key
curl -H "x-api-key: your-secret-admin-key" \
  "https://your-api.com/v1/quotes?symbols=AAPL"
```

Admin requests receive no rate limit headers since they're not subject to limits.

## Protected Endpoints

### Open Paths (No Rate Limiting)

These endpoints are always accessible without rate limiting:
- `/ping` - Basic health check
- `/docs` - API documentation
- `/openapi.json` - OpenAPI specification
- `/redoc` - Alternative API documentation

### Health Check Rate Limiting

The `/health` endpoint has separate rate limiting (30-minute cooldown per IP) to prevent health check abuse while allowing monitoring systems to function.

### All Other Endpoints

All API endpoints under `/v1/` are subject to rate limiting when `USE_SECURITY=True`.
