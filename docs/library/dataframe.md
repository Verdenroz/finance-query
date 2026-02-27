# DataFrame Support

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — ToDataFrame](https://docs.rs/finance-query/latest/finance_query/derive.ToDataFrame.html)

Finance Query provides optional Polars DataFrame conversion for data analysis workflows.

!!! warning "Feature Flag Required"
    DataFrame support requires the `dataframe` feature flag. Add it to your `Cargo.toml`:

    ```toml
    [dependencies]
    finance-query = { version = "2.0", features = ["dataframe"] }
    polars = "0.45"
    ```

## Overview

The `dataframe` feature enables `.to_dataframe()` methods on many response types, converting them into Polars DataFrames for powerful data manipulation and analysis.

**Supported Types:**

- **Charts** - `Chart`, `Candle`
- **Quotes** - `Quote`, market summary quotes, trending quotes
- **Corporate Events** - `Dividend`, `Split`, `CapitalGain`
- **Screeners** - Screener results
- **Search & Lookup** - Search results, lookup results
- **Options** - Options contracts, options chains
- **Recommendations** - Recommended symbols
- **Sector & Industry** - Company lists, ETFs, performance data
- **News** - News articles
- **Indicators** - Technical indicators summary
- **Market Data** - Exchanges, currencies, market hours

## Basic Usage

### Chart Data

Convert historical OHLCV data to DataFrame:

```rust
use finance_query::{Ticker, Interval, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    // Convert to DataFrame
    let df = chart.to_dataframe()?;

    println!("{}", df);
    // Output:
    // ┌────────────┬────────┬────────┬────────┬────────┬───────────┐
    // │ timestamp  ┆ open   ┆ high   ┆ low    ┆ close  ┆ volume    │
    // │ i64        ┆ f64    ┆ f64    ┆ f64    ┆ f64    ┆ i64       │
    // ╞════════════╪════════╪════════╪════════╪════════╪═══════════╡
    // │ 1234567890 ┆ 150.20 ┆ 152.40 ┆ 149.80 ┆ 151.30 ┆ 45000000  │
    // │ ...        ┆ ...    ┆ ...    ┆ ...    ┆ ...    ┆ ...       │
    // └────────────┴────────┴────────┴────────┴────────┴───────────┘

    Ok(())
}
```

**Chart DataFrame Columns:**

- `timestamp` (i64) - Unix timestamp
- `open` (f64) - Opening price
- `high` (f64) - High price
- `low` (f64) - Low price
- `close` (f64) - Closing price
- `volume` (i64) - Trading volume
- `adj_close` (Option<f64>) - Adjusted close price (accounts for splits/dividends)

### Quote Data

Single quote to DataFrame:

```rust
let ticker = Ticker::new("NVDA").await?;
let quote = ticker.quote().await?;

// Convert to single-row DataFrame
let df = quote.to_dataframe()?;
println!("{}", df);
```

**Quote DataFrame includes 30+ columns** like:

- `symbol`, `short_name`, `exchange`
- `regular_market_price`, `regular_market_change`, `regular_market_change_percent`
- `market_cap`, `volume`, `average_volume`
- `fifty_two_week_high`, `fifty_two_week_low`
- `pe_ratio`, `eps`, `dividend_yield`
- And many more...

### Corporate Events

Convert dividends, splits, or capital gains to DataFrame:

```rust
use finance_query::{Ticker, TimeRange, Dividend, Split, CapitalGain};

let ticker = Ticker::new("AAPL").await?;

// Dividends
let dividends = ticker.dividends(TimeRange::OneYear).await?;
let div_df = Dividend::vec_to_dataframe(&dividends)?;
// Columns: timestamp, amount

// Splits
let splits = ticker.splits(TimeRange::Max).await?;
let split_df = Split::vec_to_dataframe(&splits)?;
// Columns: timestamp, ratio

// Capital gains
let gains = ticker.capital_gains(TimeRange::FiveYears).await?;
let gains_df = CapitalGain::vec_to_dataframe(&gains)?;
// Columns: timestamp, amount
```

### Screener Results

Convert screener results to DataFrame for analysis:

```rust
use finance_query::{finance, Screener};

let gainers = finance::screener(Screener::DayGainers, 50).await?;

// Convert to DataFrame
let df = gainers.to_dataframe()?;
println!("{}", df);
```

### Indicators

!!! note "Feature Flag Required"
    The `indicators()` method requires the `indicators` feature flag:
    ```toml
    finance-query = { version = "2.0", features = ["dataframe", "indicators"] }
    ```

Convert technical indicators to DataFrame:

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("TSLA").await?;
let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

// Convert to single-row DataFrame with all 52 indicators
let df = indicators.to_dataframe()?;

// Access specific indicators
println!("RSI(14): {:?}", df.column("rsi_14")?);
println!("MACD: {:?}", df.column("macd")?);
```

## Working with Polars

### Filtering Data

!!! warning "Polars API Updates"
    The Polars API has evolved significantly. The filtering examples below use an older API style.
    For current Polars 0.52+ API, refer to the [Polars Documentation](https://docs.pola.rs/).

```rust
use polars::prelude::*;

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::SixMonths).await?;
let df = chart.to_dataframe()?;

// For filtering with current Polars API, see Polars documentation
// Older API example (may need updates):
// let high_volume = df.filter(&df.column("volume")?.gt(50_000_000)?)?;

println!("Total days: {}", df.height());
```

### Computing Statistics

```rust
use polars::prelude::*;

let df = chart.to_dataframe()?;

// Calculate average closing price
let avg_close = df.column("close")?
    .mean()
    .unwrap();
println!("Average close: ${:.2}", avg_close);

// Get max high and min low
let max_high = df.column("high")?.max::<f64>().unwrap();
let min_low = df.column("low")?.min::<f64>().unwrap();
println!("Range: ${:.2} - ${:.2}", min_low, max_high);
```

### Adding Calculated Columns

```rust
use polars::prelude::*;

let mut df = chart.to_dataframe()?;

// Add daily return column
let close = df.column("close")?;
let prev_close = close.shift(1);
let daily_return = ((close - &prev_close) / &prev_close) * lit(100.0);

df.with_column(daily_return.alias("daily_return_pct"))?;
```

### Time-based Operations

```rust
use polars::prelude::*;
use chrono::{DateTime, Utc};

let df = chart.to_dataframe()?;

// Convert timestamp to datetime
let dates: Vec<_> = df.column("timestamp")?
    .i64()?
    .into_iter()
    .map(|ts| {
        ts.map(|t| DateTime::from_timestamp(t, 0).unwrap())
    })
    .collect();

// Filter by date range
let start_ts = 1704067200; // 2024-01-01
let df_filtered = df.filter(
    &df.column("timestamp")?.gt_eq(start_ts)?
)?;
```

### Sorting and Ranking

```rust
use polars::prelude::*;

let gainers = finance::screener(Screener::DayGainers, 100).await?;

let mut df = gainers.to_dataframe()?;

// Sort by market cap descending
df = df.sort(["market_cap"], SortMultipleOptions::default().with_order_descending(true))?;

// Get top 10
let top_10 = df.head(Some(10));
println!("{}", top_10);
```

### Aggregations

```rust
use polars::prelude::*;

let df = chart.to_dataframe()?;

// Group by month and aggregate
let monthly = df.lazy()
    .select([
        (col("timestamp") / lit(86400 * 30)).alias("month"),
        col("close").mean().alias("avg_close"),
        col("volume").sum().alias("total_volume"),
        col("high").max().alias("max_high"),
        col("low").min().alias("min_low"),
    ])
    .group_by([col("month")])
    .agg([
        col("avg_close"),
        col("total_volume"),
        col("max_high"),
        col("min_low"),
    ])
    .collect()?;

println!("{}", monthly);
```

## Multiple Symbols

Combine data from multiple symbols:

```rust
use polars::prelude::*;

let aapl = Ticker::new("AAPL").await?;
let msft = Ticker::new("MSFT").await?;
let nvda = Ticker::new("NVDA").await?;

let aapl_chart = aapl.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let msft_chart = msft.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let nvda_chart = nvda.chart(Interval::OneDay, TimeRange::OneMonth).await?;

// Convert to DataFrames
let mut aapl_df = aapl_chart.to_dataframe()?;
let mut msft_df = msft_chart.to_dataframe()?;
let mut nvda_df = nvda_chart.to_dataframe()?;

// Add symbol column to each
aapl_df.with_column(Series::new("symbol", vec!["AAPL"; aapl_df.height()]))?;
msft_df.with_column(Series::new("symbol", vec!["MSFT"; msft_df.height()]))?;
nvda_df.with_column(Series::new("symbol", vec!["NVDA"; nvda_df.height()]))?;

// Combine into single DataFrame
let combined = concat(
    &[aapl_df, msft_df, nvda_df],
    UnionArgs::default(),
)?;

println!("Combined data: {} rows", combined.height());
```

## Exporting Data

### CSV Export

```rust
use polars::prelude::*;
use std::fs::File;

let df = chart.to_dataframe()?;

// Write to CSV
let mut file = File::create("aapl_prices.csv")?;
CsvWriter::new(&mut file)
    .include_header(true)
    .finish(&mut df)?;
```

### Parquet Export

```rust
use polars::prelude::*;
use std::fs::File;

let df = chart.to_dataframe()?;

// Write to Parquet (efficient columnar format)
let file = File::create("aapl_prices.parquet")?;
ParquetWriter::new(file)
    .finish(&mut df)?;
```

### JSON Export

```rust
use polars::prelude::*;
use std::fs::File;

let df = chart.to_dataframe()?;

// Write to JSON
let mut file = File::create("aapl_prices.json")?;
JsonWriter::new(&mut file)
    .finish(&mut df)?;
```

## Advanced Patterns

### Rolling Windows

```rust
use polars::prelude::*;

let df = chart.to_dataframe()?;

// Calculate 20-day moving average
let ma20 = df.lazy()
    .select([
        col("timestamp"),
        col("close"),
        col("close")
            .rolling_mean(RollingOptionsFixedWindow::default().window_size(20))
            .alias("ma_20"),
    ])
    .collect()?;

println!("{}", ma20);
```

### Joining DataFrames

!!! warning "Polars API Updates"
    The join API may require importing additional traits in newer Polars versions.
    Refer to the [Polars Documentation](https://docs.pola.rs/) for current API.

```rust
use polars::prelude::*;

let aapl_chart = aapl.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let aapl_divs = aapl.dividends(TimeRange::OneMonth).await?;

let price_df = aapl_chart.to_dataframe()?;
let div_df = Dividend::vec_to_dataframe(&aapl_divs)?;

// Note: left_join API may require trait imports in newer Polars versions
// Example: use polars_ops::frame::join::DataFrameJoinOps;
// let joined = price_df.left_join(&div_df, ["timestamp"], ["timestamp"])?;
```

### Custom Analysis

```rust
use polars::prelude::*;

let df = chart.to_dataframe()?;

// Calculate daily price range as percentage
let range_pct = df.lazy()
    .select([
        col("timestamp"),
        col("symbol"),
        ((col("high") - col("low")) / col("close") * lit(100.0))
            .alias("range_pct"),
    ])
    .collect()?;

// Find days with highest volatility
let volatile_days = range_pct.sort(
    ["range_pct"],
    SortMultipleOptions::default().with_order_descending(true),
)?
.head(Some(10));

println!("Most volatile days:\n{}", volatile_days);
```

## Type Conversions

### Vec to DataFrame

Many types support converting `Vec<T>` to DataFrame:

```rust
// Vec of dividends to DataFrame
let dividends = ticker.dividends(TimeRange::FiveYears).await?;
let df = Dividend::vec_to_dataframe(&dividends)?;

// SearchQuotes wrapper has to_dataframe() method
let results = finance::search("tech", &SearchOptions::default()).await?;
let df = results.quotes.to_dataframe()?;
```

### Single Item to DataFrame

Individual structs create single-row DataFrames:

```rust
let quote = ticker.quote().await?;
let df = quote.to_dataframe()?;  // 1 row, 30+ columns
```

## Error Handling

DataFrame conversion can fail due to Polars errors:

```rust
use finance_query::Ticker;
use polars::prelude::*;

match chart.to_dataframe() {
    Ok(df) => {
        println!("DataFrame created: {} rows", df.height());
    }
    Err(e) => {
        eprintln!("DataFrame conversion error: {}", e);
    }
}
```

## Best Practices

!!! tip "Combine with Ticker Caching"
    Ticker instances cache data automatically. Fetch once, convert to DataFrame multiple times without additional API calls:

    ```rust
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    // Convert to DataFrame for analysis
    let df = chart.to_dataframe()?;

    // Reuse the same chart data for different analyses
    let high_volume = df.filter(&df.column("volume")?.gt(50_000_000)?)?;
    let recent = df.tail(Some(5));

    // No additional API calls - data is cached in the Ticker
    ```

## Next Steps

- [Ticker API](ticker.md) - Methods that return DataFrame-compatible types
- [Technical Indicators](indicators.md) - Convert indicator results to DataFrames for analysis
- [Backtesting](backtesting.md) - Analyze backtest results in DataFrames
- [Finance Module](finance.md) - Market-wide data with DataFrame support
- [Models](models.md) - All response types and their structures
- [Polars Documentation](https://docs.pola.rs/) - Complete Polars guide
