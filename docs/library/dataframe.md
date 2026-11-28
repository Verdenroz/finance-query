# DataFrame Support

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — ToDataFrame](https://docs.rs/finance-query/latest/finance_query/derive.ToDataFrame.html)

Finance Query provides optional Polars DataFrame conversion for data analysis workflows.

!!! warning "Feature Flag Required"
    DataFrame support requires the `dataframe` feature flag. Add it to your `Cargo.toml`:

    ```toml
    [dependencies]
    finance-query = { version = "3", features = ["dataframe"] }
    polars = { version = "0.55", default-features = false, features = ["lazy", "csv", "parquet", "json", "rolling_window", "temporal"] }
    ```

    The `finance_query::polars` re-export exists so type names in public
    signatures resolve to the same version. It enables only `lazy`,
    `polars-ops`, `fmt` and `dtype-struct`, so add `polars` yourself for the
    features your code needs. Cargo unifies the two into one build. Drop
    `csv`, `parquet`, `json` and `rolling_window` from the line above if you
    don't use them.

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

```rust capture-output feature=dataframe covers=finance_query::Candle
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
shape: (21, 8)
┌────────────┬────────────┬────────────┬────────────┬───────────┬──────────┬───────────┬───────────┐
│ timestamp  ┆ open       ┆ high       ┆ low        ┆ close     ┆ volume   ┆ adj_close ┆ provider_ │
│ ---        ┆ ---        ┆ ---        ┆ ---        ┆ ---       ┆ ---      ┆ ---       ┆ id        │
│ datetime[μ ┆ f64        ┆ f64        ┆ f64        ┆ f64       ┆ i64      ┆ f64       ┆ ---       │
│ s, UTC]    ┆            ┆            ┆            ┆           ┆          ┆           ┆ str       │
╞════════════╪════════════╪════════════╪════════════╪═══════════╪══════════╪═══════════╪═══════════╡
│ 2026-10-28 ┆ 346.920013 ┆ 347.179993 ┆ 343.519989 ┆ 343.82998 ┆ 35714300 ┆ 343.56893 ┆ yahoo     │
│ 13:30:00   ┆            ┆            ┆            ┆ 7         ┆          ┆ 9         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-10-29 ┆ 348.0      ┆ 351.429993 ┆ 347.299988 ┆ 348.91000 ┆ 42582100 ┆ 348.64511 ┆ yahoo     │
│ 13:30:00   ┆            ┆            ┆            ┆ 4         ┆          ┆ 1         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-10-30 ┆ 346.720001 ┆ 349.190002 ┆ 334.350006 ┆ 344.63000 ┆ 67841600 ┆ 344.36834 ┆ yahoo     │
│ 13:30:00   ┆            ┆            ┆            ┆ 5         ┆          ┆ 7         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-02 ┆ 342.670013 ┆ 343.799988 ┆ 335.820007 ┆ 338.32000 ┆ 64757700 ┆ 338.06314 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 7         ┆          ┆ 1         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-03 ┆ 325.579987 ┆ 334.5      ┆ 325.329987 ┆ 331.89999 ┆ 63160000 ┆ 331.64801 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 4         ┆          ┆           ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ …          ┆ …          ┆ …          ┆ …          ┆ …         ┆ …        ┆ …         ┆ …         │
│ 2026-11-19 ┆ 352.910004 ┆ 355.279999 ┆ 346.470001 ┆ 349.20999 ┆ 40906900 ┆ 349.20999 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 1         ┆          ┆ 1         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-20 ┆ 352.869995 ┆ 354.339996 ┆ 340.559998 ┆ 342.55999 ┆ 80692500 ┆ 342.55999 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 8         ┆          ┆ 8         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-23 ┆ 347.869995 ┆ 352.600006 ┆ 345.440002 ┆ 349.07998 ┆ 58588700 ┆ 349.07998 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 7         ┆          ┆ 7         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-24 ┆ 348.059998 ┆ 354.399994 ┆ 345.880005 ┆ 354.07000 ┆ 41270400 ┆ 354.07000 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 7         ┆          ┆ 7         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
│ 2026-11-25 ┆ 353.480011 ┆ 356.75     ┆ 345.130005 ┆ 355.57998 ┆ 54285300 ┆ 355.57998 ┆ yahoo     │
│ 14:30:00   ┆            ┆            ┆            ┆ 7         ┆          ┆ 7         ┆           │
│ UTC        ┆            ┆            ┆            ┆           ┆          ┆           ┆           │
└────────────┴────────────┴────────────┴────────────┴───────────┴──────────┴───────────┴───────────┘
```

**Chart DataFrame Columns:**

<!-- soothfast:bind finance_query::Candle -->

- `timestamp` (Datetime, UTC) - Bar time
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
│        ┆          ┆ str        ┆            ┆   ┆ date       ┆ i64       ┆           ┆ str       │
╞════════╪══════════╪════════════╪════════════╪═══╪════════════╪═══════════╪═══════════╪═══════════╡
│ NVDA   ┆ null     ┆ null       ┆ NVIDIA Cor ┆ … ┆ 2026-10-25 ┆ 2         ┆ false     ┆ USD       │
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
dividends: (4, 3)
splits: (5, 5)
capital gains: (0, 3)
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
│        ┆            ┆            ┆ str        ┆   ┆ datetime[ ┆ ---       ┆ ---       ┆          │
│        ┆            ┆            ┆            ┆   ┆ μs, UTC]  ┆ datetime[ ┆ datetime[ ┆          │
│        ┆            ┆            ┆            ┆   ┆           ┆ μs, UTC]  ┆ μs, UTC]  ┆          │
╞════════╪════════════╪════════════╪════════════╪═══╪═══════════╪═══════════╪═══════════╪══════════╡
│ GUNN   ┆ Gunnison   ┆ Gunnison   ┆ Gunnison   ┆ … ┆ null      ┆ null      ┆ null      ┆ USD      │
│        ┆ Copper     ┆ Copper     ┆ Copper     ┆   ┆           ┆           ┆           ┆          │
│        ┆ Corp.      ┆ Corp.      ┆            ┆   ┆           ┆           ┆           ┆          │
│ XP     ┆ XP Inc.    ┆ XP Inc.    ┆ XP         ┆ … ┆ 2026-11-1 ┆ 2027-02-1 ┆ 2027-02-1 ┆ USD      │
│        ┆            ┆            ┆            ┆   ┆ 6         ┆ 1         ┆ 1         ┆          │
│        ┆            ┆            ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ VERA   ┆ Vera Thera ┆ Vera Thera ┆ Vera Thera ┆ … ┆ 2026-11-0 ┆ 2026-11-0 ┆ 2026-11-0 ┆ USD      │
│        ┆ peutics,   ┆ peutics,   ┆ peutics    ┆   ┆ 5         ┆ 5         ┆ 5         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ SNDK   ┆ Sandisk    ┆ Sandisk    ┆ Sandisk    ┆ … ┆ 2026-10-2 ┆ 2027-02-1 ┆ 2027-02-1 ┆ USD      │
│        ┆ Corporatio ┆ Corporatio ┆            ┆   ┆ 9         ┆ 5         ┆ 5         ┆          │
│        ┆ n          ┆ n          ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ AYA    ┆ Aya Gold & ┆ Aya Gold & ┆ Aya Gold & ┆ … ┆ 2026-11-1 ┆ 2026-11-1 ┆ 2026-11-1 ┆ USD      │
│        ┆ Silver     ┆ Silver     ┆ Silver     ┆   ┆ 2         ┆ 2         ┆ 2         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 13:30:00  ┆ 13:30:00  ┆ 13:30:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ …      ┆ …          ┆ …          ┆ …          ┆ … ┆ …         ┆ …         ┆ …         ┆ …        │
│ NTSK   ┆ Netskope,  ┆ Netskope,  ┆ Netskope   ┆ … ┆ 2026-12-0 ┆ 2026-12-0 ┆ 2026-12-0 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 2         ┆ 2         ┆ 2         ┆          │
│        ┆            ┆            ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ MU     ┆ Micron Tec ┆ Micron Tec ┆ Micron     ┆ … ┆ 2026-12-1 ┆ 2026-12-1 ┆ 2026-12-1 ┆ USD      │
│        ┆ hnology,   ┆ hnology,   ┆            ┆   ┆ 7         ┆ 7         ┆ 7         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ NXT    ┆ Nextpower  ┆ Nextpower  ┆ Nextpower  ┆ … ┆ 2026-10-2 ┆ 2027-01-2 ┆ 2027-01-2 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 8         ┆ 7         ┆ 7         ┆          │
│        ┆            ┆            ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ SKHY   ┆ SK hynix   ┆ SK hynix   ┆ SK hynix   ┆ … ┆ 2026-10-2 ┆ 2027-01-2 ┆ 2027-01-2 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 7         ┆ 7         ┆ 7         ┆          │
│        ┆            ┆            ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ AEM    ┆ Agnico     ┆ Agnico     ┆ Agnico     ┆ … ┆ 2026-10-2 ┆ 2027-02-1 ┆ 2027-02-1 ┆ USD      │
│        ┆ Eagle      ┆ Eagle      ┆ Eagle      ┆   ┆ 8         ┆ 1         ┆ 1         ┆          │
│        ┆ Mines      ┆ Mines      ┆ Mines      ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆ Limited    ┆ Limited    ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
└────────┴────────────┴────────────┴────────────┴───┴───────────┴───────────┴───────────┴──────────┘
```

### Indicators

!!! note "Feature Flag Required"
    The `indicators()` method requires the `indicators` feature flag:
    ```toml
    finance-query = { version = "3", features = ["dataframe", "indicators"] }
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

    // Convert to a single-row DataFrame. Composite indicators are flattened
    // into prefixed columns, e.g. `macd` becomes `macd_macd`, `macd_signal`
    // and `macd_histogram`.
    let df = indicators.to_dataframe()?;

    println!("RSI(14): {:?}", df.column("rsi_14")?);
    println!("ADX(14): {:?}", df.column("adx_14")?);
    Ok(())
}
```

```text soothfast-output
RSI(14): Scalar(ScalarColumn { name: "rsi_14", scalar: Scalar { dtype: Float64, value: Float64(50.379332821543464) }, length: 1, materialized: OnceLock(shape: (1,)
Series: 'rsi_14' [f64]
[
	50.379333
]) })
ADX(14): Scalar(ScalarColumn { name: "adx_14", scalar: Scalar { dtype: Float64, value: Float64(11.338209049151558) }, length: 1, materialized: OnceLock(shape: (1,)
Series: 'adx_14' [f64]
[
	11.338209
]) })
```

## Working with Polars

The examples below use the Polars 0.55 lazy API (`finance-query`'s `dataframe`
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
Total days: 127, high-volume days: 44
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
Average close: $324.34
Range: $273.75 - $360.83
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
Series(SeriesColumn { inner: shape: (21,)
Series: 'daily_return_pct' [f64]
[
	null
	1.477479
	-1.226677
	-1.830948
	-1.897616
	…
	-0.919285
	-1.904297
	1.903313
	1.429478
	0.426463
], materialized_at: None })
```

### Time-based Operations

Chart timestamps are `Datetime(us, UTC)` columns, so type the literal you
compare against. The `dt()` namespace needs the Polars `temporal` feature.

```rust capture-output feature=dataframe
use chrono::{Duration, Utc};
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
    let df = chart.to_dataframe()?;
    let total = df.height();

    // Keep the last six months
    let cutoff = lit((Utc::now() - Duration::days(182)).timestamp_micros())
        .cast(DataType::Datetime(TimeUnit::Microseconds, Some(TimeZone::UTC)));
    let recent = df
        .lazy()
        .filter(col("timestamp").gt_eq(cutoff))
        .collect()?;
    println!("Filtered to fewer rows: {}", recent.height() < total);
    Ok(())
}
```

```text soothfast-output
Filtered to fewer rows: true
```

Calendar operations such as `dt().date()`, `dt().year()` and `dt().month()`
work on the column directly.

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
│        ┆            ┆            ┆ str        ┆   ┆ datetime[ ┆ ---       ┆ ---       ┆          │
│        ┆            ┆            ┆            ┆   ┆ μs, UTC]  ┆ datetime[ ┆ datetime[ ┆          │
│        ┆            ┆            ┆            ┆   ┆           ┆ μs, UTC]  ┆ μs, UTC]  ┆          │
╞════════╪════════════╪════════════╪════════════╪═══╪═══════════╪═══════════╪═══════════╪══════════╡
│ ANT    ┆ Anthropic, ┆ Anthropic, ┆ null       ┆ … ┆ null      ┆ null      ┆ null      ┆ USD      │
│        ┆ PBC        ┆ PBC        ┆            ┆   ┆           ┆           ┆           ┆          │
│ SKHY   ┆ SK hynix   ┆ SK hynix   ┆ SK hynix   ┆ … ┆ 2026-10-2 ┆ 2027-01-2 ┆ 2027-01-2 ┆ USD      │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 7         ┆ 7         ┆ 7         ┆          │
│        ┆            ┆            ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ MU     ┆ Micron Tec ┆ Micron Tec ┆ Micron     ┆ … ┆ 2026-12-1 ┆ 2026-12-1 ┆ 2026-12-1 ┆ USD      │
│        ┆ hnology,   ┆ hnology,   ┆            ┆   ┆ 7         ┆ 7         ┆ 7         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ ANET   ┆ Arista     ┆ Arista     ┆ Arista     ┆ … ┆ 2026-11-0 ┆ 2027-02-0 ┆ 2027-02-0 ┆ USD      │
│        ┆ Networks,  ┆ Networks,  ┆ Networks   ┆   ┆ 3         ┆ 9         ┆ 9         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ DELL   ┆ Dell Techn ┆ Dell Techn ┆ Dell       ┆ … ┆ 2026-11-2 ┆ 2027-02-2 ┆ 2027-02-2 ┆ USD      │
│        ┆ ologies    ┆ ologies    ┆            ┆   ┆ 4         ┆ 5         ┆ 5         ┆          │
│        ┆ Inc.       ┆ Inc.       ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ QCOM   ┆ QUALCOMM   ┆ QUALCOMM   ┆ QUALCOMM   ┆ … ┆ 2026-11-0 ┆ 2027-02-0 ┆ 2027-02-0 ┆ USD      │
│        ┆ Incorporat ┆ Incorporat ┆            ┆   ┆ 4         ┆ 3         ┆ 3         ┆          │
│        ┆ ed         ┆ ed         ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ MRVL   ┆ Marvell    ┆ Marvell    ┆ Marvell    ┆ … ┆ 2026-12-0 ┆ 2026-12-0 ┆ 2026-12-0 ┆ USD      │
│        ┆ Technology ┆ Technology ┆            ┆   ┆ 1         ┆ 1         ┆ 1         ┆          │
│        ┆ , Inc.     ┆ , Inc.     ┆            ┆   ┆ 21:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ STX    ┆ Seagate    ┆ Seagate    ┆ Seagate    ┆ … ┆ 2026-10-2 ┆ 2027-01-2 ┆ 2027-01-2 ┆ USD      │
│        ┆ Technology ┆ Technology ┆            ┆   ┆ 7         ┆ 6         ┆ 6         ┆          │
│        ┆ Holdings   ┆ Holdings   ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆ PL…        ┆ pl…        ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ WDC    ┆ Western    ┆ Western    ┆ Western    ┆ … ┆ 2026-10-2 ┆ 2027-01-2 ┆ 2027-01-2 ┆ USD      │
│        ┆ Digital    ┆ Digital    ┆ Digital    ┆   ┆ 2         ┆ 8         ┆ 8         ┆          │
│        ┆ Corporatio ┆ Corporatio ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆ n          ┆ n          ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
│ SNDK   ┆ Sandisk    ┆ Sandisk    ┆ Sandisk    ┆ … ┆ 2026-10-2 ┆ 2027-02-1 ┆ 2027-02-1 ┆ USD      │
│        ┆ Corporatio ┆ Corporatio ┆            ┆   ┆ 9         ┆ 5         ┆ 5         ┆          │
│        ┆ n          ┆ n          ┆            ┆   ┆ 20:00:00  ┆ 21:00:00  ┆ 21:00:00  ┆          │
│        ┆            ┆            ┆            ┆   ┆ UTC       ┆ UTC       ┆ UTC       ┆          │
└────────┴────────────┴────────────┴────────────┴───┴───────────┴───────────┴───────────┴──────────┘
```

### Aggregations

A lazy `collect()` on a `current_thread` tokio runtime (such as `#[tokio::test]`)
must run inside `tokio::task::spawn_blocking`.

```rust capture-output feature=dataframe
use finance_query::{Interval, Ticker, TimeRange};
use polars::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
    let df = chart.to_dataframe()?;

    // Average close per calendar month
    let monthly = df
        .lazy()
        .with_columns([
            col("timestamp").dt().year().alias("year"),
            col("timestamp").dt().month().alias("month"),
        ])
        .group_by([col("year"), col("month")])
        .agg([col("close").mean().alias("close_mean")])
        .sort(["year", "month"], SortMultipleOptions::default())
        .collect()?;

    println!("{}", monthly);
    Ok(())
}
```

```text soothfast-output
shape: (13, 3)
┌──────┬───────┬────────────┐
│ year ┆ month ┆ close_mean │
│ ---  ┆ ---   ┆ ---        │
│ i32  ┆ i8    ┆ f64        │
╞══════╪═══════╪════════════╡
│ 2025 ┆ 11    ┆ 278.850006 │
│ 2025 ┆ 12    ┆ 276.306362 │
│ 2026 ┆ 1     ┆ 257.649498 │
│ 2026 ┆ 2     ┆ 269.178422 │
│ 2026 ┆ 3     ┆ 254.906363 │
│ …    ┆ …     ┆ …          │
│ 2026 ┆ 7     ┆ 322.288637 │
│ 2026 ┆ 8     ┆ 310.19381  │
│ 2026 ┆ 9     ┆ 331.056665 │
│ 2026 ┆ 10    ┆ 341.167275 │
│ 2026 ┆ 11    ┆ 348.882779 │
└──────┴───────┴────────────┘
```

## Multiple Symbols

The batch responses from `Tickers` convert directly. `to_dataframe()` stacks
every symbol into one frame with a leading `symbol` column, in input order.
Symbols that failed are listed in `errors` and left out of the frame.

```rust capture-output feature=dataframe
use finance_query::{Interval, Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT", "NVDA"]).await?;
    let response = tickers
        .charts(Interval::OneDay, TimeRange::OneMonth)
        .await?;

    let combined = response.to_dataframe()?;
    println!("Combined data: {} rows", combined.height());
    Ok(())
}
```

```text soothfast-output
Combined data: 63 rows
```

`BatchQuotesResponse`, `BatchSparksResponse`, `BatchDividendsResponse`,
`BatchSplitsResponse`, `BatchCapitalGainsResponse`, `BatchNewsResponse`,
`BatchOptionsResponse`, `BatchRecommendationsResponse` and
`BatchFinancialsResponse` have the same method.

## Financial Statements

`FinancialStatement::to_dataframe()` returns a long frame with one row per
metric and date. `to_wide_dataframe()` returns one row per date and one
nullable `f64` column per metric:

```rust no_run feature=dataframe
use finance_query::{Frequency, StatementType, Ticker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let income = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await?;

    let wide = income.to_wide_dataframe()?;
    println!("{}", wide);
    Ok(())
}
```

The `date` column is a polars `Date` sorted ascending. Metric names are
snake_case in both frames, whatever casing the provider uses: Yahoo's
`TotalRevenue` and FMP's `totalRevenue` both become `total_revenue`. The
statement map itself (`income.statement`) keeps the provider's names.

## Exporting Data

File writers live behind Polars' own feature flags, so exporting requires the
matching features (`csv`, `parquet`, `json`) on the `polars` line from the top
of this page.

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

CSV has no list type, so `CsvWriter` returns an error for a frame with a `List(String)` column. Frames from `SearchNews`, `EdgarSearchSource`, `FilingSearchHit`, `ProtocolTvl` and `FundProfile` have such columns (for example `related_tickers`, `ciks`, `chains` and `brokerages`). Drop the column, or cast or join it to a `String`, before writing CSV. Parquet and JSON handle list columns as they are.

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
joined shape: (21, 10)
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
┌─────────────────────────┬───────────┐
│ timestamp               ┆ range_pct │
│ ---                     ┆ ---       │
│ datetime[μs, UTC]       ┆ f64       │
╞═════════════════════════╪═══════════╡
│ 2026-10-30 13:30:00 UTC ┆ 4.306066  │
│ 2026-11-20 14:30:00 UTC ┆ 4.022653  │
│ 2026-11-05 14:30:00 UTC ┆ 3.870182  │
│ 2026-11-04 14:30:00 UTC ┆ 3.865414  │
│ 2026-11-25 14:30:00 UTC ┆ 3.267899  │
│ 2026-11-03 14:30:00 UTC ┆ 2.762884  │
│ 2026-11-19 14:30:00 UTC ┆ 2.522837  │
│ 2026-11-24 14:30:00 UTC ┆ 2.406301  │
│ 2026-11-02 14:30:00 UTC ┆ 2.358708  │
│ 2026-11-06 14:30:00 UTC ┆ 2.187698  │
└─────────────────────────┴───────────┘
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
20 dividend rows
7 rows
```

<!-- soothfast:bind finance_query::Dividend -->
The conversion itself needs no network. Response types are `#[non_exhaustive]`
and cannot be constructed literally, but any serde-compatible source works —
this example runs as a real test on a fixture value:

```rust capture-output feature=dataframe covers=finance_query::Dividend
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
columns = ["timestamp", "amount", "provider_id"]
```
<!-- /soothfast:bind -->

### Vec and Slices

`Vec<T>` and `[T]` implement `ToDataFrame` for every `T` that does, so a list of rows converts with method syntax. Import the prelude to bring the trait into scope:

```rust feature=dataframe
use finance_query::Dividend;
use finance_query::prelude::*;

let dividends: Vec<Dividend> = serde_json::from_str(
    r#"[{"timestamp": 1704067200, "amount": 0.24},
        {"timestamp": 1711929600, "amount": 0.25}]"#,
)
.unwrap();

assert_eq!(dividends.to_dataframe().unwrap().height(), 2);
assert_eq!(dividends[..1].to_dataframe().unwrap().height(), 1);
```

An empty list keeps the row type's schema for row types such as `Candle` or `Quote`; types that are already multi-row frames, such as `Chart` or `FinancialStatement`, give an empty frame with no columns. `Vec` and slice frames stack one row per item, so `candles.to_dataframe()` equals `Candle::vec_to_dataframe(&candles)`.

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

### Nested Rows

Types that hold a list of rows, such as the holdings of a fund, keep the parent frame flat so it stays CSV-safe. Each list has its own child frame through a `{field}_dataframe()` method. `to_nested_dataframe()` (and `vec_to_nested_dataframe()` for a slice) instead keeps the rows inside the parent as a `List(Struct)` column.

```rust feature=dataframe
use finance_query::quote::TopHoldings;

let top: TopHoldings = serde_json::from_str(
    r#"{"holdings": [
        {"symbol": "AAPL", "holdingName": "Apple Inc", "holdingPercent": {"raw": 0.07}},
        {"symbol": "MSFT", "holdingName": "Microsoft Corp", "holdingPercent": {"raw": 0.06}}
    ]}"#,
)
.unwrap();

let holdings = top.holdings_dataframe().unwrap();
assert_eq!(holdings.height(), 2);
println!("{:?}", holdings.get_column_names());

let nested = top.to_nested_dataframe().unwrap();
assert_eq!(nested.height(), 1);
println!("{:?}", nested.column("holdings").unwrap().dtype());
```

Nested and list columns cannot be written to CSV. Write the flat parent frame or a child frame instead.

### List Columns

A `Vec<String>` field marked as a list becomes a `List(String)` column. EDGAR submissions and full-text search hits use this for `tickers`, `exchanges`, `ciks` and similar fields.

```rust feature=dataframe
use finance_query::EdgarSearchSource;

let hit: EdgarSearchSource = serde_json::from_str(
    r#"{"form": "10-K", "ciks": ["0000320193"], "display_names": ["Apple Inc. (AAPL)"]}"#,
)
.unwrap();

let df = hit.to_dataframe().unwrap();
println!("{:?}", df.column("ciks").unwrap().dtype());
```

## Series and Streaming Frames

Indicator results hold one series per field, so their frames have one row per bar. Series shorter than the longest are padded with trailing nulls. There is no timestamp column, so `hstack` the frame onto the candle frame it was computed from.

```rust feature=dataframe,indicators
use finance_query::MacdResult;

let macd = MacdResult {
    macd_line: vec![None, Some(1.0), Some(1.5)],
    signal_line: vec![None, None, Some(1.2)],
    histogram: vec![None, None, Some(0.3)],
};

let df = macd.to_dataframe().unwrap();
assert_eq!(df.height(), 3);
println!("{:?}", df.get_column_names());
```

Sparks are long format, one row per point. `Spark::meta_dataframe()` returns the chart metadata as one row, and `BatchSparksResponse::to_dataframe()` stacks every symbol with a leading `symbol` column.

Streaming events convert the same way. `PriceUpdate`, `OrderBookUpdate`, `TradeTick`, `OptionContractUpdate` and `AlertEvent` all have `vec_to_dataframe`, so collect a batch of events and convert it once. An `AlertEvent` frame carries the triggering tick as `update_*` columns plus a `condition_kind` column.

```rust feature=dataframe
use finance_query::streaming::PriceUpdate;

let ticks: Vec<PriceUpdate> = ["AAPL", "MSFT"]
    .into_iter()
    .map(|id| PriceUpdate {
        id: id.to_string(),
        ..PriceUpdate::default()
    })
    .collect();

let df = PriceUpdate::vec_to_dataframe(&ticks).unwrap();
assert_eq!(df.height(), 2);
println!("{}", df.width());
```

`FearAndGreed`, `QuoteSummaryResponse` and `DividendAnalytics` also have `to_dataframe()`, each as a single row.

### Company Facts

`CompanyFacts::to_dataframe()` and `FactsByTaxonomy::to_dataframe()` return every data point in long format. Columns are `taxonomy` (company facts only), `concept` and `unit`, followed by the fact columns. Rows are sorted by taxonomy, concept and unit.

```rust feature=dataframe
use finance_query::CompanyFacts;

let facts: CompanyFacts = serde_json::from_str(
    r#"{"cik": 320193, "entityName": "Apple Inc.", "facts": {"us-gaap": {
        "Revenues": {"units": {"USD": [
            {"end": "2023-09-30", "val": 383285000000.0, "fy": 2023, "fp": "FY", "form": "10-K"},
            {"end": "2024-09-28", "val": 391035000000.0, "fy": 2024, "fp": "FY", "form": "10-K"}
        ]}}
    }}}"#,
)
.unwrap();

let df = facts.to_dataframe().unwrap();
assert_eq!(df.height(), 2);
println!("{:?}", &df.get_column_names()[..3]);
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
DataFrame created: 21 rows
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
