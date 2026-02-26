//! Compile and runtime tests for docs/library/dataframe.md
//!
//! Requires the `dataframe` feature flag:
//!   cargo test --test doc_dataframe --features dataframe
//!   cargo test --test doc_dataframe --features dataframe -- --ignored   (network tests)
//!
//! Skipped sections (require additional Polars features not enabled by `dataframe`):
//!   - "CSV Export"     — requires polars/csv feature
//!   - "Parquet Export" — requires polars/parquet feature
//!   - "JSON Export"    — requires polars/json feature
//!   - "Rolling Windows" — RollingOptionsFixedWindow not in polars/lazy
//!
//! Polars operations using the older Series/Column direct arithmetic API (noted in the
//! doc as "older API style") are expressed via the equivalent lazy API.

#![cfg(feature = "dataframe")]

use finance_query::{Interval, TimeRange};

// ---------------------------------------------------------------------------
// Network tests — Chart Data (dataframe.md "Chart Data" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_chart_to_dataframe() {
    use finance_query::Ticker;

    // From dataframe.md "Chart Data" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Convert to DataFrame
    let df = chart.to_dataframe().unwrap();
    println!("{}", df);

    // Verify documented columns exist
    assert!(df.column("timestamp").is_ok());
    assert!(df.column("open").is_ok());
    assert!(df.column("high").is_ok());
    assert!(df.column("low").is_ok());
    assert!(df.column("close").is_ok());
    assert!(df.column("volume").is_ok());
    assert!(df.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Quote Data (dataframe.md "Quote Data" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_quote_to_dataframe() {
    use finance_query::Ticker;

    // From dataframe.md "Quote Data" section
    let ticker = Ticker::new("NVDA").await.unwrap();
    let quote = ticker.quote().await.unwrap();

    // Convert to single-row DataFrame
    let df = quote.to_dataframe().unwrap();
    println!("{}", df);

    assert_eq!(df.height(), 1);
}

// ---------------------------------------------------------------------------
// Network tests — Corporate Events (dataframe.md "Corporate Events" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_corporate_events_to_dataframe() {
    use finance_query::{CapitalGain, Dividend, Split, Ticker, TimeRange};

    // From dataframe.md "Corporate Events" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Dividends
    let dividends = ticker.dividends(TimeRange::OneYear).await.unwrap();
    let div_df = Dividend::vec_to_dataframe(&dividends).unwrap();
    // Columns: timestamp, amount
    assert!(div_df.column("timestamp").is_ok());
    assert!(div_df.column("amount").is_ok());

    // Splits
    let splits = ticker.splits(TimeRange::Max).await.unwrap();
    let split_df = Split::vec_to_dataframe(&splits).unwrap();
    // Columns: timestamp, ratio
    assert!(split_df.column("timestamp").is_ok());
    assert!(split_df.column("ratio").is_ok());

    // Capital gains
    let gains = ticker.capital_gains(TimeRange::FiveYears).await.unwrap();
    let gains_df = CapitalGain::vec_to_dataframe(&gains).unwrap();
    // Columns: timestamp, amount
    assert!(gains_df.column("timestamp").is_ok());
    assert!(gains_df.column("amount").is_ok());
}

// ---------------------------------------------------------------------------
// Network tests — Screener Results (dataframe.md "Screener Results" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_screener_to_dataframe() {
    use finance_query::{Screener, finance};

    // From dataframe.md "Screener Results" section
    let gainers = finance::screener(Screener::DayGainers, 50).await.unwrap();

    // Convert to DataFrame
    let df = gainers.to_dataframe().unwrap();
    println!("{}", df);

    assert!(df.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Indicators (dataframe.md "Indicators" section)
// Requires both `dataframe` and `indicators` features.
// ---------------------------------------------------------------------------

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators_to_dataframe() {
    use finance_query::Ticker;

    // From dataframe.md "Indicators" section
    let ticker = Ticker::new("TSLA").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Convert to single-row DataFrame with all 52 indicators
    let df = indicators.to_dataframe().unwrap();

    // Access specific indicators
    println!("RSI(14): {:?}", df.column("rsi_14").unwrap());
    println!("MACD: {:?}", df.column("macd").unwrap());
}

// ---------------------------------------------------------------------------
// Network tests — Filtering (dataframe.md "Filtering Data" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_filtering() {
    use finance_query::Ticker;

    // From dataframe.md "Filtering Data" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::SixMonths)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // For filtering with current Polars API, see Polars documentation
    // Older API example (may need updates):
    // let high_volume = df.filter(&df.column("volume")?.gt(50_000_000)?)?;
    println!("Total days: {}", df.height());

    assert!(df.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Computing Statistics (dataframe.md "Computing Statistics" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_statistics() {
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Computing Statistics" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::SixMonths)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // Calculate average closing price, max high, min low via lazy API
    // (Polars 0.53 + lazy feature: Column::mean/max/min require lazy for f64 extraction)
    let stats = df
        .clone()
        .lazy()
        .select([
            col("close").mean().alias("avg_close"),
            col("high").max().alias("max_high"),
            col("low").min().alias("min_low"),
        ])
        .collect()
        .unwrap();

    let avg_close: f64 = stats
        .column("avg_close")
        .unwrap()
        .f64()
        .unwrap()
        .get(0)
        .unwrap();
    let max_high: f64 = stats
        .column("max_high")
        .unwrap()
        .f64()
        .unwrap()
        .get(0)
        .unwrap();
    let min_low: f64 = stats
        .column("min_low")
        .unwrap()
        .f64()
        .unwrap()
        .get(0)
        .unwrap();
    println!("Average close: ${:.2}", avg_close);
    println!("Range: ${:.2} - ${:.2}", min_low, max_high);

    assert!(avg_close > 0.0);
    assert!(max_high >= min_low);
}

// ---------------------------------------------------------------------------
// Network tests — Adding Calculated Columns (dataframe.md "Adding Calculated Columns")
// Doc uses outdated Series arithmetic; expressed via equivalent lazy API.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_calculated_columns() {
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Adding Calculated Columns" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // Doc shows (older API):
    //   let close = df.column("close")?;
    //   let prev_close = close.shift(1);
    //   let daily_return = ((close - &prev_close) / &prev_close) * lit(100.0);
    //   df.with_column(daily_return.alias("daily_return_pct"))?;
    //
    // Near-exact equivalent using lazy API (Polars 0.53):
    let df = df
        .lazy()
        .with_column(
            ((col("close") - col("close").shift(lit(1))) / col("close").shift(lit(1)) * lit(100.0))
                .alias("daily_return_pct"),
        )
        .collect()
        .unwrap();

    assert!(df.column("daily_return_pct").is_ok());
}

// ---------------------------------------------------------------------------
// Network tests — Time-based Operations (dataframe.md "Time-based Operations")
// Doc uses outdated Column::gt_eq(); expressed via lazy filter.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_time_based_operations() {
    use chrono::DateTime;
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Time-based Operations" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // Convert timestamp to datetime
    let dates: Vec<_> = df
        .column("timestamp")
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|ts| ts.map(|t| DateTime::from_timestamp(t, 0).unwrap()))
        .collect();
    assert!(!dates.is_empty());

    // Filter by date range (doc shows older API: df.column("timestamp")?.gt_eq(start_ts)?)
    // Near-exact equivalent using lazy filter:
    let start_ts = 1704067200i64; // 2024-01-01
    let df_filtered = df
        .lazy()
        .filter(col("timestamp").gt_eq(lit(start_ts)))
        .collect()
        .unwrap();

    println!("Filtered rows: {}", df_filtered.height());
}

// ---------------------------------------------------------------------------
// Network tests — Sorting and Ranking (dataframe.md "Sorting and Ranking" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_sorting_and_ranking() {
    use finance_query::{Screener, finance};
    use polars::prelude::*;

    // From dataframe.md "Sorting and Ranking" section
    let gainers = finance::screener(Screener::DayGainers, 100).await.unwrap();

    let mut df = gainers.to_dataframe().unwrap();

    // Sort by market cap descending
    df = df
        .sort(
            ["market_cap"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .unwrap();

    // Get top 10
    let top_10 = df.head(Some(10));
    println!("{}", top_10);

    assert!(top_10.height() <= 10);
}

// ---------------------------------------------------------------------------
// Network tests — Aggregations (dataframe.md "Aggregations" section)
// Doc's select-then-group_by pattern adapted to correct group_by-then-agg.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_aggregations() {
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Aggregations" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // Group by month and aggregate (near-exact: corrected select→group_by ordering)
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
        .collect()
        .unwrap();

    println!("{}", monthly);
    assert!(monthly.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Multiple Symbols (dataframe.md "Multiple Symbols" section)
// Doc passes DataFrame slice to concat(); adapted to lazy concat for Polars 0.53.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_multiple_symbols_concat() {
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Multiple Symbols" section
    let aapl = Ticker::new("AAPL").await.unwrap();
    let msft = Ticker::new("MSFT").await.unwrap();
    let nvda = Ticker::new("NVDA").await.unwrap();

    let aapl_chart = aapl
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let msft_chart = msft
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let nvda_chart = nvda
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Convert to DataFrames
    let mut aapl_df = aapl_chart.to_dataframe().unwrap();
    let mut msft_df = msft_chart.to_dataframe().unwrap();
    let mut nvda_df = nvda_chart.to_dataframe().unwrap();

    // Add symbol column to each (Series::new requires .into() → Column in Polars 0.53)
    aapl_df
        .with_column(Series::new("symbol".into(), vec!["AAPL"; aapl_df.height()]).into())
        .unwrap();
    msft_df
        .with_column(Series::new("symbol".into(), vec!["MSFT"; msft_df.height()]).into())
        .unwrap();
    nvda_df
        .with_column(Series::new("symbol".into(), vec!["NVDA"; nvda_df.height()]).into())
        .unwrap();

    // Combine into single DataFrame (doc shows concat(&[dfs], UnionArgs::default());
    // adapted for Polars 0.53 lazy concat)
    let combined = concat(
        [aapl_df.lazy(), msft_df.lazy(), nvda_df.lazy()],
        UnionArgs::default(),
    )
    .unwrap()
    .collect()
    .unwrap();

    println!("Combined data: {} rows", combined.height());
    assert!(combined.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Joining DataFrames (dataframe.md "Joining DataFrames" section)
// Doc notes the join API may require additional trait imports; tests DataFrame
// creation only — the commented join syntax is preserved.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_joining_dataframes() {
    use finance_query::{Dividend, Ticker, TimeRange};

    // From dataframe.md "Joining DataFrames" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let aapl_chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let aapl_divs = ticker.dividends(TimeRange::OneMonth).await.unwrap();

    let price_df = aapl_chart.to_dataframe().unwrap();
    let div_df = Dividend::vec_to_dataframe(&aapl_divs).unwrap();

    // Note: left_join API may require trait imports in newer Polars versions
    // Example: use polars_ops::frame::join::DataFrameJoinOps;
    // let joined = price_df.left_join(&div_df, ["timestamp"], ["timestamp"])?;
    assert!(price_df.column("timestamp").is_ok());
    assert!(div_df.column("timestamp").is_ok());
}

// ---------------------------------------------------------------------------
// Network tests — Custom Analysis (dataframe.md "Custom Analysis" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_polars_custom_analysis() {
    use finance_query::{Screener, Ticker, finance};
    use polars::prelude::*;

    // From dataframe.md "Custom Analysis" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();

    // Calculate daily price range as percentage
    let range_pct = df
        .clone()
        .lazy()
        .select([
            col("timestamp"),
            ((col("high") - col("low")) / col("close") * lit(100.0)).alias("range_pct"),
        ])
        .collect()
        .unwrap();

    // Find days with highest volatility
    let volatile_days = range_pct
        .sort(
            ["range_pct"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .unwrap()
        .head(Some(10));

    println!("Most volatile days:\n{}", volatile_days);
    assert!(volatile_days.height() <= 10);

    // Also cover the screener sort pattern from "Sorting and Ranking" with finance module
    let _ = finance::screener(Screener::DayGainers, 100).await.unwrap();
}

// ---------------------------------------------------------------------------
// Network tests — Vec to DataFrame (dataframe.md "Vec to DataFrame" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_vec_to_dataframe() {
    use finance_query::{Dividend, SearchOptions, Ticker, TimeRange, finance};

    // From dataframe.md "Type Conversions — Vec to DataFrame" section

    // Vec of dividends to DataFrame
    let ticker = Ticker::new("AAPL").await.unwrap();
    let dividends = ticker.dividends(TimeRange::FiveYears).await.unwrap();
    let df = Dividend::vec_to_dataframe(&dividends).unwrap();
    assert!(df.column("timestamp").is_ok());

    // SearchQuotes wrapper has to_dataframe() method
    let results = finance::search("tech", &SearchOptions::default())
        .await
        .unwrap();
    let df = results.quotes.to_dataframe().unwrap();
    assert!(df.height() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Single Item to DataFrame (dataframe.md "Single Item to DataFrame")
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_single_item_to_dataframe() {
    use finance_query::Ticker;

    // From dataframe.md "Single Item to DataFrame" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let quote = ticker.quote().await.unwrap();
    let df = quote.to_dataframe().unwrap(); // 1 row, 30+ columns

    assert_eq!(df.height(), 1);
    assert!(df.width() >= 30);
}

// ---------------------------------------------------------------------------
// Network tests — Error Handling (dataframe.md "Error Handling" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_error_handling_match() {
    use finance_query::Ticker;

    // From dataframe.md "Error Handling" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    match chart.to_dataframe() {
        Ok(df) => {
            println!("DataFrame created: {} rows", df.height());
            assert!(df.height() > 0);
        }
        Err(e) => {
            eprintln!("DataFrame conversion error: {}", e);
            panic!("Expected successful conversion");
        }
    }
}

// ---------------------------------------------------------------------------
// Network tests — Best Practices (dataframe.md "Best Practices" section)
// df.filter() older API expressed via lazy; df.tail() is stable.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_best_practices_cache_and_tail() {
    use finance_query::Ticker;
    use polars::prelude::*;

    // From dataframe.md "Best Practices — Combine with Ticker Caching" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Convert to DataFrame for analysis
    let df = chart.to_dataframe().unwrap();

    // Reuse the same chart data for different analyses
    // Doc shows: df.filter(&df.column("volume")?.gt(50_000_000)?)?  (older API)
    // Near-exact equivalent using lazy filter:
    let high_volume = df
        .clone()
        .lazy()
        .filter(col("volume").gt(lit(50_000_000i64)))
        .collect()
        .unwrap();
    let recent = df.tail(Some(5));

    println!("High volume days: {}", high_volume.height());
    println!("Recent 5 days:\n{}", recent);

    // No additional API calls - data is cached in the Ticker
    assert!(recent.height() <= 5);
}
