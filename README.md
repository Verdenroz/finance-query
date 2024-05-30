
# FinanceQuery

FinanceQuery is a simple API to query financial data. It provides endpoints to get quotes, historical prices, indices, market movers, similar stocks, finance news, indicators, search, and sectors. Data is acquired through web scraping and third party libraries. It is the successor to the [GoogleFinanceAPI](https://github.com/Verdenroz/GoogleFinanceAPI).

## Documentation

[Documentation](https://d3tidzj12m5ipv.cloudfront.net)





## API Reference

#### Get quotes

```http
  GET /v1/quotes
```

| Query Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `symbols` | `string` | **Required**. Comma-separated list of stock symbols | 

#### Get simplified quotes

```http
  GET /v1/simple-quotes
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbols`      | `string` | **Required**. Comma-separated list of stock symbols |

#### Get historical data for a stock

```http
  GET /v1/historical
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Required**. The symbol of the stock |
| `time` | `string` | **Required**. Time period: (1d, 5d, 1mo, 3mo, 6mo, YTD, 1Y, 5Y, 10Y, max) |
| `interval`  |`string` | **Required**. Interval: between data points (15m, 30m, 1h, 1d, 1wk, 1mo, 3mo) |

#### Get technical indicators

```http
  GET /v1/indicators
```
***I would not recommend changing the optional params unless you know what you are doing***

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

```http
  GET /v1/analysis
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Required**. The symbol of the stock |
| `interval`  |`string` | **Optional [Default 1d]**. Interval: between data points (15m, 30m, 1h, 1d, 1wk, 1mo, 3mo) |

#### Get similar stocks

```http
  GET /v1/similar-stocks
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Required**. The symbol of the stock to find similar stocks around |

#### Get news

```http
  GET /v1/news
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `symbol`  | `string` | **Optional**. Specify symbol to find specific news around a stock |

#### Search

```http
  GET /v1/search
```

| Query Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `query`  | `string` | **Optional**. Search query by symbol or name |


#### Get U.S. market indices

```http
  GET /v1/indices
```

#### Get active stocks

```http
  GET /v1/actives
```

#### Get losing stocks

```http
  GET /v1/losers
```

#### Get gaining stocks

```http
  GET /v1/gainers
```

#### Get sector performance

```http
  GET /v1/sectors
```



## Usage/Examples

The exposed endpoint to the API is https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod

An x-api-key header must be added to all requests. The demo key is **FinanceQueryDemoAWSHT** (500 requests/day)


## Run Locally

Clone the project

```bash
  git clone https://github.com/Verdenroz/FinanceQuery.git
```

Install dependencies

```bash
 pip install -r requirements.txt
```

Start the server

```bash
  python.exe -m uvicorn src.main:app --reload  
```
## Environment Variables

To run this project locally, you will need to add the following environment variables to your .env

`ALGOLIA_APP_ID`

`ALGOLIA_KEY`

`REDIS_HOST`

`REDIS_PASSWORD`

`REDIS_PORT`

`REDIS_USERNAME`

***If you do not add these environment variables or do not use algolia/redis, you can simply disable the redis cache by deleting the @cache decorator to all routes. Search will not work without algolia.***


## Feedback

*As most data is scraped, some endpoints may break*

If something is not working or if you have any suggestions, contact me at harveytseng2@gmail.com


## License

[MIT](https://choosealicense.com/licenses/mit/)

