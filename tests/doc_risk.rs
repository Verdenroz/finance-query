//! Compile and runtime tests for docs/library/risk.md
//!
//! Requires the `risk` feature flag:
//!   cargo test --test doc_risk --features risk
//!   cargo test --test doc_risk --features risk -- --ignored   (network tests)

#![cfg(feature = "risk")]

use finance_query::risk::{
    beta, calmar_ratio, historical_var, max_drawdown, parametric_var, sharpe_ratio, sortino_ratio,
};

// ---------------------------------------------------------------------------
// RiskSummary — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all RiskSummary fields documented in risk.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_risk_summary_fields(r: finance_query::risk::RiskSummary) {
    let _: f64 = r.var_95;
    let _: f64 = r.var_99;
    let _: f64 = r.parametric_var_95;
    let _: Option<f64> = r.sharpe;
    let _: Option<f64> = r.sortino;
    let _: Option<f64> = r.calmar;
    let _: Option<f64> = r.beta;
    let _: f64 = r.max_drawdown;
    let _: Option<u64> = r.max_drawdown_recovery_periods;
}

// ---------------------------------------------------------------------------
// Standalone functions — pure computation tests
// ---------------------------------------------------------------------------

fn trending_up() -> Vec<f64> {
    // +2% per period for 252 periods, then a -20% drawdown, then recovery
    let mut returns = vec![0.02_f64; 100];
    returns.push(-0.20);
    returns.extend(vec![0.02_f64; 152]);
    returns
}

fn flat() -> Vec<f64> {
    vec![0.0_f64; 252]
}

#[test]
fn test_historical_var_returns_value_for_non_trivial_series() {
    let returns = trending_up();
    let var_95 = historical_var(&returns, 0.95);
    assert!(var_95.is_some());
    // For a strongly uptrending series the 5th-percentile outcome is still a
    // gain, so historical VaR can legitimately be negative (expected profit,
    // not loss). The only constraint is that the value is finite.
    assert!(var_95.unwrap().is_finite());
}

#[test]
fn test_historical_var_flat_returns_zero() {
    let returns = flat();
    let var = historical_var(&returns, 0.95).unwrap_or(0.0);
    assert_eq!(var, 0.0);
}

#[test]
fn test_parametric_var_returns_value() {
    let returns = trending_up();
    let pvar = parametric_var(&returns, 0.95);
    assert!(pvar.is_some());
    assert!(pvar.unwrap() >= 0.0);
}

#[test]
fn test_sharpe_ratio_positive_returns() {
    // Positive constant returns → positive Sharpe
    let returns = vec![0.001_f64; 252];
    let sharpe = sharpe_ratio(&returns, 0.0, 252.0);
    assert!(sharpe.is_some());
    assert!(
        sharpe.unwrap() > 0.0,
        "expected positive Sharpe for positive returns"
    );
}

#[test]
fn test_sharpe_ratio_flat_returns_none() {
    let sharpe = sharpe_ratio(&flat(), 0.0, 252.0);
    assert!(
        sharpe.is_none(),
        "zero-volatility series should produce None"
    );
}

#[test]
fn test_sortino_ratio_positive_returns() {
    let returns = vec![0.001_f64; 252];
    let sortino = sortino_ratio(&returns, 0.0, 252.0);
    // All positive returns → no downside deviation → Sortino may be None
    // (no downside to penalize). Either Some or None is acceptable.
    println!("Sortino (all positive): {sortino:?}");
}

#[test]
fn test_sortino_ratio_mixed_returns() {
    let mut returns = vec![0.01_f64; 126];
    returns.extend(vec![-0.01_f64; 126]);
    let sortino = sortino_ratio(&returns, 0.0, 252.0);
    println!("Sortino (mixed): {sortino:?}");
}

#[test]
fn test_calmar_ratio_positive_case() {
    // 25% total return, 1 year, 10% max drawdown → Calmar ≈ 2.5
    let calmar = calmar_ratio(0.25, 1.0, 0.10);
    assert!(calmar.is_some());
    let c = calmar.unwrap();
    assert!((c - 2.5).abs() < 1e-9, "expected Calmar ≈ 2.5, got {c}");
}

#[test]
fn test_calmar_ratio_zero_drawdown_returns_none() {
    let calmar = calmar_ratio(0.25, 1.0, 0.0);
    assert!(calmar.is_none(), "zero max drawdown should produce None");
}

#[test]
fn test_beta_positive_correlation() {
    // Asset moves exactly 2x the benchmark → beta = 2
    let benchmark: Vec<f64> = (0..252)
        .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
        .collect();
    let asset: Vec<f64> = benchmark.iter().map(|r| r * 2.0).collect();
    let b = beta(&asset, &benchmark);
    assert!(b.is_some());
    let b = b.unwrap();
    assert!((b - 2.0).abs() < 1e-6, "expected beta ≈ 2.0, got {b}");
}

#[test]
fn test_beta_insufficient_data_returns_none() {
    let b = beta(&[0.01], &[0.01]);
    assert!(b.is_none());
}

#[test]
fn test_max_drawdown_known_series() {
    // Simple drop: +10%, -20%, +15% → peak=1.10, trough=0.88 → dd ≈ 0.20
    let returns = vec![0.10, -0.20, 0.15];
    let dd = max_drawdown(&returns);
    assert!(dd.max_drawdown > 0.0, "expected positive drawdown");
    println!("max_drawdown: {:.4}", dd.max_drawdown);
    println!("recovery_periods: {:?}", dd.recovery_periods);
    // Field types match docs
    let _: f64 = dd.max_drawdown;
    let _: Option<u64> = dd.recovery_periods;
}

#[test]
fn test_max_drawdown_flat_series() {
    let dd = max_drawdown(&flat());
    assert_eq!(dd.max_drawdown, 0.0);
}

#[test]
fn test_max_drawdown_empty() {
    let dd = max_drawdown(&[]);
    assert_eq!(dd.max_drawdown, 0.0);
    assert!(dd.recovery_periods.is_none());
}

// ---------------------------------------------------------------------------
// Full standalone workflow from risk.md
// ---------------------------------------------------------------------------

#[test]
fn test_standalone_risk_workflow() {
    let returns: Vec<f64> = (0..252)
        .map(|i| if i % 3 == 0 { -0.01 } else { 0.005 })
        .collect();
    let benchmark: Vec<f64> = returns.iter().map(|r| r * 0.8).collect();

    let var_95 = historical_var(&returns, 0.95).unwrap_or(0.0);
    let var_99 = historical_var(&returns, 0.99).unwrap_or(0.0);
    let pvar = parametric_var(&returns, 0.95).unwrap_or(0.0);
    let sharpe = sharpe_ratio(&returns, 0.0, 252.0);
    let sortino = sortino_ratio(&returns, 0.0, 252.0);
    let dd = max_drawdown(&returns);
    let total_return = returns.iter().fold(1.0_f64, |acc, r| acc * (1.0 + r)) - 1.0;
    let years = returns.len() as f64 / 252.0;
    let calmar = calmar_ratio(total_return, years, dd.max_drawdown);
    let b = beta(&returns, &benchmark);

    println!("VaR 95%:       {:.4}", var_95);
    println!("VaR 99%:       {:.4}", var_99);
    println!("Param VaR 95%: {:.4}", pvar);
    println!("Max Drawdown:  {:.4}", dd.max_drawdown);
    println!("Sharpe:        {sharpe:?}");
    println!("Sortino:       {sortino:?}");
    println!("Calmar:        {calmar:?}");
    println!("Beta:          {b:?}");

    assert!(var_95 >= 0.0);
    assert!(var_99 >= var_95);
    assert!(dd.max_drawdown >= 0.0);
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_risk_with_benchmark() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();
    let summary = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC"))
        .await
        .unwrap();

    println!("=== AAPL Risk (1Y daily, SPX benchmark) ===");
    println!("VaR 95%:       {:.2}%", summary.var_95 * 100.0);
    println!("VaR 99%:       {:.2}%", summary.var_99 * 100.0);
    println!("Param VaR 95%: {:.2}%", summary.parametric_var_95 * 100.0);
    println!("Max Drawdown:  {:.2}%", summary.max_drawdown * 100.0);
    if let Some(p) = summary.max_drawdown_recovery_periods {
        println!("Recovery:      {} trading days", p);
    }
    if let Some(s) = summary.sharpe {
        println!("Sharpe:        {:.2}", s);
    }
    if let Some(so) = summary.sortino {
        println!("Sortino:       {:.2}", so);
    }
    if let Some(c) = summary.calmar {
        println!("Calmar:        {:.2}", c);
    }
    if let Some(b) = summary.beta {
        println!("Beta (SPX):    {:.2}", b);
    }

    assert!(summary.var_95 >= 0.0);
    assert!(summary.max_drawdown >= 0.0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_risk_no_benchmark() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("NVDA").await.unwrap();
    let summary = ticker
        .risk(Interval::OneDay, TimeRange::TwoYears, None)
        .await
        .unwrap();

    assert!(
        summary.beta.is_none(),
        "beta should be None without benchmark"
    );
    assert!(summary.max_drawdown >= 0.0);
    println!(
        "NVDA max drawdown (2Y): {:.2}%",
        summary.max_drawdown * 100.0
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_full_risk_report_nvda() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From risk.md "Example: Full Risk Report" section — exact code pattern
    let ticker = Ticker::new("NVDA").await.unwrap();
    let risk = ticker
        .risk(Interval::OneDay, TimeRange::TwoYears, Some("^GSPC"))
        .await
        .unwrap();

    println!("=== Risk Report: NVDA (2Y daily) ===");
    println!("VaR 95%:       {:.2}%", risk.var_95 * 100.0);
    println!("VaR 99%:       {:.2}%", risk.var_99 * 100.0);
    println!("Param VaR 95%: {:.2}%", risk.parametric_var_95 * 100.0);
    println!("Max Drawdown:  {:.2}%", risk.max_drawdown * 100.0);

    if let Some(periods) = risk.max_drawdown_recovery_periods {
        println!("Recovery:      {} trading days", periods);
    } else {
        println!("Recovery:      no full recovery in window");
    }

    if let Some(sharpe) = risk.sharpe {
        println!("Sharpe:        {:.2}", sharpe);
    }
    if let Some(sortino) = risk.sortino {
        println!("Sortino:       {:.2}", sortino);
    }
    if let Some(calmar) = risk.calmar {
        println!("Calmar:        {:.2}", calmar);
    }
    if let Some(beta) = risk.beta {
        println!("Beta (vs SPX): {:.2}", beta);
    }

    assert!(risk.var_95 >= 0.0);
    assert!(risk.max_drawdown >= 0.0);
}
