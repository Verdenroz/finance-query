//! Compile and runtime tests for docs/library/configuration.md
//!
//! Pure tests verify enum variants, struct field access, and builder patterns.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_configuration`
//! Run network tests: `cargo test --test doc_configuration -- --ignored`

use finance_query::{Interval, Region, TimeRange, ValueFormat};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Region enum — all 28 variants documented in configuration.md
// ---------------------------------------------------------------------------

#[test]
fn test_region_variants_compile() {
    let _ = Region::Argentina;
    let _ = Region::Australia;
    let _ = Region::Brazil;
    let _ = Region::Canada;
    let _ = Region::China;
    let _ = Region::Denmark;
    let _ = Region::Finland;
    let _ = Region::France;
    let _ = Region::Germany;
    let _ = Region::Greece;
    let _ = Region::HongKong;
    let _ = Region::India;
    let _ = Region::Israel;
    let _ = Region::Italy;
    let _ = Region::Malaysia;
    let _ = Region::NewZealand;
    let _ = Region::Norway;
    let _ = Region::Portugal;
    let _ = Region::Russia;
    let _ = Region::Singapore;
    let _ = Region::Spain;
    let _ = Region::Sweden;
    let _ = Region::Taiwan;
    let _ = Region::Thailand;
    let _ = Region::Turkey;
    let _ = Region::UnitedKingdom;
    let _ = Region::UnitedStates;
    let _ = Region::Vietnam;
}

#[test]
fn test_region_lang_and_region_methods() {
    // Verify documented lang/region pairs from configuration.md
    assert_eq!(Region::France.lang(), "fr-FR");
    assert_eq!(Region::France.region(), "FR");
    assert_eq!(Region::Germany.lang(), "de-DE");
    assert_eq!(Region::Germany.region(), "DE");
    assert_eq!(Region::Taiwan.lang(), "zh-TW");
    assert_eq!(Region::Taiwan.region(), "TW");
    assert_eq!(Region::UnitedStates.lang(), "en-US");
    assert_eq!(Region::UnitedStates.region(), "US");
    assert_eq!(Region::UnitedKingdom.lang(), "en-GB");
    assert_eq!(Region::UnitedKingdom.region(), "GB");
}

// ---------------------------------------------------------------------------
// Interval enum — all variants documented in configuration.md
// ---------------------------------------------------------------------------

#[test]
fn test_interval_variants_compile() {
    // Intraday intervals
    let _ = Interval::OneMinute;
    let _ = Interval::FiveMinutes;
    let _ = Interval::FifteenMinutes;
    let _ = Interval::ThirtyMinutes;
    let _ = Interval::OneHour;
    // Daily and longer
    let _ = Interval::OneDay;
    let _ = Interval::OneWeek;
    let _ = Interval::OneMonth;
    let _ = Interval::ThreeMonths;
}

// ---------------------------------------------------------------------------
// TimeRange enum — all variants documented in configuration.md
// ---------------------------------------------------------------------------

#[test]
fn test_time_range_variants_compile() {
    // Short term
    let _ = TimeRange::OneDay;
    let _ = TimeRange::FiveDays;
    let _ = TimeRange::OneMonth;
    let _ = TimeRange::ThreeMonths;
    let _ = TimeRange::SixMonths;
    // Long term
    let _ = TimeRange::OneYear;
    let _ = TimeRange::TwoYears;
    let _ = TimeRange::FiveYears;
    let _ = TimeRange::TenYears;
    let _ = TimeRange::YearToDate;
    let _ = TimeRange::Max;
}

// ---------------------------------------------------------------------------
// ValueFormat enum — all variants documented in configuration.md
// ---------------------------------------------------------------------------

#[test]
fn test_value_format_variants_compile() {
    let _ = ValueFormat::Raw;
    let _ = ValueFormat::Pretty;
    let _ = ValueFormat::Both;
}

// ---------------------------------------------------------------------------
// Compile-time — proxy builder methods (can't exercise real proxies in tests)
// ---------------------------------------------------------------------------

#[test]
fn test_ticker_builder_proxy_compiles() {
    use finance_query::Ticker;

    // From configuration.md "Proxy" section
    let _builder = Ticker::builder("AAPL").proxy("http://proxy.example.com:8080");

    // With authentication
    let _builder = Ticker::builder("AAPL").proxy("http://user:pass@proxy.example.com:8080");
}

#[test]
fn test_ticker_builder_proxy_and_timeout_compiles() {
    use finance_query::Ticker;

    // From configuration.md "Best Practices — Configure Timeouts and Proxies" section
    let _builder = Ticker::builder("AAPL")
        .proxy("http://corporate-proxy.company.com:8080")
        .timeout(Duration::from_secs(45));
}

// ---------------------------------------------------------------------------
// Network tests — builder patterns that require Yahoo auth
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_region_france() {
    use finance_query::Ticker;

    // From configuration.md "Using Regions" section
    let ticker = Ticker::builder("MC.PA")
        .region(Region::France)
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "MC.PA");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_region_germany() {
    use finance_query::Ticker;

    let ticker = Ticker::builder("SAP.DE")
        .region(Region::Germany)
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "SAP.DE");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_manual_lang_region() {
    use finance_query::Ticker;

    // From configuration.md "Manual Language and Region" section
    let ticker = Ticker::builder("AAPL")
        .lang("en-US")
        .region_code("US")
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_timeout() {
    use finance_query::Ticker;

    // From configuration.md "Timeout" section
    let ticker = Ticker::builder("AAPL")
        .timeout(Duration::from_secs(60))
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert!(!quote.symbol.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_builder_region() {
    use finance_query::Tickers;

    // From configuration.md "Batch Operations" section
    let tickers = Tickers::builder(vec!["2330.TW", "2317.TW", "2454.TW"])
        .region(Region::Taiwan)
        .timeout(Duration::from_secs(60))
        .build()
        .await
        .unwrap();

    let response = tickers.quotes().await.unwrap();
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_valid_interval_range_combination() {
    use finance_query::Ticker;

    // From configuration.md "Interval and Range Compatibility" example
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Valid combinations
    let daily = ticker
        .chart(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    assert!(!daily.candles.is_empty());

    let intraday = ticker
        .chart(Interval::FiveMinutes, TimeRange::OneDay)
        .await
        .unwrap();
    assert!(!intraday.candles.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_region_uk() {
    use finance_query::Ticker;

    // From configuration.md "Using Regions" section — UK stock
    let ticker = Ticker::builder("HSBA.L")
        .region(Region::UnitedKingdom)
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "HSBA.L");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_best_practices_logo_and_join() {
    use finance_query::Ticker;

    // From configuration.md "Best Practices — Match Symbols to Regions" section
    let apple = Ticker::builder("AAPL")
        .region(Region::UnitedStates)
        .logo()
        .build()
        .await
        .unwrap();

    let tsmc = Ticker::builder("2330.TW")
        .region(Region::Taiwan)
        .logo()
        .build()
        .await
        .unwrap();

    let sap = Ticker::builder("SAP.DE")
        .region(Region::Germany)
        .logo()
        .build()
        .await
        .unwrap();

    // Fetch quotes in parallel
    let (apple_quote, tsmc_quote, sap_quote) =
        tokio::join!(apple.quote(), tsmc.quote(), sap.quote());

    assert_eq!(apple_quote.unwrap().symbol, "AAPL");
    assert_eq!(tsmc_quote.unwrap().symbol, "2330.TW");
    assert_eq!(sap_quote.unwrap().symbol, "SAP.DE");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_financial_statement_frequencies() {
    use finance_query::{Frequency, StatementType, Ticker};

    // From configuration.md "Financial Statement Frequencies" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Annual statements (default)
    let income_annual = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await
        .unwrap();
    assert!(!income_annual.statement.is_empty());

    // Quarterly statements
    let income_quarterly = ticker
        .financials(StatementType::Income, Frequency::Quarterly)
        .await
        .unwrap();
    assert!(!income_quarterly.statement.is_empty());
}
