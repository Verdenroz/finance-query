# DataFrame Support

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — ToDataFrame](https://docs.rs/finance-query/latest/finance_query/derive.ToDataFrame.html)

Finance Query provides optional Polars DataFrame conversion for data analysis workflows.

!!! warning "Feature Flag Required"
    DataFrame support requires the `dataframe` feature flag. Add it to your `Cargo.toml`:

    ```toml
    [dependencies]
    finance-query = { version = "2.0", features = ["dataframe"] }
    polars = "0.53"
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

The Polars conversion path is deliberately **outside the measured performance
gate** — DataFrame construction cost is dominated by Polars itself, not this
crate.

## Basic Usage

### Chart Data

Convert historical OHLCV data to DataFrame:

```rust capture-output feature=dataframe covers=finance_query::models::chart::candle::Candle
use finance_query::{Interval, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    // Convert to DataFrame
    let df = chart.to_dataframe()?;

    println!("{}", df);
    Ok(())
}
```

```text soothfast-output
shape: (23, 7)
┌────────────┬────────────┬────────────┬────────────┬────────────┬──────────┬────────────┐
│ timestamp  ┆ open       ┆ high       ┆ low        ┆ close      ┆ volume   ┆ adj_close  │
│ ---        ┆ ---        ┆ ---        ┆ ---        ┆ ---        ┆ ---      ┆ ---        │
│ i64        ┆ f64        ┆ f64        ┆ f64        ┆ f64        ┆ i64      ┆ f64        │
╞════════════╪════════════╪════════════╪════════════╪════════════╪══════════╪════════════╡
│ 1783517400 ┆ 311.910004 ┆ 314.820007 ┆ 307.049988 ┆ 313.390015 ┆ 41323500 ┆ 313.390015 │
│ 1783603800 ┆ 310.51001  ┆ 316.529999 ┆ 308.160004 ┆ 316.220001 ┆ 48124500 ┆ 316.220001 │
│ 1783690200 ┆ 314.720001 ┆ 316.910004 ┆ 312.170013 ┆ 315.320007 ┆ 34132300 ┆ 315.320007 │
│ 1783949400 ┆ 317.019989 ┆ 323.450012 ┆ 315.779999 ┆ 317.309998 ┆ 43257800 ┆ 317.309998 │
│ 1784035800 ┆ 313.76001  ┆ 316.190002 ┆ 311.910004 ┆ 314.859985 ┆ 36336800 ┆ 314.859985 │
│ …          ┆ …          ┆ …          ┆ …          ┆ …          ┆ …        ┆ …          │
│ 1785763800 ┆ 309.579987 ┆ 311.799988 ┆ 302.559998 ┆ 303.420013 ┆ 75052000 ┆ 303.420013 │
│ 1785850200 ┆ 302.730011 ┆ 310.420013 ┆ 301.320007 ┆ 309.380005 ┆ 68001000 ┆ 309.380005 │
│ 1785936600 ┆ 309.359985 ┆ 311.709991 ┆ 305.670013 ┆ 311.0      ┆ 49438800 ┆ 311.0      │
│ 1786023000 ┆ 314.339996 ┆ 316.290009 ┆ 309.230011 ┆ 312.410004 ┆ 46139900 ┆ 312.410004 │
│ 1786109400 ┆ 311.450012 ┆ 314.809998 ┆ 310.73999  ┆ 313.329987 ┆ 34407100 ┆ 313.329987 │
└────────────┴────────────┴────────────┴────────────┴────────────┴──────────┴────────────┘
```

**Chart DataFrame Columns:**

<!-- soothfast:bind finance_query::models::chart::candle::Candle -->

- `timestamp` (i64) - Unix timestamp
- `open` (f64) - Opening price
- `high` (f64) - High price
- `low` (f64) - Low price
- `close` (f64) - Closing price
- `volume` (i64) - Trading volume
- `adj_close` (Option<f64>) - Adjusted close price (accounts for splits/dividends)

<!-- /soothfast:bind -->

### Quote Data

Single quote to DataFrame:

```rust capture-output feature=dataframe
use finance_query::{Ticker, format::Both};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("NVDA").await?;
    let quote = ticker.quote::<Both>().await?;

    // Convert to single-row DataFrame
    let df = quote.to_dataframe()?;
    println!("{}", df);
    Ok(())
}
```

```text soothfast-output
shape: (1, 154)
┌────────┬──────────┬────────────┬────────────┬───┬────────────┬───────────┬───────────┬───────────┐
│ symbol ┆ logo_url ┆ company_lo ┆ short_name ┆ … ┆ most_recen ┆ price_hin ┆ tradeable ┆ financial │
│ ---    ┆ ---      ┆ go_url     ┆ ---        ┆   ┆ t_quarter  ┆ t         ┆ ---       ┆ _currency │
│ str    ┆ str      ┆ ---        ┆ str        ┆   ┆ ---        ┆ ---       ┆ bool      ┆ ---       │
│        ┆          ┆ str        ┆            ┆   ┆ i64        ┆ i64       ┆           ┆ str       │
╞════════╪══════════╪════════════╪════════════╪═══╪════════════╪═══════════╪═══════════╪═══════════╡
│ NVDA   ┆ null     ┆ null       ┆ NVIDIA Cor ┆ … ┆ 1777161600 ┆ 2         ┆ false     ┆ USD       │
│        ┆          ┆            ┆ poration   ┆   ┆            ┆           ┆           ┆           │
└────────┴──────────┴────────────┴────────────┴───┴────────────┴───────────┴───────────┴───────────┘
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

```rust capture-output feature=dataframe
use finance_query::{CapitalGain, Dividend, Split, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Dividends
    let dividends = ticker.dividends(TimeRange::OneYear).await?;
    let div_df = Dividend::vec_to_dataframe(&dividends)?;
    // Columns: timestamp, amount
    println!("dividends: {:?}", div_df.shape());

    // Splits
    let splits = ticker.splits(TimeRange::Max).await?;
    let split_df = Split::vec_to_dataframe(&splits)?;
    // Columns: timestamp, ratio
    println!("splits: {:?}", split_df.shape());

    // Capital gains (AAPL is a stock, not a fund, so this is typically empty)
    let gains = ticker.capital_gains(TimeRange::FiveYears).await?;
    let gains_df = CapitalGain::vec_to_dataframe(&gains)?;
    // Columns: timestamp, amount
    println!("capital gains: {:?}", gains_df.shape());
    Ok(())
}
```

```text soothfast-output
dividends: (4, 2)
splits: (5, 4)
capital gains: (0, 2)
```

### Screener Results

Convert screener results to DataFrame for analysis:

```rust capture-output feature=dataframe
use finance_query::{Screener, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gainers = finance::screener(Screener::DayGainers, 50).await?;

    // Convert to DataFrame
    let df = gainers.to_dataframe()?;
    println!("{}", df);
    Ok(())
}
```

```text soothfast-output
shape: (50, 59)
┌────────┬────────────┬────────────┬────────────┬───┬───────────┬───────────┬───────────┬──────────┐
│ symbol ┆ short_name ┆ long_name  ┆ display_na ┆ … ┆ earnings_ ┆ earnings_ ┆ earnings_ ┆ currency │
│ ---    ┆ ---        ┆ ---        ┆ me         ┆   ┆ timestamp ┆ timestamp ┆ timestamp ┆ ---      │
│ str    ┆ str        ┆ str        ┆ ---        ┆   ┆ ---       ┆ _start    ┆ _end      ┆ str      │
│        ┆            ┆            ┆ str        ┆   ┆ i64       ┆ ---       ┆ ---       ┆          │
│        ┆            ┆            ┆            ┆   ┆           ┆ i64       ┆ i64       ┆          │
╞════════╪════════════╪════════════╪════════════╪═══╪═══════════╪═══════════╪═══════════╪══════════╡
│ TEAM   ┆ Atlassian  ┆ Atlassian  ┆ Atlassian  ┆ … ┆ 178604640 ┆ 179330400 ┆ 179330400 ┆ USD      │
│        ┆ Corporatio ┆ Corporatio ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ n          ┆ n          ┆            ┆   ┆           ┆           ┆           ┆          │
│ DOCS   ┆ Doximity,  ┆ Doximity,  ┆ Doximity   ┆ … ┆ 178604640 ┆ 179390880 ┆ 179390880 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ FIGS   ┆ FIGS, Inc. ┆ FIGS, Inc. ┆ FIGS       ┆ … ┆ 178604640 ┆ 179390880 ┆ 179390880 ┆ USD      │
│        ┆            ┆            ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ TWLO   ┆ Twilio     ┆ Twilio     ┆ Twilio     ┆ … ┆ 178604640 ┆ 179330400 ┆ 179330400 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ BTG    ┆ B2Gold     ┆ B2Gold     ┆ B2Gold     ┆ … ┆ 178604640 ┆ 179382240 ┆ 179382240 ┆ USD      │
│        ┆ Corp       ┆ Corp.      ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ …      ┆ …          ┆ …          ┆ …          ┆ … ┆ …         ┆ …         ┆ …         ┆ …        │
│ PLSE   ┆ Pulse Bios ┆ Pulse Bios ┆ Pulse Bios ┆ … ┆ 178604640 ┆ 179390880 ┆ 179390880 ┆ USD      │
│        ┆ ciences,   ┆ ciences,   ┆ ciences    ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Inc        ┆ Inc.       ┆            ┆   ┆           ┆           ┆           ┆          │
│ WGS    ┆ GeneDx     ┆ GeneDx     ┆ GeneDx     ┆ … ┆ 178578720 ┆ 179313120 ┆ 179313120 ┆ USD      │
│        ┆ Holdings   ┆ Holdings   ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Corp.      ┆ Corp.      ┆            ┆   ┆           ┆           ┆           ┆          │
│ ARIS   ┆ Aris       ┆ Aris       ┆ Aris       ┆ … ┆ 178535520 ┆ 179321760 ┆ 179321760 ┆ USD      │
│        ┆ Mining Cor ┆ Mining Cor ┆ Mining     ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ poration   ┆ poration   ┆            ┆   ┆           ┆           ┆           ┆          │
│ SSRM   ┆ SSR Mining ┆ SSR Mining ┆ SSR Mining ┆ … ┆ 178587360 ┆ 179373600 ┆ 179373600 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ ERO    ┆ Ero Copper ┆ Ero Copper ┆ Ero Copper ┆ … ┆ 178596000 ┆ 179373600 ┆ 179373600 ┆ USD      │
│        ┆ Corp.      ┆ Corp.      ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
└────────┴────────────┴────────────┴────────────┴───┴───────────┴───────────┴───────────┴──────────┘
```

### Indicators

!!! note "Feature Flag Required"
    The `indicators()` method requires the `indicators` feature flag:
    ```toml
    finance-query = { version = "2.0", features = ["dataframe", "indicators"] }
    ```

Convert technical indicators to DataFrame:

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("TSLA").await?;
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    // Convert to single-row DataFrame with all 52 indicators
    let df = indicators.to_dataframe()?;

    // Access specific scalar indicators. Nested indicators (e.g. `macd`,
    // an Option<MacdData>) are skipped by the derive — read them off the struct.
    println!("RSI(14): {:?}", df.column("rsi_14")?);
    println!("ADX(14): {:?}", df.column("adx_14")?);
    Ok(())
}
```

```text soothfast-output
RSI(14): Scalar(ScalarColumn { name: "rsi_14", scalar: Scalar { dtype: Float64, value: Float64(47.64980926070213) }, length: 1, materialized: OnceLock(shape: (1,)
Series: 'rsi_14' [f64]
[
	47.649809
]) })
ADX(14): Scalar(ScalarColumn { name: "adx_14", scalar: Scalar { dtype: Float64, value: Float64(30.896739718155665) }, length: 1, materialized: OnceLock(shape: (1,)
Series: 'adx_14' [f64]
[
	30.89674
]) })
```

## Working with Polars

The examples below use the Polars 0.53 lazy API (`finance-query`'s `dataframe`
feature enables `polars/lazy`). For the full expression reference, see the
[Polars Documentation](https://docs.pola.rs/).

### Filtering Data

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::SixMonths).await?;
    let df = chart.to_dataframe()?;

    // Keep only high-volume days
    let high_volume = df
        .clone()
        .lazy()
        .filter(col("volume").gt(lit(50_000_000i64)))
        .collect()?;

    println!(
        "Total days: {}, high-volume days: {}",
        df.height(),
        high_volume.height()
    );
    Ok(())
}
```

```text soothfast-output
Total days: 125, high-volume days: 42
```

### Computing Statistics

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::SixMonths).await?;
    let df = chart.to_dataframe()?;

    // Average close, max high, min low in one pass
    let stats = df
        .lazy()
        .select([
            col("close").mean().alias("avg_close"),
            col("high").max().alias("max_high"),
            col("low").min().alias("min_low"),
        ])
        .collect()?;

    let avg_close: f64 = stats.column("avg_close")?.f64()?.get(0).unwrap();
    let max_high: f64 = stats.column("max_high")?.f64()?.get(0).unwrap();
    let min_low: f64 = stats.column("min_low")?.f64()?.get(0).unwrap();
    println!("Average close: ${:.2}", avg_close);
    println!("Range: ${:.2} - ${:.2}", min_low, max_high);
    Ok(())
}
```

```text soothfast-output
Average close: $285.81
Range: $245.51 - $344.57
```

### Adding Calculated Columns

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let df = chart.to_dataframe()?;

    // Add daily return column
    let df = df
        .lazy()
        .with_column(
            ((col("close") - col("close").shift(lit(1))) / col("close").shift(lit(1))
                * lit(100.0))
            .alias("daily_return_pct"),
        )
        .collect()?;

    println!("{:?}", df.column("daily_return_pct")?);
    Ok(())
}
```

```text soothfast-output
Series(SeriesColumn { inner: shape: (23,)
Series: 'daily_return_pct' [f64]
[
	null
	0.903024
	-0.28461
	0.631102
	-0.772119
	…
	-1.777213
	1.964271
	0.523626
	0.453377
	0.294479
], materialized_at: None })
```

### Time-based Operations

```rust capture-output feature=dataframe
use chrono::DateTime;
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
    let df = chart.to_dataframe()?;

    // Convert timestamp to datetime
    let dates: Vec<_> = df
        .column("timestamp")?
        .i64()?
        .into_iter()
        .map(|ts| ts.map(|t| DateTime::from_timestamp(t, 0).unwrap()))
        .collect();
    println!("{} rows", dates.len());

    // Filter by date range
    let start_ts = 1704067200i64; // 2024-01-01
    let df_filtered = df
        .lazy()
        .filter(col("timestamp").gt_eq(lit(start_ts)))
        .collect()?;
    println!("Rows since 2024-01-01: {}", df_filtered.height());
    Ok(())
}
```

```text soothfast-output
251 rows
Rows since 2024-01-01: 251
```

### Sorting and Ranking

```rust capture-output feature=dataframe
use finance_query::{Screener, finance};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gainers = finance::screener(Screener::DayGainers, 100).await?;

    let mut df = gainers.to_dataframe()?;

    // Sort by market cap descending
    df = df.sort(
        ["market_cap"],
        SortMultipleOptions::default().with_order_descending(true),
    )?;

    // Get top 10
    let top_10 = df.head(Some(10));
    println!("{}", top_10);
    Ok(())
}
```

```text soothfast-output
shape: (10, 59)
┌────────┬────────────┬────────────┬────────────┬───┬───────────┬───────────┬───────────┬──────────┐
│ symbol ┆ short_name ┆ long_name  ┆ display_na ┆ … ┆ earnings_ ┆ earnings_ ┆ earnings_ ┆ currency │
│ ---    ┆ ---        ┆ ---        ┆ me         ┆   ┆ timestamp ┆ timestamp ┆ timestamp ┆ ---      │
│ str    ┆ str        ┆ str        ┆ ---        ┆   ┆ ---       ┆ _start    ┆ _end      ┆ str      │
│        ┆            ┆            ┆ str        ┆   ┆ i64       ┆ ---       ┆ ---       ┆          │
│        ┆            ┆            ┆            ┆   ┆           ┆ i64       ┆ i64       ┆          │
╞════════╪════════════╪════════════╪════════════╪═══╪═══════════╪═══════════╪═══════════╪══════════╡
│ SPCX   ┆ Space Expl ┆ Space Expl ┆ Space Expl ┆ … ┆ 178587360 ┆ 179373600 ┆ 179373600 ┆ USD      │
│        ┆ oration    ┆ oration    ┆ oration    ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Technologi ┆ Technologi ┆            ┆   ┆           ┆           ┆           ┆          │
│        ┆ es…        ┆ es…        ┆            ┆   ┆           ┆           ┆           ┆          │
│ PLTR   ┆ Palantir   ┆ Palantir   ┆ Palantir   ┆ … ┆ 178578720 ┆ 179364960 ┆ 179364960 ┆ USD      │
│        ┆ Technologi ┆ Technologi ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ es Inc.    ┆ es Inc.    ┆            ┆   ┆           ┆           ┆           ┆          │
│ ABNB   ┆ Airbnb,    ┆ Airbnb,    ┆ Airbnb     ┆ … ┆ 178604640 ┆ 179390880 ┆ 179390880 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ COHR   ┆ Coherent   ┆ Coherent   ┆ Coherent   ┆ … ┆ 178656480 ┆ 178656480 ┆ 178656480 ┆ USD      │
│        ┆ Corp.      ┆ Corp.      ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ HONA   ┆ Honeywell  ┆ Honeywell  ┆ Honeywell  ┆ … ┆ 178596000 ┆ 179382240 ┆ 179382240 ┆ USD      │
│        ┆ Aerospace  ┆ Aerospace  ┆ Aerospace  ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆           ┆           ┆           ┆          │
│ RKLB   ┆ Rocket Lab ┆ Rocket Lab ┆ Rocket Lab ┆ … ┆ 178639200 ┆ 178639200 ┆ 178639200 ┆ USD      │
│        ┆ Corporatio ┆ Corporatio ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ n          ┆ n          ┆            ┆   ┆           ┆           ┆           ┆          │
│ AU     ┆ AngloGold  ┆ AngloGold  ┆ AngloGold  ┆ … ┆ 178550100 ┆ 178550100 ┆ 178550100 ┆ USD      │
│        ┆ Ashanti    ┆ Ashanti    ┆ Ashanti    ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ PLC        ┆ plc        ┆            ┆   ┆           ┆           ┆           ┆          │
│ CRDO   ┆ Credo      ┆ Credo      ┆ Credo      ┆ … ┆ 178034400 ┆ 178837920 ┆ 178837920 ┆ USD      │
│        ┆ Technology ┆ Technology ┆ Technology ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Group      ┆ Group      ┆ Group      ┆   ┆           ┆           ┆           ┆          │
│        ┆ Holding…   ┆ Holding…   ┆ Holding    ┆   ┆           ┆           ┆           ┆          │
│ NTRA   ┆ Natera,    ┆ Natera,    ┆ Natera     ┆ … ┆ 178604640 ┆ 179390880 ┆ 179390880 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│ AXON   ┆ Axon Enter ┆ Axon Enter ┆ Axon       ┆ … ┆ 178596000 ┆ 179373600 ┆ 179373600 ┆ USD      │
│        ┆ prise,     ┆ prise,     ┆ Enterprise ┆   ┆ 0         ┆ 0         ┆ 0         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆           ┆           ┆           ┆          │
└────────┴────────────┴────────────┴────────────┴───┴───────────┴───────────┴───────────┴──────────┘
```

### Aggregations

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
    let df = chart.to_dataframe()?;

    // Group by (approximate) month and aggregate
    let monthly = df
        .lazy()
        .with_column((col("timestamp") / lit(86400i64 * 30i64)).alias("month"))
        .group_by([col("month")])
        .agg([
            col("close").mean().alias("avg_close"),
            col("volume").sum().alias("total_volume"),
            col("high").max().alias("max_high"),
            col("low").min().alias("min_low"),
        ])
        .collect()?;

    println!("{}", monthly);
    Ok(())
}
```

```text soothfast-output
shape: (14, 5)
┌───────┬────────────┬──────────────┬────────────┬────────────┐
│ month ┆ avg_close  ┆ total_volume ┆ max_high   ┆ min_low    │
│ ---   ┆ ---        ┆ ---          ┆ ---        ┆ ---        │
│ i64   ┆ f64        ┆ i64          ┆ f64        ┆ f64        │
╞═══════╪════════════╪══════════════╪════════════╪════════════╡
│ 679   ┆ 261.201363 ┆ 1086109600   ┆ 277.320007 ┆ 244.0      │
│ 688   ┆ 322.733637 ┆ 1216397200   ┆ 344.570007 ┆ 300.0      │
│ 676   ┆ 229.350006 ┆ 113854000    ┆ 231.0      ┆ 219.25     │
│ 677   ┆ 231.592499 ┆ 943741000    ┆ 241.320007 ┆ 223.779999 │
│ 684   ┆ 253.480499 ┆ 789735900    ┆ 262.480011 ┆ 245.509995 │
│ …     ┆ …          ┆ …            ┆ …          ┆ …          │
│ 689   ┆ 312.246663 ┆ 129985800    ┆ 316.290009 ┆ 305.670013 │
│ 686   ┆ 303.452858 ┆ 1001455700   ┆ 316.940002 ┆ 285.779999 │
│ 682   ┆ 259.249999 ┆ 1186011100   ┆ 279.5      ┆ 243.419998 │
│ 685   ┆ 268.947273 ┆ 1041091400   ┆ 288.029999 ┆ 245.699997 │
│ 678   ┆ 247.65909  ┆ 1257230200   ┆ 259.23999  ┆ 225.949997 │
└───────┴────────────┴──────────────┴────────────┴────────────┘
```

## Multiple Symbols

Combine data from multiple symbols:

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    aapl_df.with_column(Series::new("symbol".into(), vec!["AAPL"; aapl_df.height()]).into())?;
    msft_df.with_column(Series::new("symbol".into(), vec!["MSFT"; msft_df.height()]).into())?;
    nvda_df.with_column(Series::new("symbol".into(), vec!["NVDA"; nvda_df.height()]).into())?;

    // Combine into single DataFrame
    let combined = concat(
        [aapl_df.lazy(), msft_df.lazy(), nvda_df.lazy()],
        UnionArgs::default(),
    )?
    .collect()?;

    println!("Combined data: {} rows", combined.height());
    Ok(())
}
```

```text soothfast-output
Combined data: 69 rows
```

## Exporting Data

The `dataframe` feature enables Polars with only its `lazy` feature. File
writers live behind Polars' own feature flags, so exporting requires adding
`polars` to your `Cargo.toml` with the matching features (`csv`, `parquet`,
`json`):

```toml
polars = { version = "0.53", features = ["lazy", "csv", "parquet", "json"] }
```

### CSV Export

```rust ignore
use polars::prelude::*;
use std::fs::File;

let mut df = chart.to_dataframe()?;

// Write to CSV (requires the polars `csv` feature)
let mut file = File::create("aapl_prices.csv")?;
CsvWriter::new(&mut file)
    .include_header(true)
    .finish(&mut df)?;
```

### Parquet Export

```rust ignore
use polars::prelude::*;
use std::fs::File;

let mut df = chart.to_dataframe()?;

// Write to Parquet (requires the polars `parquet` feature)
let file = File::create("aapl_prices.parquet")?;
ParquetWriter::new(file)
    .finish(&mut df)?;
```

### JSON Export

```rust ignore
use polars::prelude::*;
use std::fs::File;

let mut df = chart.to_dataframe()?;

// Write to JSON (requires the polars `json` feature)
let mut file = File::create("aapl_prices.json")?;
JsonWriter::new(&mut file)
    .finish(&mut df)?;
```

## Advanced Patterns

### Rolling Windows

Rolling aggregations require the Polars `rolling_window` feature in addition
to `lazy`:

```rust ignore
use polars::prelude::*;

let df = chart.to_dataframe()?;

// Calculate 20-day moving average (requires the polars `rolling_window` feature)
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

Requires the `polars-ops` Polars feature (enabled by default alongside `lazy` in finance-query's own `dataframe` feature).

```rust capture-output feature=dataframe
use finance_query::{Dividend, Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aapl = Ticker::new("AAPL").await?;
    let aapl_chart = aapl.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let aapl_divs = aapl.dividends(TimeRange::OneMonth).await?;

    let price_df = aapl_chart.to_dataframe()?;
    let div_df = Dividend::vec_to_dataframe(&aapl_divs)?;

    let joined = price_df.left_join(&div_df, ["timestamp"], ["timestamp"])?;
    println!("joined shape: {:?}", joined.shape());
    Ok(())
}
```

```text soothfast-output
joined shape: (23, 8)
```

### Custom Analysis

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let df = chart.to_dataframe()?;

    // Calculate daily price range as percentage
    let range_pct = df
        .lazy()
        .select([
            col("timestamp"),
            ((col("high") - col("low")) / col("close") * lit(100.0)).alias("range_pct"),
        ])
        .collect()?;

    // Find days with highest volatility
    let volatile_days = range_pct
        .sort(
            ["range_pct"],
            SortMultipleOptions::default().with_order_descending(true),
        )?
        .head(Some(10));

    println!("Most volatile days:\n{}", volatile_days);
    Ok(())
}
```

```text soothfast-output
Most volatile days:
shape: (10, 2)
┌────────────┬───────────┐
│ timestamp  ┆ range_pct │
│ ---        ┆ ---       │
│ i64        ┆ f64       │
╞════════════╪═══════════╡
│ 1784899800 ┆ 3.828599  │
│ 1784122200 ┆ 3.483971  │
│ 1785504600 ┆ 3.460556  │
│ 1784554200 ┆ 3.071129  │
│ 1785763800 ┆ 3.04528   │
│ 1785850200 ┆ 2.941369  │
│ 1783603800 ┆ 2.64689   │
│ 1783517400 ┆ 2.479345  │
│ 1783949400 ┆ 2.417199  │
│ 1784208600 ┆ 2.367516  │
└────────────┴───────────┘
```

## Type Conversions

### Vec to DataFrame

Many types support converting `Vec<T>` to DataFrame:

```rust capture-output feature=dataframe
use finance_query::{Dividend, SearchOptions, Ticker, TimeRange, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Vec of dividends to DataFrame
    let ticker = Ticker::new("AAPL").await?;
    let dividends = ticker.dividends(TimeRange::FiveYears).await?;
    let df = Dividend::vec_to_dataframe(&dividends)?;
    println!("{} dividend rows", df.height());

    // SearchQuotes wrapper has to_dataframe() method
    let results = finance::search("tech", &SearchOptions::default()).await?;
    let df = results.quotes.to_dataframe()?;
    println!("{} rows", df.height());
    Ok(())
}
```

```text soothfast-output
19 dividend rows
7 rows
```

<!-- soothfast:bind finance_query::models::chart::events::Dividend -->
The conversion itself needs no network. Response types are `#[non_exhaustive]`
and cannot be constructed literally, but any serde-compatible source works —
this example runs as a real test on a fixture value:

```rust capture-output feature=dataframe covers=finance_query::models::chart::events::Dividend
use finance_query::Dividend;

let dividends: Vec<Dividend> = serde_json::from_str(
    r#"[{"timestamp": 1704067200, "amount": 0.24},
        {"timestamp": 1711929600, "amount": 0.25}]"#,
)
.unwrap();

let df = Dividend::vec_to_dataframe(&dividends).unwrap();
assert_eq!(df.height(), 2);
assert!(df.column("timestamp").is_ok());
assert!(df.column("amount").is_ok());
println!("rows = {}", df.height());
println!("columns = {:?}", df.get_column_names());
```

```text soothfast-output
rows = 2
columns = ["timestamp", "amount"]
```
<!-- /soothfast:bind -->

### Single Item to DataFrame

Individual structs create single-row DataFrames:

```rust capture-output feature=dataframe
use finance_query::{Ticker, format::Both};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let quote = ticker.quote::<Both>().await?;
    let df = quote.to_dataframe()?; // 1 row, 30+ columns
    println!("{} columns", df.width());
    Ok(())
}
```

```text soothfast-output
154 columns
```

## Error Handling

DataFrame conversion can fail due to Polars errors:

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    match chart.to_dataframe() {
        Ok(df) => {
            println!("DataFrame created: {} rows", df.height());
        }
        Err(e) => {
            eprintln!("DataFrame conversion error: {}", e);
        }
    }
    Ok(())
}
```

```text soothfast-output
DataFrame created: 23 rows
```

## Best Practices

!!! tip "Combine with Ticker Caching"
    Ticker instances cache data automatically. Fetch once, convert to DataFrame multiple times without additional API calls:

    ```rust no_run feature=dataframe
    use finance_query::{Interval, Ticker, TimeRange};
    use polars::prelude::*;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let ticker = Ticker::new("AAPL").await?;
        let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

        // Convert to DataFrame for analysis
        let df = chart.to_dataframe()?;

        // Reuse the same chart data for different analyses
        let high_volume = df
            .clone()
            .lazy()
            .filter(col("volume").gt(lit(50_000_000i64)))
            .collect()?;
        let recent = df.tail(Some(5));

        // No additional API calls - data is cached in the Ticker
        println!("{} high-volume days, recent:\n{}", high_volume.height(), recent);
        Ok(())
    }
    ```

## Next Steps

- [Ticker API](ticker.md) - Methods that return DataFrame-compatible types
- [Technical Indicators](indicators.md) - Convert indicator results to DataFrames for analysis
- [Backtesting](backtesting.md) - Analyze backtest results in DataFrames
- [Finance Module](finance.md) - Market-wide data with DataFrame support
- [Polars Documentation](https://docs.pola.rs/) - Complete Polars guide
