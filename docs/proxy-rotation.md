# Proxy & IP Rotation Configuration

FinanceQuery includes a sophisticated proxy rotation system that enables automatic IP rotation across multiple proxies. This is essential for preventing rate limiting when making frequent requests to external APIs and web scraping operations.

## Overview

The proxy rotation system provides:

- **Automatic IP rotation** across multiple proxy endpoints
- **Intelligent proxy selection** with three rotation strategies
- **Health tracking** to avoid problematic proxies
- **Automatic retry** with different proxies on failure
- **BrightData integration** with automatic IP whitelisting
- **Per-request proxy assignment** for optimal distribution

## Why Use Proxy Rotation?

Proxy rotation is crucial when:

- **Rate limiting**: Prevent getting blocked by external APIs due to too many requests from a single IP
- **Web scraping**: Avoid detection patterns when scraping financial data
- **Reliability**: Improve success rates by distributing requests across multiple IPs
- **Compliance**: Meet requirements from data providers that require proxy usage
- **Geographic diversity**: Access region-specific content through different proxy locations

!!! tip "**Best Practice**"
For production deployments making frequent API calls, IP rotation is highly recommended to maintain reliability and avoid rate limits.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `USE_PROXY` | `False` | Enable/disable proxy functionality |
| `PROXY_URL` | `None` | Single proxy URL (format: `http://user:pass@host:port`) |
| `PROXY_POOL` | `None` | Comma-separated list of proxy URLs for rotation |
| `PROXY_ROTATION_STRATEGY` | `round_robin` | Rotation strategy: `round_robin`, `random`, or `weighted` |
| `PROXY_MAX_FAILURES` | `3` | Maximum failures before excluding a proxy |
| `PROXY_TOKEN` | `None` | BrightData API token for automatic IP whitelisting |

### Basic Setup (Single Proxy)

For simple use cases with a single proxy:

```env
USE_PROXY=true
PROXY_URL=http://username:password@proxy.example.com:8080
PROXY_TOKEN=your-brightdata-api-token  # Optional, for BrightData IP whitelisting
```

### Advanced Setup (IP Rotation - Recommended)

For production use with multiple proxies:

```env
USE_PROXY=true
PROXY_POOL=http://proxy1:8080,http://proxy2:8080,http://proxy3:8080
PROXY_ROTATION_STRATEGY=round_robin
PROXY_MAX_FAILURES=3
PROXY_TOKEN=your-brightdata-api-token
```

### BrightData Configuration

When using BrightData proxies, format your URLs as:

```env
USE_PROXY=true
PROXY_POOL=http://brd-customer-ZONE-USERNAME:PASSWORD@zproxy.lum-superproxy.io:22225,http://brd-customer-ZONE-USERNAME:PASSWORD@zproxy.lum-superproxy.io:22226
PROXY_ROTATION_STRATEGY=round_robin
PROXY_MAX_FAILURES=3
PROXY_TOKEN=your-brightdata-api-token
```

!!! note "**Proxy URL Format**"
Proxy URLs should follow the format: `http://username:password@host:port` or `https://username:password@host:port`

## IP Whitelisting

### Automatic IP Whitelisting (BrightData)

When using BrightData proxies with a `PROXY_TOKEN`, the system automatically whitelists your server's IP address on startup:

1. **Detection**: The system detects your server's public IP using `api.ipify.org`
2. **Whitelisting**: Sends a POST request to BrightData's API to add your IP
3. **Cleanup**: Automatically removes the IP from whitelist on application shutdown

This happens automatically during application startup. You'll see logs indicating success or failure:

```
INFO: IP whitelisted successfully
```

Or if it fails:

```
WARNING: IP whitelisting failed or skipped - you may need to manually whitelist your IP in BrightData dashboard
```

### Manual IP Whitelisting

If automatic whitelisting fails or you need to whitelist IPs manually:

#### Using BrightData Dashboard

1. Log in to your [BrightData account](https://brightdata.com)
2. Navigate to **Zones** section
3. Select the zone you're using
4. Go to **Overview** tab
5. In **Access Details** section, click the edit icon
6. Add your server's IP address to the allowlist

#### Using BrightData API

Get your server's IP address:

```bash
curl https://api.ipify.org/
```

Add IP to whitelist:

```bash
curl --request POST \
  --url https://api.brightdata.com/zone/whitelist \
  --header 'Authorization: Bearer YOUR_PROXY_TOKEN' \
  --header 'Content-Type: application/json' \
  --data '{"ip": "YOUR_SERVER_IP"}'
```

Verify whitelist:

```bash
curl --request GET \
  --url https://api.brightdata.com/zone/whitelist \
  --header 'Authorization: Bearer YOUR_PROXY_TOKEN'
```

Remove IP from whitelist:

```bash
curl --request DELETE \
  --url https://api.brightdata.com/zone/whitelist \
  --header 'Authorization: Bearer YOUR_PROXY_TOKEN' \
  --header 'Content-Type: application/json' \
  --data '{"ip": "IP_TO_REMOVE"}'
```

## Rotation Strategies

The proxy rotation system supports three strategies for selecting which proxy to use:

### Round Robin (Default)

Proxies are used in a sequential, rotating order:

```
Request 1 → Proxy 1
Request 2 → Proxy 2
Request 3 → Proxy 3
Request 4 → Proxy 1 (cycles back)
```

**Best for**: Even distribution across all proxies, predictable load balancing.

```env
PROXY_ROTATION_STRATEGY=round_robin
```

### Random

A random proxy is selected from the available pool:

```
Request 1 → Proxy 2 (random)
Request 2 → Proxy 1 (random)
Request 3 → Proxy 3 (random)
```

**Best for**: Unpredictable patterns, avoiding detection, testing proxy performance.

```env
PROXY_ROTATION_STRATEGY=random
```

### Weighted

Proxies are selected based on their success rate. Proxies with higher success rates are more likely to be chosen:

```
Proxy 1: 95% success rate → Higher weight
Proxy 2: 80% success rate → Medium weight
Proxy 3: 60% success rate → Lower weight
```

**Best for**: Optimizing for reliability, automatically favoring better-performing proxies.

```env
PROXY_ROTATION_STRATEGY=weighted
```

## Proxy Health Tracking

The system automatically tracks the health of each proxy:

### Failure Tracking

- Each proxy tracks **success** and **failure** counts
- When a proxy fails, it's marked with a failure
- If failures exceed `PROXY_MAX_FAILURES`, the proxy is temporarily excluded

### Automatic Recovery

- Failed proxies are automatically retried after some time
- Successful requests restore proxies to the active pool
- The system automatically resets failed proxies if all proxies become unavailable

### Proxy Statistics

The system maintains statistics for each proxy:

```python
{
  "http://proxy1:8080": {"success": 150, "failures": 2},
  "http://proxy2:8080": {"success": 145, "failures": 5},
  "http://proxy3:8080": {"success": 148, "failures": 1}
}
```

## How It Works

### Architecture

```
┌─────────────────┐
│  API Request   │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│   ProxyRotator          │
│  - Select next proxy    │
│  - Check health status  │
│  - Track statistics    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│   fetch() function      │
│  - Get proxy from       │
│    ProxyRotator        │
│  - Make HTTP request    │
│  - Mark success/failure │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│   External API/Website  │
└─────────────────────────┘
```

### Request Flow

1. **Request Initiated**: An API endpoint needs to fetch external data
2. **Proxy Selection**: The `fetch()` function requests a proxy from `ProxyRotator`
3. **Strategy Application**: `ProxyRotator` selects a proxy based on configured strategy
4. **Request Execution**: HTTP request is made through the selected proxy
5. **Result Processing**:
   - **Success**: Proxy is marked as successful, statistics updated
   - **Failure**: Proxy failure is recorded, request retries with different proxy

### Automatic Retry

When a request fails:

1. The failed proxy is marked with a failure count
2. If failures exceed threshold, proxy is excluded from rotation
3. Request automatically retries with a different proxy
4. Retry delay increases exponentially (1s, 2s, 4s...)

Example log output:

```
WARNING: Request failed with proxy: http://proxy1:8080
DEBUG: Retrying request after 1.0s with different proxy
DEBUG: Using proxy for request: http://proxy2:8080 (attempt: 2)
DEBUG: Request succeeded: http://proxy2:8080
```

## Deployment Examples

### Docker

```bash
docker run -p 8000:8000 \
  -e USE_PROXY=true \
  -e PROXY_POOL=http://proxy1:8080,http://proxy2:8080,http://proxy3:8080 \
  -e PROXY_ROTATION_STRATEGY=round_robin \
  -e PROXY_MAX_FAILURES=3 \
  -e PROXY_TOKEN=your-brightdata-token \
  finance-query
```

### Environment File

Create a `.env` file:

```env
USE_PROXY=true
PROXY_POOL=http://proxy1:8080,http://proxy2:8080,http://proxy3:8080
PROXY_ROTATION_STRATEGY=round_robin
PROXY_MAX_FAILURES=3
PROXY_TOKEN=your-brightdata-api-token
```

### AWS Lambda / Render

Set environment variables in your platform's configuration:

```env
USE_PROXY=true
PROXY_POOL=http://proxy1:8080,http://proxy2:8080
PROXY_ROTATION_STRATEGY=weighted
PROXY_MAX_FAILURES=5
PROXY_TOKEN=your-token
```

## Verification & Monitoring

### Enable Debug Logging

To see proxy rotation in action, enable debug logging:

```env
LOG_LEVEL=DEBUG
```

You'll see detailed logs:

```
DEBUG: Using proxy for request: http://proxy1:8080, attempt: 1, url: https://finance.yahoo.com/...
DEBUG: Request succeeded: http://proxy1:8080, attempt: 1
DEBUG: Using proxy for request: http://proxy2:8080, attempt: 1, url: https://finance.yahoo.com/...
DEBUG: Request succeeded: http://proxy2:8080, attempt: 1
```

### Startup Logs

On application startup, you'll see:

```
INFO: ProxyRotator initialized, proxy_count: 2, strategy: round_robin, max_failures: 3
INFO: IP whitelisted successfully
INFO: Using ProxyRotator for per-request proxy selection
```

### Testing Proxy Rotation

Make multiple requests to see rotation in action:

```bash
# Request 1 - Should use proxy 1
curl "http://localhost:8000/v1/sectors"

# Request 2 - Should use proxy 2  
curl "http://localhost:8000/v1/sectors"

# Request 3 - Should use proxy 1 (round-robin)
curl "http://localhost:8000/v1/sectors"
```

Check your application logs to verify different proxies are being used.

## Troubleshooting

### Proxy Not Rotating

**Issue**: All requests use the same proxy

**Solutions**:
- Verify `PROXY_POOL` contains multiple comma-separated URLs
- Check `USE_PROXY=true` is set
- Ensure `PROXY_ROTATION_STRATEGY` is set correctly
- Check logs for ProxyRotator initialization messages

### IP Whitelisting Failed

**Issue**: Automatic IP whitelisting fails

**Solutions**:
- Manually whitelist your IP in BrightData dashboard
- Verify `PROXY_TOKEN` is correct
- Check your server's public IP: `curl https://api.ipify.org/`
- Verify network connectivity to BrightData API

### Proxies Failing

**Issue**: All proxies are marked as failed

**Solutions**:
- Check proxy credentials are correct
- Verify proxy endpoints are accessible
- Review `PROXY_MAX_FAILURES` setting (may be too low)
- Check network connectivity from your server
- Review proxy provider's status/downtime

### Requests Not Using Proxies

**Issue**: Requests go through without proxy

**Solutions**:
- Verify `USE_PROXY=true` is set
- Check `PROXY_URL` or `PROXY_POOL` is configured
- Review logs for proxy initialization messages
- Some endpoints use `FinanceClient` which may have different proxy configuration

## Best Practices

### Production Recommendations

1. **Use Multiple Proxies**: Always use `PROXY_POOL` with at least 2-3 proxies
2. **Monitor Health**: Enable debug logging in staging to verify rotation
3. **Set Appropriate Failures**: Adjust `PROXY_MAX_FAILURES` based on your proxy reliability
4. **Use Weighted Strategy**: For production, `weighted` strategy optimizes for reliability
5. **Keep Tokens Secure**: Never commit `PROXY_TOKEN` to version control

### Configuration Tips

- **Round Robin**: Best for even load distribution
- **Random**: Best for avoiding detection patterns
- **Weighted**: Best for maximizing reliability
- **Max Failures**: Start with 3, adjust based on proxy reliability (2-5 recommended)

### Security Considerations

- Store proxy credentials securely (environment variables, secrets management)
- Rotate proxy credentials periodically
- Monitor proxy usage for anomalies
- Use HTTPS proxies when possible
- Keep `PROXY_TOKEN` secret and rotated

## Supported Endpoints

Proxy rotation works with endpoints that use the `fetch()` function:

- ✅ `/v1/news` - News scraping
- ✅ `/v1/sectors` - Sector data scraping
- ✅ `/v1/movers` - Market movers scraping
- ✅ Quote scraping endpoints
- ✅ Similar stocks scraping
- ⚠️ `/v1/quotes` - Uses Yahoo Finance API (may use static proxy)

## Limitations

- **Yahoo Finance API**: Some endpoints using `FinanceClient` may use a static proxy set at client creation
- **Session-level proxies**: When using single proxy mode, proxies are set at session level
- **BrightData only**: Automatic IP whitelisting currently supports BrightData API
- **No proxy persistence**: Proxy stats reset on application restart

## Advanced Configuration

### Custom Proxy Providers

While BrightData is the primary supported provider, you can use any proxy service:

```env
USE_PROXY=true
PROXY_POOL=http://user:pass@custom-proxy1.com:8080,http://user:pass@custom-proxy2.com:8080
```

Just ensure:
- Proxy URLs follow the correct format
- Proxies support HTTP/HTTPS forwarding
- Credentials are valid
- IP whitelisting is handled manually if required

### Programmatic Access

You can access proxy statistics programmatically (if needed for custom endpoints):

```python
from src.utils.proxy_rotator import ProxyRotator

# Get proxy rotator from app state
proxy_rotator = app.state.proxy_rotator

if proxy_rotator:
    # Get statistics
    stats = proxy_rotator.get_stats()
    print(stats)
    
    # Reset statistics
    proxy_rotator.reset_stats()
```

---

For more information on configuration and deployment, see:
- [Getting Started](getting-started.md)
- [Deployment Options](deployment.md)
- [Logging Configuration](logging.md)

