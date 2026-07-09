//! Compile and runtime tests for docs/library/providers/coingecko.md
//!
//! Requires the `crypto` feature flag:
//!   cargo test --test doc_coingecko --features crypto
//!   cargo test --test doc_coingecko --features crypto -- --ignored   (network tests)

#![cfg(feature = "crypto")]

use finance_query::crypto::CoinQuote;

// ---------------------------------------------------------------------------
// CoinQuote — compile-time field verification (mirrors the Fields table)
// ---------------------------------------------------------------------------

/// Verifies all CoinQuote fields documented in providers/coingecko.md exist
/// with correct types. Never called; exists only for the compiler to
/// type-check.
#[allow(dead_code)]
fn _verify_coin_quote_fields(c: CoinQuote) {
    let _: String = c.id;
    let _: String = c.symbol;
    let _: String = c.name;
    let _: Option<f64> = c.current_price;
    let _: Option<f64> = c.market_cap;
    let _: Option<u32> = c.market_cap_rank;
    let _: Option<f64> = c.price_change_percentage_24h;
    let _: Option<f64> = c.total_volume;
    let _: Option<f64> = c.circulating_supply;
    let _: Option<String> = c.image;
}

// ---------------------------------------------------------------------------
// Network tests — Top Coins by Market Cap / Single Coin Lookup
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_top_coins_by_market_cap() {
    use finance_query::crypto;

    let top = crypto::coins("usd", 10).await.unwrap();

    for coin in &top {
        let price = coin.current_price.unwrap_or(0.0);
        let change = coin.price_change_percentage_24h.unwrap_or(0.0);
        let rank = coin.market_cap_rank.unwrap_or(0);
        println!(
            "#{} {} ({}): ${:.2} ({:+.2}%)",
            rank, coin.name, coin.symbol, price, change
        );
    }

    assert!(!top.is_empty());
    assert!(top.len() <= 10);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_single_coin_lookup() {
    use finance_query::crypto;

    let btc = crypto::coin("bitcoin", "usd").await.unwrap();
    println!("Bitcoin: ${:.2}", btc.current_price.unwrap_or(0.0));
    assert_eq!(btc.id, "bitcoin");

    let eth = crypto::coin("ethereum", "usd").await.unwrap();
    let mktcap = eth.market_cap.unwrap_or(0.0);
    println!("Ethereum market cap: ${:.2}B", mktcap / 1e9);
    assert_eq!(eth.id, "ethereum");
}
