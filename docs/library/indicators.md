# Indicators

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — indicators](https://docs.rs/finance-query/latest/finance_query/indicators/index.html)

Access 42 technical indicators and 20 candlestick patterns calculated from historical price data.

## Enable Feature

Add the `indicators` feature to your `Cargo.toml`:

```toml
[dependencies]
finance-query = { version = "2", features = ["indicators"] }
```

Or enable it alongside other features:

```toml
[dependencies]
finance-query = { version = "2", features = ["dataframe", "indicators"] }
```

## Getting Started

Fetch indicators for a symbol:

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;
let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

println!("RSI(14): {:?}", indicators.rsi_14);
println!("SMA(200): {:?}", indicators.sma_200);
println!("MACD: {:?}", indicators.macd);
```

## Three Ways to Calculate Indicators

Finance Query provides three approaches for calculating indicators, each suited for different use cases:

!!! tip "Decision Matrix"

    | Approach | Use Case | Custom Periods | Data Source | Caching |
    |----------|----------|----------------|-------------|---------|
    | **Summary API** | Multiple indicators, dashboards | ✗ Fixed only | Automatic | ✓ Yes |
    | **Chart Methods** | Few indicators, custom periods | ✓ Yes | Chart data | ✗ No |
    | **Direct Functions** | Advanced, backtesting, custom data | ✓ Yes | Any Vec<f64> | ✗ No |


### 1. Summary API

Get all indicators pre-calculated with standard periods. Best for dashboards and analysis requiring many indicators.

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;
let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

// All indicators calculated at once with standard periods
println!("RSI(14): {:?}", indicators.rsi_14);
println!("SMA(200): {:?}", indicators.sma_200);
println!("MACD: {:?}", indicators.macd);
```

### 2. Chart Extension Methods

Call indicators directly on chart data with custom periods. Best when you need specific periods or a few indicators.

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

// Calculate indicators with custom periods
let sma_15 = chart.sma(15);           // Custom period: 15
let rsi_21 = chart.rsi(21)?;          // Custom period: 21
let macd = chart.macd(12, 26, 9)?;    // Custom MACD parameters

// Access the last value
if let Some(&last_sma) = sma_15.last().and_then(|v| v.as_ref()) {
    println!("Latest SMA(15): {:.2}", last_sma);
}

// Candlestick patterns (same chart, no extra request)
let signals = chart.patterns();
```

### 3. Direct Indicator Functions

Call raw indicator functions on price arrays. Best for custom data sources, backtesting, or advanced use cases.

```rust
use finance_query::{Ticker, Interval, TimeRange};
use finance_query::indicators::{sma, rsi, macd};

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

// Extract price data (convenience methods on Chart)
let closes: Vec<f64> = chart.close_prices();
let highs: Vec<f64> = chart.high_prices();
let lows: Vec<f64> = chart.low_prices();

// Calculate indicators directly
let sma_25 = sma(&closes, 25);                    // Returns Vec<Option<f64>>
let rsi_10 = rsi(&closes, 10)?;                   // Returns Result<Vec<Option<f64>>>
let macd_result = macd(&closes, 12, 26, 9)?;      // Returns Result<MacdResult>

// Access results
if let Some(&last_rsi) = rsi_10.last().and_then(|v| v.as_ref()) {
    println!("RSI(10): {:.2}", last_rsi);
}

// MACD returns a struct with three series
if let Some(&last_macd) = macd_result.macd_line.last().and_then(|v| v.as_ref()) {
    println!("MACD Line: {:.4}", last_macd);
}
```

### Working with Compound Indicators

Some indicators return multiple series in a result struct. Here's how to use them with direct functions:

```rust
use finance_query::indicators::{bollinger_bands, stochastic, macd};

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

let closes = chart.close_prices();
let highs = chart.high_prices();
let lows = chart.low_prices();

// Bollinger Bands - returns BollingerBands struct
let bb = bollinger_bands(&closes, 20, 2.0)?;
if let Some(&upper) = bb.upper.last().and_then(|v| v.as_ref()) {
    if let Some(&middle) = bb.middle.last().and_then(|v| v.as_ref()) {
        if let Some(&lower) = bb.lower.last().and_then(|v| v.as_ref()) {
            println!("BB: Upper={:.2}, Middle={:.2}, Lower={:.2}", upper, middle, lower);
        }
    }
}

// Stochastic Oscillator - returns StochasticResult struct
let stoch = stochastic(&highs, &lows, &closes, 14, 3)?;
if let Some(&k) = stoch.k.last().and_then(|v| v.as_ref()) {
    if let Some(&d) = stoch.d.last().and_then(|v| v.as_ref()) {
        println!("Stochastic: %K={:.2}, %D={:.2}", k, d);
    }
}

// MACD - returns MacdResult struct
let macd_data = macd(&closes, 12, 26, 9)?;
if let Some(&line) = macd_data.macd_line.last().and_then(|v| v.as_ref()) {
    if let Some(&signal) = macd_data.signal_line.last().and_then(|v| v.as_ref()) {
        if let Some(&hist) = macd_data.histogram.last().and_then(|v| v.as_ref()) {
            println!("MACD: Line={:.4}, Signal={:.4}, Histogram={:.4}", line, signal, hist);
        }
    }
}
```

### Available Result Structs

Direct indicator functions return these result types:

- **Simple indicators** (SMA, EMA, RSI, ATR): `Vec<Option<f64>>`
- **MACD**: `MacdResult { macd_line, signal_line, histogram }`
- **Bollinger Bands**: `BollingerBands { upper, middle, lower }`
- **Stochastic**: `StochasticResult { k, d }`
- **Aroon**: `AroonResult { aroon_up, aroon_down }`
- **SuperTrend**: `SuperTrendResult { value, is_uptrend }`
- **Ichimoku**: `IchimokuResult { conversion_line, base_line, leading_span_a, leading_span_b, lagging_span }`
- **Keltner Channels**: `KeltnerChannelsResult { upper, middle, lower }`
- **Donchian Channels**: `DonchianChannelsResult { upper, middle, lower }`
- **Bull/Bear Power**: `BullBearPowerResult { bull_power, bear_power }`
- **Elder Ray**: `ElderRayResult { bull_power, bear_power }`

## Available Indicators

All indicators return `Option<T>` — `None` when there is insufficient data to calculate.

### Moving Averages

Simple, exponential, and specialized moving averages for trend identification.

**Simple Moving Averages (SMA):**
`sma_10`, `sma_20`, `sma_50`, `sma_100`, `sma_200`

**Exponential Moving Averages (EMA):**
`ema_10`, `ema_20`, `ema_50`, `ema_100`, `ema_200`

**Weighted Moving Averages (WMA):**
`wma_10`, `wma_20`, `wma_50`, `wma_100`, `wma_200`

**Advanced Moving Averages:**

- `dema_20` - Double Exponential Moving Average
- `tema_20` - Triple Exponential Moving Average
- `hma_20` - Hull Moving Average
- `vwma_20` - Volume Weighted Moving Average
- `alma_9` - Arnaud Legoux Moving Average
- `mcginley_dynamic_20` - McGinley Dynamic

### Momentum Oscillators

Measure rate of change and momentum for entry/exit signals.

- `rsi_14` - Relative Strength Index
- `stochastic` - Stochastic Oscillator (K and D lines)
- `stochastic_rsi` - Stochastic RSI
- `cci_20` - Commodity Channel Index
- `williams_r_14` - Williams %R
- `roc_12` - Rate of Change
- `momentum_10` - Momentum
- `cmo_14` - Chande Momentum Oscillator
- `awesome_oscillator` - Bill Williams Awesome Oscillator
- `coppock_curve` - Coppock Curve

### Trend Indicators

Identify trend direction and strength.

- `macd` - MACD (line, signal, histogram)
- `adx_14` - Average Directional Index
- `aroon` - Aroon Up/Down
- `supertrend` - Supertrend (value, trend direction)
- `ichimoku` - Ichimoku Cloud (multiple components)
- `parabolic_sar` - Parabolic SAR
- `bull_bear_power` - Bull and Bear Power
- `elder_ray_index` - Elder Ray Index (bull power, bear power)

### Volatility Indicators

Measure price volatility and support/resistance levels.

- `bollinger_bands` - Bollinger Bands (upper, middle, lower)
- `keltner_channels` - Keltner Channels (upper, middle, lower)
- `donchian_channels` - Donchian Channels (upper, lower)
- `atr_14` - Average True Range
- `true_range` - True Range (raw)
- `choppiness_index_14` - Choppiness Index

### Volume Indicators

Analyze volume patterns and accumulation/distribution.

- `obv` - On-Balance Volume
- `mfi_14` - Money Flow Index
- `cmf_20` - Chaikin Money Flow
- `chaikin_oscillator` - Chaikin Oscillator
- `accumulation_distribution` - Accumulation/Distribution Line
- `vwap` - Volume Weighted Average Price
- `balance_of_power` - Balance of Power

## Candlestick Patterns

Detect 20 classic candlestick patterns across an entire chart in one call.

```rust
use finance_query::{Ticker, Interval, TimeRange};
use finance_query::indicators::{patterns, CandlePattern, PatternSentiment};

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

// Via Chart extension method — same length as chart.candles
let signals = chart.patterns();

// Or call the function directly with a candle slice
let signals = patterns(&chart.candles);

// Each slot is Some(pattern) or None; iterate with candles for context
for (candle, pattern) in chart.candles.iter().zip(signals.iter()) {
    if let Some(p) = pattern {
        println!(
            "timestamp={}: {:?} ({:?})",
            candle.timestamp, p, p.sentiment()
        );
    }
}
```

**Pattern catalogue:**

| Bars | Pattern | Signal |
|------|---------|--------|
| 3 | `MorningStar` | Bullish reversal |
| 3 | `EveningStar` | Bearish reversal |
| 3 | `ThreeWhiteSoldiers` | Bullish continuation |
| 3 | `ThreeBlackCrows` | Bearish continuation |
| 2 | `BullishEngulfing` | Bullish reversal |
| 2 | `BearishEngulfing` | Bearish reversal |
| 2 | `BullishHarami` | Bullish reversal |
| 2 | `BearishHarami` | Bearish reversal |
| 2 | `PiercingLine` | Bullish reversal |
| 2 | `DarkCloudCover` | Bearish reversal |
| 2 | `TweezerBottom` | Bullish reversal at support |
| 2 | `TweezerTop` | Bearish reversal at resistance |
| 1 | `Hammer` | Bullish reversal (requires prior downtrend) |
| 1 | `InvertedHammer` | Bullish reversal (requires prior downtrend) |
| 1 | `HangingMan` | Bearish reversal (requires prior uptrend) |
| 1 | `ShootingStar` | Bearish reversal (requires prior uptrend) |
| 1 | `BullishMarubozu` | Bullish momentum |
| 1 | `BearishMarubozu` | Bearish momentum |
| 1 | `Doji` | Indecision |
| 1 | `SpinningTop` | Indecision |

**Key design notes:**

- **Priority chain:** three-bar wins over two-bar wins over one-bar. Each candle slot holds at most one pattern.
- **Trend-aware:** `Hammer` / `HangingMan` and `InvertedHammer` / `ShootingStar` are the same physical shape — context (prior 3-bar trend) determines which label is assigned.
- **Harami Cross:** A Doji inside a large body is classified as `BullishHarami` / `BearishHarami` — this is the stronger variant per Nison's definition; no separate variant needed.
- **Alignment:** output is always `Vec<Option<CandlePattern>>` of the same length as the input candle slice.

### Using PatternSentiment

```rust
let signals = chart.patterns();

let bullish = signals.iter().filter(|s| {
    s.map(|p| p.sentiment() == PatternSentiment::Bullish).unwrap_or(false)
}).count();

let bearish = signals.iter().filter(|s| {
    s.map(|p| p.sentiment() == PatternSentiment::Bearish).unwrap_or(false)
}).count();

println!("Bull/Bear ratio: {}/{}", bullish, bearish);
```

### Combining Patterns with Indicators

```rust
let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;
let rsi = chart.rsi(14)?;
let signals = chart.patterns();

// Find bars where RSI is oversold AND a bullish pattern just completed
for (i, (pattern, rsi_val)) in signals.iter().zip(rsi.iter()).enumerate() {
    let is_bullish_pattern = pattern
        .map(|p| p.sentiment() == PatternSentiment::Bullish)
        .unwrap_or(false);
    let is_oversold = rsi_val.map(|r| r < 30.0).unwrap_or(false);

    if is_bullish_pattern && is_oversold {
        println!(
            "Strong buy signal at bar {}: {:?} with RSI={:.1}",
            i,
            pattern.unwrap(),
            rsi_val.unwrap()
        );
    }
}
```

## Working with Indicator Results

Different indicators return different types. Simple indicators return `Option<f64>`, while compound indicators return special struct types:

```rust
// Simple indicators (Option<f64>)
if let Some(rsi) = indicators.rsi_14 {
    println!("RSI(14): {:.2}", rsi);
    if rsi < 30.0 {
        println!("  Oversold");
    } else if rsi > 70.0 {
        println!("  Overbought");
    }
}

// Moving averages
if let Some(sma200) = indicators.sma_200 {
    println!("SMA(200): {:.2}", sma200);
}

// MACD (compound - MacdData struct)
if let Some(macd) = indicators.macd {
    if let Some(line) = macd.macd {
        println!("MACD Line: {:.4}", line);
    }
    if let Some(signal) = macd.signal {
        println!("Signal: {:.4}", signal);
    }
    if let Some(histogram) = macd.histogram {
        println!("Histogram: {:.4}", histogram);
    }
}

// Stochastic (StochasticData struct)
if let Some(stoch) = indicators.stochastic {
    if let Some(k) = stoch.k {
        println!("%K: {:.2}", k);
    }
    if let Some(d) = stoch.d {
        println!("%D: {:.2}", d);
    }
}

// Bollinger Bands (BollingerBandsData struct)
if let Some(bb) = indicators.bollinger_bands {
    if let Some(upper) = bb.upper {
        println!("Upper: {:.2}", upper);
    }
    if let Some(middle) = bb.middle {
        println!("Middle: {:.2}", middle);
    }
    if let Some(lower) = bb.lower {
        println!("Lower: {:.2}", lower);
    }
}

// Aroon (AroonData struct)
if let Some(aroon) = indicators.aroon {
    if let Some(up) = aroon.aroon_up {
        println!("Aroon Up: {:.2}", up);
    }
    if let Some(down) = aroon.aroon_down {
        println!("Aroon Down: {:.2}", down);
    }
}

// Ichimoku (IchimokuData struct)
if let Some(ichimoku) = indicators.ichimoku {
    if let Some(conversion) = ichimoku.conversion_line {
        println!("Conversion Line: {:.2}", conversion);
    }
    if let Some(base) = ichimoku.base_line {
        println!("Base Line: {:.2}", base);
    }
}
```

## Converting to DataFrame

Convert all indicators to a Polars DataFrame for analysis:

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;
let indicators = ticker.indicators(
    Interval::OneDay,
    TimeRange::ThreeMonths
).await?;

let df = indicators.to_dataframe()?;
println!("{}", df);
```

## Caching Behavior

Indicators are cached by (interval, range) combination:

```rust
// First call fetches and caches
let ind1 = ticker.indicators(Interval::OneDay, TimeRange::OneMonth).await?;

// Second call returns cached result
let ind2 = ticker.indicators(Interval::OneDay, TimeRange::OneMonth).await?;

// Different range: fetches new data
let ind3 = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
```

## Common Patterns

### Trend Confirmation with Multiple MAs

```rust
let indicators = ticker.indicators(Interval::OneDay, TimeRange::OneYear).await?;

let sma_200 = indicators.sma_200.unwrap_or(0.0);
let ema_50 = indicators.ema_50.unwrap_or(0.0);
let ema_20 = indicators.ema_20.unwrap_or(0.0);

if ema_20 > ema_50 && ema_50 > sma_200 {
    println!("Uptrend confirmed");
}
```

### RSI Extremes

```rust
if let Some(rsi) = indicators.rsi_14 {
    if rsi < 30.0 {
        println!("Oversold");
    } else if rsi > 70.0 {
        println!("Overbought");
    }
}
```

### MACD Crossover

```rust
if let Some(macd) = indicators.macd {
    if let (Some(line), Some(signal)) = (macd.macd, macd.signal) {
        if line > signal {
            println!("Bullish MACD crossover");
        } else {
            println!("Bearish MACD crossover");
        }
    }
}
```

## Best Practices

!!! tip "Optimize Performance and Data Usage"
    - **Store indicator results** - Indicators are calculated fresh each time, so store the result if accessing multiple values
    - **Underlying chart data is cached** - Same `(interval, range)` avoids network requests but still recalculates indicators
    - **Fetch appropriate ranges** - Use the minimum time range needed for your indicators to calculate
    - **Check for None** - Always pattern match on `Option<T>` before using indicator values
    - **Ensure sufficient data** - Indicators require minimum data points to calculate:
        - Most 14-period indicators need 14+ candles
        - MACD needs ~26+ candles (slow EMA period)
        - Ichimoku needs ~26+ candles
        - Short-period indicators (SMA/EMA 10) need at least 10 candles
        - Candlestick patterns need 3+ candles for three-bar patterns
        - If insufficient data, the indicator returns `None`

    ```rust
    // Good: Store result once, access multiple indicators
    let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

    if let Some(rsi) = indicators.rsi_14 {
        if rsi < 30.0 {
            // Oversold - check other indicators from same result
            if let Some(macd) = &indicators.macd {
                if let (Some(line), Some(signal)) = (macd.macd, macd.signal) {
                    if line > signal {
                        println!("Potential buy: RSI oversold + MACD bullish");
                    }
                }
            }
        }
    }

    // Less efficient: Multiple calls recalculate all indicators
    let rsi_result = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
    if let Some(rsi) = rsi_result.rsi_14 { /* ... */ }
    let macd_result = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
    // Still wastes CPU recalculating all indicators
    ```

## Next Steps

- [Backtesting](backtesting.md) - Use indicators in custom trading strategies
- [Ticker API](ticker.md) - Complete reference for fetching indicators and other data
- [DataFrame Support](dataframe.md) - Convert indicator results to Polars DataFrames for analysis
- [Models Reference](models.md) - Understanding indicator data structures
