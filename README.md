<h1 align="center">FinanceQuery</h1>

<p align="center">
  <img src=".github/assets/logo.png" alt="FinanceQuery" width="187">
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
technical indicators. It sources data from the unofficial Yahoo Finance API, web scraping, and other financial data
providers.

## Documentation

[Interactive API Documentation](https://financequery.apidocumentation.com/reference)

## Run Locally

Clone the project

```bash
git clone https://github.com/Verdenroz/finance-query.git
```

Go to the project directory

```bash
cd finance-query
```

Install dependencies

```bash
# Using uv (recommended)
uv sync

# Using pip
pip install -e .
```

Cythonize files

```bash
python setup.py build_ext --inplace
```

Start the server

```bash
python -m uvicorn src.main:app --reload
```

## Usage/Examples

The exposed endpoints to the API are:

- https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod
- https://finance-query.onrender.com

An `x-api-key` header can be added if you have enabled security and rate limiting. If a key is not provided, or an
invalid key is used, a rate limit of 8000 requests/day is applied to the request's ip address.

> If you are deploying this for yourself, you can create your own admin key which will not be rate limited. See
> the [.env template](.env.template).

### Example REST Request

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
    "afterHoursPrice": "121.60",
    "change": "-11.13",
    "percentChange": "-8.48%",
    "open": "135.00",
    "high": "135.00",
    "low": "120.01",
    "yearHigh": "153.13",
    "yearLow": "75.61",
    "volume": 432855617,
    "avgVolume": 238908833,
    "marketCap": "2.94T",
    "pe": "40.87",
    "dividend": "0.04",
    "yield": "0.03%",
    "exDividend": "Mar 12, 2025",
    "earningsDate": "May 20, 2025 - May 26, 2025",
    "lastDividend": "0.01",
    "sector": "Technology",
    "industry": "Semiconductors",
    "about": "NVIDIA Corporation provides graphics and compute and networking solutions in the United States, Taiwan, China, Hong Kong, and internationally. The Graphics segment offers GeForce GPUs for gaming and PCs, the GeForce NOW game streaming service and related infrastructure, and solutions for gaming platforms; Quadro/NVIDIA RTX GPUs for enterprise workstation graphics; virtual GPU or vGPU software for cloud-based visual and virtual computing; automotive platforms for infotainment systems; and Omniverse software for building and operating metaverse and 3D internet applications. The Compute & Networking segment comprises Data Center computing platforms and end-to-end networking platforms, including Quantum for InfiniBand and Spectrum for Ethernet; NVIDIA DRIVE automated-driving platform and automotive development agreements; Jetson robotics and other embedded platforms; NVIDIA AI Enterprise and other software; and DGX Cloud software and services. The company's products are used in gaming, professional visualization, data center, and automotive markets. It sells its products to original equipment manufacturers, original device manufacturers, system integrators and distributors, independent software vendors, cloud service providers, consumer internet companies, add-in board manufacturers, distributors, automotive manufacturers and tier-1 automotive suppliers, and other ecosystem participants. It has a strategic collaboration with IQVIA to help realize the potential of AI in healthcare and life sciences. NVIDIA Corporation was incorporated in 1993 and is headquartered in Santa Clara, California.",
    "fiveDaysReturn": "-14.25%",
    "oneMonthReturn": "1.46%",
    "threeMonthReturn": "-11.22%",
    "sixMonthReturn": "-6.35%",
    "ytdReturn": "-10.53%",
    "yearReturn": "52.67%",
    "threeYearReturn": "397.37%",
    "fiveYearReturn": "1,802.61%",
    "tenYearReturn": "21,686.04%",
    "maxReturn": "274,528.59%",
    "logo": "https://img.logo.dev/nvidia.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"
  }
]
```

### Example WebSocket Connection

```javascript
// Connect to WebSocket for real-time updates
const ws = new WebSocket('wss://finance-query.onrender.com/quotes');

ws.onopen = () => {
    console.log('Connected to FinanceQuery WebSocket');
    // Send symbol to subscribe to updates
    ws.send('TSLA');
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Real-time update:', data);
};
```

#### Sample WebSocket Message

```json
[
  {
    "symbol": "TSLA",
    "name": "Tesla, Inc.",
    "price": "398.09",
    "change": "+0.94",
    "percentChange": "+0.24%",
    "logo": "https://img.logo.dev/tesla.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"
  }
]
```

## Available REST Endpoints

| Endpoint                  | Description                                    |
|---------------------------|------------------------------------------------|
| `/health`, `/ping`        | API status and health monitoring               |
| `/hours`                  | Trading hours and market status                |
| `/v1/quotes`              | Detailed quotes and information                |
| `/v1/simple-quotes`       | Simplified quotes with summary information     |
| `/v1/similar`             | Find similar quotes to queried symbol          |
| `/v1/historical`          | Historical price data with customizable ranges |
| `/v1/movers`              | Market gainers, losers, and most active stocks |
| `/v1/news`                | Financial news and market updates              |
| `/v1/indices`             | Major market indices (S&P 500, NASDAQ, DOW)    |
| `/v1/sectors`             | Market sector performance and analysis         |
| `/v1/search`              | Search for securities with filters             |
| `/v1/indicator`           | Get specific indicator history over time       |
| `/v1/indicators`          | Technical indicators summary for interval      |
| `/v1/holders`             | Company ownership and holder information       |
| `/v1/financials`          | Financial statements and company metrics       |
| `/v1/analysis`            | Analyst recommendations, price targets, estimates |
| `/v1/earnings-transcript` | Earnings call transcripts and analysis         |
| `/v1/stream`              | SSE for real-time quote updates                |

## Available WebSocket Endpoints

| Endpoint   | Description                                               |
|------------|-----------------------------------------------------------|
| `/quotes`  | Real-time quotes updates                                  |
| `/profile` | Real-time detailed ticker updates (quote, news, similar)  |
| `/market`  | Real-time market updates (indices, news, movers, sectors) |
| `/hours`   | Real-time market hour updates                             |

## Deployment

### AWS Lambda

Perfect for serverless applications with automatic scaling:

- Follow
  the [AWS Lambda Deployment Guide](https://docs.aws.amazon.com/lambda/latest/dg/python-image.html#python-image-instructions)
- Remember to add the environment variables to the Lambda function
- Alternatively use the [AWS Deployment Workflow](.github/workflows/aws-deploy.yml), providing repository secrets for
  `AWS_SECRET_ID` and `AWS_SECRET_KEY`.
- Also edit the `AWS_REGION`, `ECR_REPOSITORY`, and `FUNCTION_NAME` in the workflow file

#### Testing AWS Lambda Locally

Test the AWS Lambda Docker image locally before deployment:

```bash
# Build and test the Lambda image with automated health checks
make docker-aws
```

This command will:
1. Build the image from `Dockerfile.aws`
2. Run the container with the Lambda Runtime Interface Emulator
3. Test `/ping` and `/health` endpoints using Lambda events
4. Clean up the container after testing

### Render

Easy deployment with WebSocket support:

- Follow the [Render Deployment Guide](https://render.com/docs/deploy-fastapi)
- The deployment should use the `Dockerfile` file in the repository
- Be sure to override the CMD in the Dockerfile in your Render project settings to
  `python -m uvicorn src.main:app --host 0.0.0.0 --port $PORT`
- Alternatively use the [Render Deployment Workflow](.github/workflows/render-deploy.yml), providing repository secrets
  for `RENDER_DEPLOY_HOOK_URL`.
- The deploy hook url can be found in the settings of your Render project

### Docker

Deploy anywhere with Docker:

```bash
# Basic deployment
docker build -t financequery .
docker run -p 8000:8000 financequery
```

#### Docker with Custom Logging Configuration

Configure logging at build time:

```bash
# Build with custom logging settings
docker build \
  --build-arg LOG_LEVEL=DEBUG \
  --build-arg LOG_FORMAT=text \
  --build-arg PERFORMANCE_THRESHOLD_MS=1000 \
  -t financequery .
```

Or configure at runtime:

```bash
# Run with environment variables
docker run -p 8000:8000 \
  -e LOG_LEVEL=WARNING \
  -e LOG_FORMAT=json \
  -e PERFORMANCE_THRESHOLD_MS=5000 \
  financequery
```

#### Production Docker Example

```bash
# Production deployment with structured logging
docker run -p 8000:8000 \
  -e LOG_LEVEL=INFO \
  -e LOG_FORMAT=json \
  -e PERFORMANCE_THRESHOLD_MS=2000 \
  -e REDIS_URL=redis://redis:6379 \
  financequery
```

> **Note**: There are two workflows that will automatically deploy to render and AWS, but they will require repository
> secrets for `AWS_SECRET_ID`, `AWS_SECRET_KEY`, and `RENDER_DEPLOY_HOOK_URL`. Quite frankly, render is easier to work
> with since it enables the websockets, but will require the paid Starter Plan as this API requires extensive memory. If
> you are tight on cash, consider Lambda.

> **WebSocket Support**: Remember the websockets above are not available through Lambda. If you deploy to Render
> instead, you will be able to connect to the websockets through a request that looks like
`wss://finance-query.onrender.com/...`

## Configuration

Customize FinanceQuery with environment variables. These environment variables are optional. The API will function with
default settings if not provided.

### Security Configuration

```env
USE_SECURITY=true
ADMIN_API_KEY=your-secret-admin-key
```

### Proxy Configuration

```env
USE_PROXY=true
PROXY_URL=your-proxy-url
PROXY_TOKEN=your-proxy-token
```

### Redis Caching

```env
REDIS_URL=redis://localhost:6379
```

### Algolia Search

```env
ALGOLIA_APP_ID=your-algolia-app-id
ALGOLIA_API_KEY=your-algolia-api-key
```

### Logging Configuration

Control logging behavior for debugging, monitoring, and production deployments:

```env
# Log level - controls verbosity
LOG_LEVEL=INFO  # Options: DEBUG, INFO, WARNING, ERROR, CRITICAL

# Log format - structured vs human-readable
LOG_FORMAT=json  # Options: json, text

# Performance monitoring threshold in milliseconds
PERFORMANCE_THRESHOLD_MS=2000  # Operations taking longer than this trigger warnings
```

#### Log Levels

- **DEBUG**: Detailed information, including cache hits/misses and operation details
- **INFO**: General information about requests, responses, and external API calls
- **WARNING**: Performance issues and slow operations
- **ERROR**: Error conditions and failed operations
- **CRITICAL**: System-level failures that require immediate attention

#### Log Formats

- **JSON** (`LOG_FORMAT=json`): Structured logging for production monitoring systems
- **Text** (`LOG_FORMAT=text`): Human-readable format for development and debugging

#### Performance Monitoring

Adjust `PERFORMANCE_THRESHOLD_MS` based on your requirements:
- **Development**: `500` - Strict performance monitoring
- **Production**: `2000` - Balanced monitoring (default)
- **High-load**: `5000` - Relaxed monitoring for systems under heavy load

## Performance

FinanceQuery leverages:

- **[FastAPI](https://fastapi.tiangolo.com)** for lightning-fast HTTP performance
- **[fastapi-injectable](https://github.com/JasperSui/fastapi-injectable)** for efficient dependency injection
- **[curl_cffi](https://github.com/yifeikong/curl_cffi)** for async browser curl impersonation
- **[lxml](https://lxml.de)** for fast and reliable web scraping
- **[Cython](https://cython.org)** for accelerated technical indicator calculations
- **[Redis](https://redis.io)** for intelligent caching of market data
- **[logo.dev](https://logo.dev)** for fetching stock logos

## License

This project is licensed under the terms of the **[MIT License](https://opensource.org/licenses/MIT)**.

## Support & Feedback

**Need Help?**

* 📧 **Email**: harveytseng2@gmail.com
* 🐛 **Issues**: [GitHub Issues](https://github.com/Verdenroz/finance-query/issues)
* 📖 **OpenAPI Documentation**: [OpenAPI Documentation](https://financequery.apidocumentation.com/)

*As most data is scraped, some endpoints may break. If something is not working or if you have any suggestions, please
reach out!*
