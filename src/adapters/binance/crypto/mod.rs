//! `CRYPTO` capability for Binance public market data.

use crate::error::Result;
use crate::models::crypto::CryptoQuote;

use super::models::Ticker24hr;
use super::symbols;

/// Map a 24-hour ticker onto the canonical [`CryptoQuote`].
///
/// `market_cap` and `circulating_supply` stay `None`: an exchange knows what
/// trades on it, not how much of an asset exists. Route `CRYPTO` to CoinGecko
/// if you need supply-side figures.
pub(super) fn to_quote(id: &str, ticker: Ticker24hr) -> CryptoQuote {
    let base = symbols::split_pair(&ticker.symbol)
        .map(|(base, _)| base.to_string())
        .unwrap_or_else(|| ticker.symbol.clone());
    let name = symbols::asset_name(&base)
        .map(str::to_string)
        .unwrap_or_else(|| base.clone());

    let num = |s: &str| s.parse::<f64>().ok();

    CryptoQuote {
        id: id.to_string(),
        symbol: base,
        name,
        price: num(&ticker.last_price),
        market_cap: None,
        volume_24h: num(&ticker.quote_volume),
        change_24h: num(&ticker.price_change),
        change_percent_24h: num(&ticker.price_change_percent),
        high_24h: num(&ticker.high_price),
        low_24h: num(&ticker.low_price),
        circulating_supply: None,
    }
}

/// Fetch a Binance spot quote for `id` priced in `vs_currency`.
pub(crate) async fn fetch_crypto_quote_response(
    id: &str,
    vs_currency: &str,
) -> Result<CryptoQuote> {
    let market = symbols::pair(id, vs_currency);
    let ticker = super::client()?.ticker_24hr(&market).await?;
    Ok(to_quote(id, ticker))
}
