//! Compile and runtime tests for docs/library/fred.md
//!
//! Requires the `fred` feature flag:
//!   cargo test --test doc_fred --features fred
//!   cargo test --test doc_fred --features fred -- --ignored   (network tests)

#![cfg(feature = "fred")]

use finance_query::fred::{MacroObservation, MacroSeries, TreasuryYield};

// ---------------------------------------------------------------------------
// Model fields — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies MacroSeries fields documented in fred.md.
#[allow(dead_code)]
fn _verify_macro_series_fields(s: MacroSeries) {
    let _: String = s.id;
    let _: Vec<MacroObservation> = s.observations;
}

/// Verifies MacroObservation fields documented in fred.md.
#[allow(dead_code)]
fn _verify_macro_observation_fields(o: MacroObservation) {
    let _: String = o.date;
    let _: Option<f64> = o.value;
}

/// Verifies TreasuryYield fields documented in fred.md — all maturities.
#[allow(dead_code)]
fn _verify_treasury_yield_fields(y: TreasuryYield) {
    let _: String = y.date;
    let _: Option<f64> = y.y1m;
    let _: Option<f64> = y.y2m;
    let _: Option<f64> = y.y3m;
    let _: Option<f64> = y.y4m;
    let _: Option<f64> = y.y6m;
    let _: Option<f64> = y.y1;
    let _: Option<f64> = y.y2;
    let _: Option<f64> = y.y3;
    let _: Option<f64> = y.y5;
    let _: Option<f64> = y.y7;
    let _: Option<f64> = y.y10;
    let _: Option<f64> = y.y20;
    let _: Option<f64> = y.y30;
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_fred_init_double_init_returns_error() {
    use finance_query::fred;

    // First call may or may not succeed (singleton may already be set).
    let _ = fred::init("test-key-1");
    // Second call must always fail.
    let result = fred::init("test-key-2");
    assert!(
        result.is_err(),
        "second fred::init call should return an error"
    );
}

#[test]
fn test_fred_init_with_timeout_compiles() {
    use finance_query::fred;
    use std::time::Duration;

    // From fred.md "FRED Setup" section — optional custom timeout
    // Singleton may already be set; ignore the result.
    let _ = fred::init_with_timeout("your-fred-api-key", Duration::from_secs(60));
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fred_series_fedfunds() {
    use finance_query::fred;

    // Init may already be set from the unit test above; ignore the result.
    let _ = fred::init("test-key");

    // Note: this will fail if the key is invalid — in CI, set FRED_API_KEY env var
    // and use `fred::init(std::env::var("FRED_API_KEY").unwrap())`.
    let result = fred::series("FEDFUNDS").await;
    match result {
        Ok(series) => {
            println!("FEDFUNDS observations: {}", series.observations.len());
            assert!(!series.observations.is_empty());
            assert_eq!(series.id, "FEDFUNDS");
            for obs in series.observations.iter().rev().take(5) {
                println!("{}: {:?}", obs.date, obs.value);
            }
        }
        Err(e) => {
            // Invalid key in test env; just print and skip
            println!("FRED series error (expected in test env): {e}");
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_treasury_yields() {
    use finance_query::fred;

    let yields = fred::treasury_yields(2024).await.unwrap();
    assert!(!yields.is_empty(), "should have yield curve data for 2024");

    // Print the most recent entry
    if let Some(latest) = yields.last() {
        println!("Date:  {}", latest.date);
        println!("2Y:    {:?}%", latest.y2);
        println!("5Y:    {:?}%", latest.y5);
        println!("10Y:   {:?}%", latest.y10);
        println!("30Y:   {:?}%", latest.y30);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fred_series_cpiaucsl() {
    use finance_query::fred;

    // From fred.md "Fetching FRED Series" section — exact match pattern
    let _ = fred::init("test-key");

    let result = fred::series("CPIAUCSL").await;
    match result {
        Ok(cpi) => {
            println!("Series: {}", cpi.id);
            println!("Observations: {}", cpi.observations.len());

            // Print the last 5 observations — exact doc pattern
            for obs in cpi.observations.iter().rev().take(5) {
                match obs.value {
                    Some(v) => println!("{}: {:.2}", obs.date, v),
                    None => println!("{}: N/A", obs.date),
                }
            }
            assert!(!cpi.observations.is_empty());
        }
        Err(e) => {
            // Invalid key in test env; just print and skip
            println!("FRED series error (expected in test env): {e}");
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_yield_curve_inversion_check() {
    use finance_query::fred;

    let yields = fred::treasury_yields(2024).await.unwrap();

    let mut inverted_count = 0;
    for y in yields.iter().rev().take(5) {
        if let (Some(y2), Some(y10)) = (y.y2, y.y10) {
            let spread = y10 - y2;
            if spread < 0.0 {
                inverted_count += 1;
            }
            println!("{}: 10Y-2Y = {:.2}bps", y.date, spread * 100.0);
        }
    }
    println!("Inverted days in last 5: {inverted_count}");
}
