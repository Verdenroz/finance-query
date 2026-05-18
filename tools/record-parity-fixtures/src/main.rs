//! Record canonical JSON fixtures for cross-language parity tests.
//!
//! Run: cargo run -p record-parity-fixtures -- AAPL MSFT NVDA TSLA SPY BTC-USD ETH-USD EURUSD=X GBPUSD=X GC=F
//!
//! Output: finance-query-python/tests/parity/fixtures/quote_<symbol>.json

use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let symbols: Vec<String> = std::env::args().skip(1).collect();
    if symbols.is_empty() {
        eprintln!("Usage: record-parity SYM1 [SYM2 ...]");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from("finance-query-python/tests/parity/fixtures");
    fs::create_dir_all(&out_dir)?;

    for symbol in &symbols {
        eprintln!("recording {}", symbol);
        let ticker = finance_query::Ticker::new(symbol.clone()).await?;
        let quote = ticker.quote().await?;

        let fname = format!(
            "quote_{}.json",
            symbol.replace('=', "_").replace('-', "_").to_lowercase()
        );
        let path = out_dir.join(fname);
        fs::write(&path, serde_json::to_string_pretty(&quote)?)?;
        eprintln!("  wrote {}", path.display());
    }
    Ok(())
}
