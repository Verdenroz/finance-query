//! Compile and runtime tests for docs/library/crypto.md
//!
//! Requires the `crypto` feature flag:
//!   cargo test --test doc_crypto --features crypto
//!   cargo test --test doc_crypto --features crypto -- --ignored   (network tests)

#![cfg(feature = "crypto")]

use finance_query::crypto::CoinQuote;

// ---------------------------------------------------------------------------
// CoinQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all CoinQuote fields documented in crypto.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
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
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_crypto_top_coins() {
    use finance_query::crypto;

    let top = crypto::coins("usd", 10).await.unwrap();
    assert!(!top.is_empty(), "should return coins");
    assert!(top.len() <= 10);

    for coin in &top {
        let price = coin.current_price.unwrap_or(0.0);
        let change = coin.price_change_percentage_24h.unwrap_or(0.0);
        let rank = coin.market_cap_rank.unwrap_or(0);
        println!(
            "#{} {} ({}): ${:.2} ({:+.2}%)",
            rank, coin.name, coin.symbol, price, change
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_crypto_single_coin_bitcoin() {
    use finance_query::crypto;

    let btc = crypto::coin("bitcoin", "usd").await.unwrap();
    assert_eq!(btc.id, "bitcoin");
    assert_eq!(btc.symbol.to_uppercase(), "BTC");
    let price = btc.current_price.unwrap_or(0.0);
    assert!(price > 0.0, "BTC price should be positive");
    println!("Bitcoin: ${:.2}", price);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_crypto_single_coin_ethereum() {
    use finance_query::crypto;

    let eth = crypto::coin("ethereum", "usd").await.unwrap();
    let mktcap = eth.market_cap.unwrap_or(0.0);
    println!("Ethereum market cap: ${:.2}B", mktcap / 1e9);
    assert!(mktcap > 0.0);
}
