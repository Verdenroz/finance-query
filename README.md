
# FinanceQuery

FinanceQuery is a simple API to query financial data. It provides endpoints to get quotes, historical prices, indices, market movers, similar stocks, finance news, indicators, search, and sectors. Data is acquired through web scraping and third party libraries. It is the successor to the [GoogleFinanceAPI](https://github.com/Verdenroz/GoogleFinanceAPI).

# Table of Contents

- [Documentation](#documentation)
- [Run Locally](#run-locally)
- [Deployment](#deployment)
  - [AWS Lambda](#aws-lambda)
  - [Render](#render)
- [API Reference](#api-reference)
  - [Get quotes](#get-quotes)
  - [Get simplified quotes](#get-simplified-quotes)
  - [Get historical data for a stock](#get-historical-data-for-a-stock)
  - [Get technical indicators](#get-technical-indicators)
  - [Get summary technical analysis](#get-summary-technical-analysis)
  - [Get similar stocks](#get-similar-stocks)
  - [Get news](#get-news)
  - [Search](#search)
  - [Get sector performance](#get-sector-performance)
  - [Get U.S. market indices](#get-us-market-indices)
  - [Get active stocks](#get-active-stocks)
  - [Get losing stocks](#get-losing-stocks)
  - [Get gaining stocks](#get-gaining-stocks)
  - [Get market open/closed status](#get-market-openclosed-status)
- [Websockets Guide](#websockets-guide)
  - [Quote profile](#quote-profile)
  - [Watchlist](#watchlist)
  - [General market data](#general-market-data)
  - [Market status](#market-status)
- [Usage/Examples](#usageexamples)
- [Proxies](#proxies)
- [Environment Variables](#environment-variables)
- [Feedback](#feedback)
- [License](#license)

## Documentation

[Documentation](https://finance-query.onrender.com/docs)

## Run Locally

Clone the project

```bash
  git clone https://github.com/Verdenroz/FinanceQuery.git
```

Go to the project directory

```bash
  cd finance-query
```

Install dependencies

```bash
 pip install -r requirements.txt
```

Start the server

```bash
  python.exe -m uvicorn src.main:app --reload  
```

## Deployment

#### AWS Lambda

- Follow the [AWS Lambda Deployment Guide](https://docs.aws.amazon.com/lambda/latest/dg/python-image.html#python-image-instructions)
- Remember to add the environment variables to the Lambda function
- Alternatively use the [AWS Deployment Workflow](.github/workflows/aws-deploy.yml), providing repository secrets for `AWS_SECRET_ID` and `AWS_SECRET_KEY`.
    - Also edit the `AWS_REGION`, `ECR_REPOSITORY`, and `FUNCTION_NAME` in the workflow file

#### Render

- Follow the [Render Deployment Guide](https://render.com/docs/deploy-fastapi)
- The deployment should use the `Dockerfile` file in the repository 
- Be sure to override the CMD in the Dockerfile in your Render project settings to `python -m uvicorn src.main:app --host 0.0.0.0 --port $PORT`
- Alternatively use the [Render Deployment Workflow](.github/workflows/render-deploy.yml), providing repository secrets for `RENDER_DEPLOY_HOOK_URL`.
  - The deploy hook url can be found in the settings of your Render project


## API Reference

#### Get quotes

```
  GET /v1/quotes
```

| Query Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `symbols` | `string` | **Required**. Comma-separated list of stock symbols | 

#### Get simplified quotes

```
  GET /v1/simple-quotes
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbols`      | `string` | **Required**. Comma-separated list of stock symbols |

#### Get historical data for a stock

```
  GET /v1/historical
```
***Minutes intervals are available up to 1mo    -   1h interval available up to 1Y***

| Query Parameter | Type     | Description                                                                           |
| :-------- | :------- |:--------------------------------------------------------------------------------------|
| `symbol`  | `string` | **Required**. The symbol of the stock                                                 |
| `time` | `string` | **Required**. Time period: (1d, 5d, 7d, 1mo, 3mo, 6mo, YTD, 1Y, 5Y, 10Y, max)         |
| `interval`  |`string` | **Required**. Interval: between data points (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo) |

#### Get technical indicators

```
  GET /v1/indicators
```
> ***I would not recommend changing the optional params unless you know what you are doing***

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `function`  | `string` | **Required**. The type of indicator: (SMA, EMA, WMA, VWMA, RSI, SRSI, STOCH, CCI, OBV, BBANDS, AROON, ADX, MACD, SUPERTREND, ICHIMOKU) |
| `symbol` | `string` | **Required** The symbol of the stock |
| `interval`| `string`| **Optional [Default 1d]** Interval between data points: (15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)|
|`period`  |`int`| **Optional [Default varies per function]** The look-back period|
|`stoch_period`| `int` | **Optional [Default 14]**  The stochastic look-back period for STOCH and SRSI|
|`signal_period`| `int` | **Optional [Default varies per function]**  The signal period for MACD, STOCH, and SRSI|
|`smooth`| `int` | **Optional [Default 3]**  The smoothing period for STOCH and SRSI.|
|`fast_period`| `int` | **Optional [Default 12]**  The fast period for MACD.|
|`slow_period`| `int` | **Optional [Default 26]**  The slow period for MACD.|
|`std_dev`| `int` | **Optional [Default 2]**  The standard deviation for Bollinger Bands.|
|`sma_periods`| `int` | **Optional [Default None]**  The look-back period for the SMA in OBV.|
|`multiplier`| `int` | **Optional [Default 3]**  The multiplier for SuperTrend.|
|`tenkan_period`| `int` | **Optional [Default 9]**  The look-back period for the Tenkan line in Ichimoku.|
|`kijun_period`| `int` | **Optional [Default 26]**  The look-back period for the Kijun line in Ichimoku.|
|`senkou_period`| `int` | **Optional [Default 52]**  The look-back period for the Senkou span in Ichimoku.|

#### Get summary technical analysis

```
  GET /v1/analysis
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Required**. The symbol of the stock |
| `interval`  |`string` | **Optional [Default 1d]**. Interval: between data points (15m, 30m, 1h, 1d, 1wk, 1mo, 3mo) |

#### Get similar stocks

```
  GET /v1/similar
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Required**. The symbol of the stock to find similar stocks around |

#### Get news

```
  GET /v1/news
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Optional**. Specify symbol to find specific news around a stock |

#### Search

```
  GET /v1/search
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `query`  | `string` | **Optional**. Search query by symbol or name |


#### Get sector performance

```
  GET /v1/sectors
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Optional**. Specify symbol to find specific sector performance belonging to the symbol |
| ` name ` | ` string`| **Optional**. The specific name of the individual sector (Technology, Consumer Cyclical etc) |


#### Get U.S. market indices

```
  GET /v1/indices
```

#### Get active stocks

```
  GET /v1/actives
```

#### Get losing stocks

```
  GET /v1/losers
```

#### Get gaining stocks

```
  GET /v1/gainers
```

#### Get market open/closed status

```
  GET /hours

```


## Websockets Guide

> **The websockets depend on Redis PubSub and will require Redis credentials in your [.env](https://github.com/Verdenroz/finance-query?tab=readme-ov-file#environment-variables)**

There are currently four implemented websocket routes: `profile`, `quotes`, `market`, and `hours`. These will not be accessible through Lambda. If you are interested in deployment, I recommend deploying to [Render](https://render.com/) as it will be able to host the entire FastAPI server, including the websockets. If you are testing locally, your requests will be `ws` instead of `wss`. Data is returned on a set interval every 10 seconds.

### Quote profile 
> #### Combines `quote`, `similar stocks`, `sector for symbol`, `news for symbol`

```
WSS /profile/{symbol}
```

### Watchlist
> #### Requires comma separated list of symbols to be sent initially, streaming simplified quotes for all provided symbols

```
WSS /quotes
```

### General market data
> #### Combines `gainers`, `losers`, `actives`, `news`, and `sectors`

```
WSS /market
```

### Market status
> #### Streams whether the market is open or closed, sending a message only when the status changes

```
WSS /hours
```

## Usage/Examples

The exposed endpoints to the API is
- https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod.
- https://finance-query.onrender.com

There are two workflows that will automatically deploy to render and AWS, but they will require repository secrets for `AWS_SECRET_ID`, `AWS_SECRET_KEY`, and `RENDER_DEPLOY_HOOK_URL`. Quite frankly, render is easier to work with since it enables the websockets, but will require the paid Starter Plan as this API requires extensive memory. If you are tight on cash, consider Lambda.

An x-api-key header must be added to all requests. The demo key is **FinanceQueryDemoAWSHT** (2000 requests/day). 

> If you are deploying this for yourself, you can create your own admin key which will not be rate limited. See the [.env template](.env.template).

> Again, remember the websockets above are not available through Lambda. If you deploy to Render instead, you will be able to connect to the websockets through a request that looks like `wss://finance-query.onrender.com/...`

## Proxies

Proxies are off by default, but they can be enabled with the correct environment variables. See the [.env template](.env.template). It is recommended that all deployed instances use proxies to avoid any potential blocking. I am currently using [BrightData](https://brightdata.com/), though you are welcome to change whatever fits you best. FastAPI's lifespan events handles ip address whitelisting and session cleanup.


## Environment Variables

To run this project locally, you will need to add the following environment variables to your .env.

`REDIS_HOST`

`REDIS_PASSWORD`

`REDIS_PORT`

`REDIS_USERNAME`

`ALGOLIA_APP_ID`

`ALGOLIA_KEY`

`USE_PROXY`

`PROXY_URL`

`PROXY_USER`

`PROXY_PASSWORD`

`USE_SECURITY`

`ADMIN_API_KEY`

> - ***If you do not use redis, you can simply disable the redis cache by deleting the @cache decorator to all routes. Cache can still be enabled with in-memory async-lru. See [@alru_cache](https://pypi.org/project/async-lru/)***
> - ***Websockets and Rate Limiting will not work without Redis.***
> - ***Search endpoint will not work without Algolia.***
> - ***Proxies are optional but recommended for deployment.***
> - ***If security is enabled, api keys will be validated and rate limits enforced, though this depends on Redis, so it is off by default.***
> - ***Set if admin key security is enabled. Admin API key must be kept secret as this will have no rate limit attached.***

## Feedback

*As most data is scraped, some endpoints may break*

If something is not working or if you have any suggestions, contact me at harveytseng2@gmail.com


## License

[MIT](https://opensource.org/license/MIT)

