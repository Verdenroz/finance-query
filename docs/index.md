<h1 align="center">FinanceQuery</h1>

<p align="center">
  <img src="assets/logo.png" alt="FinanceQuery" width="187">
</p>

[![Tests](https://github.com/Verdenroz/finance-query/actions/workflows/tests.yml/badge.svg)](https://github.com/Verdenroz/finance-query/actions/workflows/tests.yml)
[![codecov](https://codecov.io/gh/Verdenroz/finance-query/graph/badge.svg?token=0S3003BAZY)](https://codecov.io/gh/Verdenroz/finance-query)
[![AWS Deploy](https://img.shields.io/github/actions/workflow/status/Verdenroz/finance-query/aws-deploy.yml?branch=master&logo=amazon-aws&label=AWS%20Deploy)](https://github.com/Verdenroz/finance-query/actions/workflows/aws-deploy.yml)
[![Render Deploy](https://img.shields.io/github/actions/workflow/status/Verdenroz/finance-query/render-deploy.yml?branch=master&logo=render&label=Render%20Deploy)](https://github.com/Verdenroz/finance-query/actions/workflows/render-deploy.yml)
[![Code style: ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)
[![Python 3.11+](https://img.shields.io/badge/python-3.11+-blue.svg)](https://www.python.org/downloads/)
[![FastAPI](https://img.shields.io/badge/FastAPI-005571?style=flat&logo=fastapi)](https://fastapi.tiangolo.com)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**FinanceQuery** is an open-source API for financial data that provides real-time quotes, market data, news, and
technical indicators.
It sources data from the unofficial Yahoo Finance API, web scraping, and other financial data providers.

The key features are:

* **Fast**: Very high performance, on par with NodeJS and Go (thanks to FastAPI and Starlette). One of the fastest
  Python frameworks available for financial data.
* **Robust**: Get production-ready code. With automatic interactive documentation.
* **Standards-based**: Based on (and fully compatible with) the open standards for APIs: OpenAPI and JSON Schema.

---

!!! success "**Free & Open Source**"
FinanceQuery is completely free and open-source. No hidden fees, no rate limits on your own deployment. Built by
developers, for developers.

!!! tip "**Production Ready**"
Deploy to AWS Lambda, Render, or any cloud provider. Includes Docker support, automatic documentation, and comprehensive
testing.

!!! info "**Real-Time Data**"
Get live stock quotes, market data, and financial news through WebSocket connections and REST APIs. Perfect for trading
applications and financial dashboards.
---

## Getting Started

For requirements, installation instructions and quick start guide, see [Getting Started](getting-started.md).

## Interactive API Documentation

For a live interactive API documentation with demo requests,
visit [Scalar FinanceQuery](https://financequery.apidocumentation.com/reference).

---

## Example Usage

A demo API is ready to use out of the box. Here's how to get stock data:

#### REST API Example

```bash
# Get detailed quote for NVIDIA stock
curl -X GET 'https://finance-query.onrender.com/v1/quotes?symbols=nvda' \
  -H 'x-api-key: your-api-key'
```

#### Response

```json
[
  {
    "symbol": "NVDA",
    "name": "NVIDIA Corporation",
    "price": "120.15",
    "change": "-11.13",
    "percentChange": "-8.48%",
    "marketCap": "2.94T",
    "sector": "Technology",
    "industry": "Semiconductors"
  }
]
```

### WebSocket Real-Time Updates

```javascript
// Connect to WebSocket for real-time updates
const ws = new WebSocket('wss://finance-query.onrender.com/quotes');

ws.onopen = () => {
    console.log('Connected to FinanceQuery WebSocket');
    ws.send('TSLA'); // Subscribe to Tesla updates
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Real-time update:', data);
};
```

---

## What You Get

In summary, you declare your financial data needs once, and **FinanceQuery** provides:

- ✅ **Data validation**:
    - Automatic and clear errors when requests are invalid
    - Validation for stock symbols, date ranges, and parameters
    - Support for complex nested financial data structures

- ✅ **Multiple data sources**:
    - Yahoo Finance integration
    - Web scraping for additional data points and fallbacks
    - Real-time and historical data support

- ✅ **Performance optimizations**:
    - Cython-accelerated technical indicators
    - Redis caching for frequently requested data
    - Efficient data processing and serialization

- ✅ **Production features**:
    - Rate limiting and API key management
    - Proxy support for reliable data fetching
    - Docker containerization
    - Cloud deployment ready (AWS Lambda, Render)

---

## Available REST Endpoints

| Endpoint                              | Description                                    |
|---------------------------------------|------------------------------------------------|
| [`/health`, `/ping`](api/health.md)   | API status and health monitoring               |
| [`/hours`](api/hours.md)              | Trading hours and market status                |
| [`/v1/quotes`](api/quotes.md)         | Detailed quotes and information                |
| [`/v1/simple-quotes`](api/quotes.md)  | Simplified quotes with summary information     |
| [`/v1/similar`](api/quotes.md)        | Find similar quotes to queried symbol          |
| [`/v1/historical`](api/historical.md) | Historical price data with customizable ranges |
| [`/v1/movers`](api/movers.md)         | Market gainers, losers, and most active stocks |
| [`/v1/news`](api/news.md)             | Financial news and market updates              |
| [`/v1/indices`](api/indices.md)       | Major market indices (S&P 500, NASDAQ, DOW)    |
| [`/v1/sectors`](api/sectors.md)       | Market sector performance and analysis         |
| [`/v1/search`](api/search.md)         | Search for securities with filters             |
| [`/v1/indicator`](api/indicators.md)  | Get specific indicator history over time       |
| [`/v1/indicators`](api/indicators.md) | Technical indicators summary for interval      |
| [`/v1/stream`](api/stream.md)         | SSE for real-time quote updates                |

## Available WebSocket Endpoints

| Endpoint   | Description                                               |
|------------|-----------------------------------------------------------|
| `/quotes`  | Real-time quotes updates                                  |
| `/profile` | Real-time detailed ticker updates (quote, news, similar)  |
| `/market`  | Real-time market updates (indices, news, movers, sectors) |
| `/hours`   | Real-time market hour updates                             |

---

## Deployment Options

!!! abstract "**Multiple Deployment Options**"

    === "AWS Lambda"
        Perfect for serverless applications with automatic scaling:
        ```bash
        # Use the provided AWS deployment workflow
        # Add AWS_SECRET_ID and AWS_SECRET_KEY to repository secrets
        ```

    === "Render"
        Easy deployment with WebSocket support:
        ```bash
        # Deploy using the Render workflow
        # Add RENDER_DEPLOY_HOOK_URL to repository secrets
        ```

    === "Docker"
        Deploy anywhere with Docker:
        ```bash
        docker build -t financequery .
        docker run -p 8000:8000 financequery
        ```

---

## Configuration

Customize FinanceQuery with environment variables:

These environment variables are optional. The API will function with default settings if not provided.

!!! settings "**Security Configuration**"
```env
USE_SECURITY=true
ADMIN_API_KEY=your-secret-admin-key
```

!!! settings "**Proxy Configuration**"
```env
USE_PROXY=true
PROXY_URL=your-proxy-url
# Or use multiple proxies for IP rotation:
PROXY_POOL=http://proxy1:8080,http://proxy2:8080,http://proxy3:8080
PROXY_ROTATION_STRATEGY=round_robin  # Options: round_robin, random, weighted
PROXY_MAX_FAILURES=3  # Max failures before marking proxy as dead
PROXY_TOKEN=your-proxy-token
```

**Proxy Rotation:**
- `PROXY_POOL`: Comma-separated list of proxy URLs for IP rotation (recommended for preventing rate limits)
- `PROXY_URL`: Single proxy URL (backward compatible, used if PROXY_POOL not set)
- `PROXY_ROTATION_STRATEGY`: 
  - `round_robin`: Rotate through proxies sequentially (default)
  - `random`: Select proxy randomly
  - `weighted`: Prefer proxies with higher success rates
- `PROXY_MAX_FAILURES`: Maximum failures before automatically excluding a proxy (default: 3)

!!! settings "**Redis Caching**"
```env
REDIS_URL=redis://localhost:6379
```

!!! settings "**Algolia Search**"
```env
ALGOLIA_APP_ID=your-algolia-app-id
ALGOLIA_API_KEY=your-algolia-api-key
```

---

## Performance

FinanceQuery leverages:

- **[FastAPI](https://fastapi.tiangolo.com)** for lightning-fast HTTP performance
- **[fastapi-injectable](https://github.com/JasperSui/fastapi-injectable)** for efficient dependency injection
- **[curl_cffi](https://github.com/yifeikong/curl_cffi)** for async browser curl impersonation
- **[lxml](https://lxml.de)** for fast and reliable web scraping
- **[Cython](https://cython.org)** for accelerated technical indicator calculations
- **[Redis](https://redis.io)** for intelligent caching of market data
- **[logo.dev](https://logo.dev)** for fetching stock logos

---

## License

This project is licensed under the terms of the **MIT License**.

---

## Support & Feedback

!!! question "**Need Help?**"

* 📧 **Email**: harveytseng2@gmail.com
* 🐛 **Issues**: [GitHub Issues](https://github.com/Verdenroz/finance-query/issues)
* 📖 **Documentation**: [OpenAPI Documentation](https://financequery.apidocumentation.com/)

*As most data is scraped, some endpoints may break. If something is not working or if you have any suggestions, please
reach out!*
