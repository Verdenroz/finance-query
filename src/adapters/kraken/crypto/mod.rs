//! `CRYPTO` capability for Kraken public market data.

use crate::error::Result;
use crate::models::crypto::CryptoQuote;

use super::models::KrakenTicker;
use super::symbols;

/// Index of the rolling 24-hour figure in Kraken's `[today, 24h]` arrays.
const ROLLING_24H: usize = 1;

/// Map a Kraken ticker onto the canonical [`CryptoQuote`].
///
/// `change_24h` is measured against **today's opening price** (00:00 UTC), not
/// a price from exactly 24 hours ago: Kraken publishes the former and not the
/// latter. Early in the UTC day the figure is therefore a short-window change.
pub(super) fn to_quote(id: &str, base_ticker: &str, ticker: &KrakenTicker) -> CryptoQuote {
    let last = KrakenTicker::at(&ticker.c, 0);
    let open = ticker.o.parse::<f64>().ok();
    let change = match (last, open) {
        (Some(l), Some(o)) => Some(l - o),
        _ => None,
    };
    let change_percent = match (last, open) {
        (Some(l), Some(o)) if o != 0.0 => Some((l - o) / o * 100.0),
        _ => None,
    };

    CryptoQuote {
        id: id.to_string(),
        symbol: base_ticker.to_string(),
        name: symbols::asset_name(base_ticker)
            .map(str::to_string)
            .unwrap_or_else(|| base_ticker.to_string()),
        price: last,
        // An exchange knows what trades on it, not how much of an asset exists.
        market_cap: None,
        volume_24h: KrakenTicker::at(&ticker.v, ROLLING_24H),
        change_24h: change,
        change_percent_24h: change_percent,
        high_24h: KrakenTicker::at(&ticker.h, ROLLING_24H),
        low_24h: KrakenTicker::at(&ticker.l, ROLLING_24H),
        circulating_supply: None,
    }
}

/// Fetch a Kraken spot quote for `id` priced in `vs_currency`.
pub(crate) async fn fetch_crypto_quote_response(
    id: &str,
    vs_currency: &str,
) -> Result<CryptoQuote> {
    let pair = symbols::pair(id, vs_currency);
    let ticker = super::client()?.ticker(&pair).await?;
    Ok(to_quote(id, &symbols::ticker_for(id), &ticker))
}
