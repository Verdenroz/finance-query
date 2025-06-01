# Technical Indicators

## GET /v1/indicator

### Overview

**Purpose:** Retrieve historical technical indicator data for a specific stock  
**Response Format:** Time series data for the requested technical indicator with historical values

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter        | Type    | Required | Description                                  | Example |
|------------------|---------|:--------:|----------------------------------------------|---------|
| `function`       | string  |    ✓     | Technical indicator to fetch                 | `SMA`   |
| `symbol`         | string  |    ✓     | Stock symbol                                 | `NVDA`  |
| `range`          | string  |          | Time range for historical data (default: 2y) | `1y`    |
| `interval`       | string  |          | Interval between data points (default: 1d)   | `1d`    |
| `epoch`          | boolean |          | Return timestamps as epoch (default: false)  | `true`  |
| `lookBackPeriod` | integer |          | Look-back period for indicators              | `14`    |
| `stochPeriod`    | integer |          | Stochastic look-back period                  | `14`    |
| `signalPeriod`   | integer |          | Signal period                                | `9`     |
| `smooth`         | integer |          | Smoothing period                             | `3`     |
| `fastPeriod`     | integer |          | Fast period (for MACD)                       | `12`    |
| `slowPeriod`     | integer |          | Slow period (for MACD)                       | `26`    |
| `stdDev`         | integer |          | Standard deviation (for BBANDS)              | `2`     |
| `smaPeriods`     | integer |          | SMA look-back (for OBV)                      | `20`    |
| `multiplier`     | integer |          | Multiplier (for SUPERTREND)                  | `3`     |
| `tenkanPeriod`   | integer |          | Tenkan look-back (for ICHIMOKU)              | `9`     |
| `kijunPeriod`    | integer |          | Kijun look-back (for ICHIMOKU)               | `26`    |
| `senkouPeriod`   | integer |          | Senkou look-back (for ICHIMOKU)              | `52`    |

#### Valid function values

`SMA`, `EMA`, `WMA`, `VWMA`, `RSI`, `SRSI`, `STOCH`, `CCI`, `OBV`, `BBANDS`, `AROON`, `ADX`, `MACD`, `SUPERTREND`,
`ICHIMOKU`

#### Available Range Options

`1d`, `5d`, `1mo`, `3mo`, `6mo`, `ytd`, `1y`, `2y`, `5y`, `10y`, `max`

#### Available Interval Options

`1m`, `5m`, `15m`, `30m`, `1h`, `1d`, `1wk`, `1mo`

!!! warning "Interval and Range Compatibility"
    | Interval | Compatible Ranges                                   |
    |----------|-----------------------------------------------------|
    | `1m`     | `1d`, `5d` only                                     |
    | `5m`     | `1d`, `5d`, `1mo` only                              |
    | `15m`    | `1d`, `5d`, `1mo` only                              |
    | `30m`    | `1d`, `5d`, `1mo` only                              |
    | `1h`     | `1d`, `5d`, `1mo`, `3mo`, `6mo`, `ytd`, `1y` only   |
    | `1mo`    | Required for `max` range                            |
    
    Attempting incompatible combinations will result in a 400 Bad Request error.
**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Schema:** [`TechnicalIndicator`](#technicalindicator-schema) object
    - **Example (200):**
      ```json
      {
        "type": "SMA",
        "Technical Analysis": {
          "2021-07-09": { "value": 30.0 },
          "2021-07-10": { "value": 31.5 }
        }
      }
      ```

- **400 Bad Request**
  ```json
  { "detail": "Invalid parameter: {parameter} for the {function} function." }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "function": [
        "Field required",
        "Input should be 'SMA', 'EMA', 'WMA', 'VWMA', 'RSI', 'SRSI', 'STOCH', 'CCI', 'OBV', 'BBANDS', 'AROON', 'ADX', 'MACD', 'SUPERTREND' or 'ICHIMOKU'"
      ],
      "symbol": ["Field required"],
      "interval": ["Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', or '1mo'"],
      "period": ["Input should be a valid integer, unable to parse string as an integer"],
      "stoch_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "signal_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "smooth": ["Input should be a valid integer, unable to parse string as an integer"],
      "fast_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "slow_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "std_dev": ["Input should be a valid integer, unable to parse string as an integer"],
      "sma_periods": ["Input should be a valid integer, unable to parse string as an integer"],
      "multiplier": ["Input should be a valid integer, unable to parse string as an integer"],
      "tenkan_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "kijun_period": ["Input should be a valid integer, unable to parse string as an integer"],
      "senkou_period": ["Input should be a valid integer, unable to parse string as an integer"]
    }
  }
  ```

## GET /v1/indicators

### Overview

**Purpose:** Retrieve latest values for multiple technical indicators for a specific stock  
**Response Format:** Current values for requested technical indicators with their parameters

### Authentication

Optional authentication via `x-api-key` header token

### Request Parameters

| Parameter   | Type   | Required | Description                                | Example       |
|-------------|--------|:--------:|--------------------------------------------|---------------|
| `symbol`    | string |    ✓     | Stock symbol                               | `NVDA`        |
| `interval`  | string |          | Interval for historical data (default: 1d) | `1d`          |
| `functions` | string |          | Comma-separated list of indicators         | `SMA,EMA,RSI` |

#### Available Interval Options

`1m`, `5m`, `15m`, `30m`, `1h`, `1d`, `1wk`, `1mo`

**Responses:**

- **200 OK**
    - **Content-Type:** `application/json`
    - **Example (200):**
      ```json
      {
        "SMA(10)": {
          "SMA": 129.03
        },
        "SMA(20)": {
          "SMA": 131.08
        },
        "SMA(50)": {
          "SMA": 134.95
        },
        "SMA(100)": {
          "SMA": 135.54
        },
        "SMA(200)": {
          "SMA": 124.78
        },
        "EMA(10)": {
          "EMA": 131.93
        },
        "EMA(20)": {
          "EMA": 131.64
        },
        "EMA(50)": {
          "EMA": 133.51
        },
        "EMA(100)": {
          "EMA": 131.7
        },
        "EMA(200)": {
          "EMA": 120.76
        },
        "WMA(10)": {
          "WMA": 125.72
        },
        "WMA(20)": {
          "WMA": 132.3
        },
        "WMA(50)": {
          "WMA": 136.83
        },
        "WMA(100)": {
          "WMA": 135.32
        },
        "WMA(200)": {
          "WMA": 118.59
        },
        "VWMA(20)": {
          "VWMA": 128.17
        },
        "RSI(14)": {
          "RSI": 56.56
        },
        "SRSI(3,3,14,14)": {
          "%K": 92.79,
          "%D": 81.77
        },
        "STOCH %K(14,3,3)": {
          "%K": 81.25,
          "%D": 67.41
        },
        "CCI(20)": {
          "CCI": 63.36
        },
        "BBANDS(20,2)": {
          "Upper Band": 149.81,
          "Middle Band": 131.08,
          "Lower Band": 112.35
        },
        "Aroon(25)": {
          "Aroon Up": 40,
          "Aroon Down": 64
        },
        "ADX(14)": {
          "ADX": 14.43
        },
        "MACD(12,26)": {
          "MACD": -0.53,
          "Signal": -2.1
        },
        "Super Trend": {
          "Super Trend": 140.25,
          "Trend": "DOWN"
        },
        "Ichimoku Cloud": {
          "Conversion Line": 127.97,
          "Base Line": 130.99,
          "Lagging Span": 138.85,
          "Leading Span A": 141.74,
          "Leading Span B": 140
        }
      }
      ```

- **400 Bad Request**
  ```json
  { "detail": "Invalid parameter: {parameter} for the {function} function." }
  ```

- **404 Not Found**
  ```json
  { "detail": "Symbol not found" }
  ```

- **422 Unprocessable Entity**
  ```json
  {
    "detail": "Invalid request",
    "errors": {
      "symbol": ["Field required"],
      "interval": ["Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', or '1mo'"]
    }
  }
  ```

## Schema References

### TechnicalIndicator Schema

| Field              | Type   | Description                           | Required |
|--------------------|--------|---------------------------------------|----------|
| type               | string | The technical indicator (e.g., "SMA") | ✓        |
| Technical Analysis | object | Dates mapped to indicator values      | ✓        |

### SMAData Schema

| Field | Type  | Description                                                   | Required |
|-------|-------|---------------------------------------------------------------|----------|
| value | float | Average price over a specified period, smoothing price action |          |

### EMAData Schema

| Field | Type  | Description                                              | Required |
|-------|-------|----------------------------------------------------------|----------|
| value | float | Weighted average giving more importance to recent prices |          |

### WMAData Schema

| Field | Type  | Description                                                     | Required |
|-------|-------|-----------------------------------------------------------------|----------|
| value | float | Average where recent prices carry more weight than older prices |          |

### VWMAData Schema

| Field | Type  | Description                                                                    | Required |
|-------|-------|--------------------------------------------------------------------------------|----------|
| value | float | Price average weighted by volume, showing where most trading activity occurred |          |

### RSIData Schema

| Field | Type  | Description                                                           | Required |
|-------|-------|-----------------------------------------------------------------------|----------|
| value | float | Momentum oscillator (0-100) indicating overbought/oversold conditions |          |

### SRSIData Schema

| Field | Type  | Description                                | Required |
|-------|-------|--------------------------------------------|----------|
| k     | float | Fast stochastic line applied to RSI values |          |
| d     | float | Smoothed signal line of the stochastic RSI |          |

### STOCHData Schema

| Field | Type  | Description                                                  | Required |
|-------|-------|--------------------------------------------------------------|----------|
| k     | float | Fast line showing current price position within recent range |          |
| d     | float | Smoothed signal line of the stochastic oscillator            |          |

### CCIData Schema

| Field | Type  | Description                                                        | Required |
|-------|-------|--------------------------------------------------------------------|----------|
| value | float | Measures price deviation from statistical average to spot extremes |          |

### MACDData Schema

| Field  | Type  | Description                                            | Required |
|--------|-------|--------------------------------------------------------|----------|
| value  | float | Difference between fast and slow moving averages       |          |
| signal | float | Smoothed MACD line used for buy/sell signal generation |          |

### ADXData Schema

| Field | Type  | Description                                             | Required |
|-------|-------|---------------------------------------------------------|----------|
| value | float | Strength of price trend regardless of direction (0-100) |          |

### AROONData Schema

| Field      | Type  | Description                                    | Required |
|------------|-------|------------------------------------------------|----------|
| aroon_up   | float | Time since highest high within lookback period |          |
| aroon_down | float | Time since lowest low within lookback period   |          |

### BBANDSData Schema

| Field       | Type  | Description                                          | Required |
|-------------|-------|------------------------------------------------------|----------|
| upper_band  | float | Price ceiling based on standard deviations above SMA |          |
| middle_band | float | Simple moving average serving as the centerline      |          |
| lower_band  | float | Price floor based on standard deviations below SMA   |          |

### OBVData Schema

| Field | Type  | Description                                                     | Required |
|-------|-------|-----------------------------------------------------------------|----------|
| value | float | Running total of volume based on price direction (up/down days) |          |

### SuperTrendData Schema

| Field | Type   | Description                                        | Required |
|-------|--------|----------------------------------------------------|----------|
| value | float  | Dynamic support/resistance level based on ATR      |          |
| trend | string | Current market direction ("up", "down", "neutral") | ✓        |

### IchimokuData Schema

| Field         | Type  | Description                                                   | Required |
|---------------|-------|---------------------------------------------------------------|----------|
| tenkan_sen    | float | Conversion line - short-term momentum and support/resistance  |          |
| kijun_sen     | float | Base line - medium-term momentum and major support/resistance |          |
| chikou_span   | float | Lagging span - current price plotted 26 periods back          |          |
| senkou_span_a | float | Leading span A - future support/resistance level              |          |
| senkou_span_b | float | Leading span B - future support/resistance level              |          |
