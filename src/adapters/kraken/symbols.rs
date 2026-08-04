//! Symbol normalisation for Kraken public markets.
//!
//! Kraken predates most ticker conventions and kept its own: Bitcoin is
//! `XBT`, Dogecoin is `XDG`, and older pairs carry `X`/`Z` class prefixes
//! (`XXBTZUSD`). Callers should never have to know that, so every spelling is
//! translated here. Only the Kraken-specific aliasing lives in this file — the
//! coin id table is shared with Binance in [`crate::adapters::common::coins`].

use crate::adapters::common::coins;

pub(crate) use coins::{asset_name, resolve_ticker};

/// Assets whose Kraken code differs from the ticker everyone else uses.
const ASSET_ALIASES: &[(&str, &str)] = &[
    ("BTC", "XBT"),
    ("DOGE", "XDG"),
    // Kraken lists Terra Classic under its original code.
    ("LUNC", "LUNA"),
];

/// Quote assets Kraken lists, longest first so a concatenated market splits
/// on the right boundary.
const QUOTE_ASSETS: &[&str] = &[
    "USDT", "USDC", "CHF", "AUD", "USD", "EUR", "GBP", "JPY", "CAD", "XBT", "ETH", "DAI",
];

/// Translate a common ticker to Kraken's own asset code.
pub(crate) fn kraken_asset(ticker: &str) -> String {
    let upper = ticker.trim().to_uppercase();
    ASSET_ALIASES
        .iter()
        .find(|(common, _)| *common == upper)
        .map(|(_, kraken)| (*kraken).to_string())
        .unwrap_or(upper)
}

/// Strip Kraken's legacy `X`/`Z` asset-class prefixes and undo its aliases,
/// recovering the ticker callers expect.
///
/// `XXBT` → `BTC`, `ZUSD` → `USD`, `SOL` → `SOL`. Only 4-character codes carry
/// the prefix, so 3-character modern codes pass through untouched.
pub(crate) fn from_kraken_asset(code: &str) -> String {
    let trimmed = match code.len() {
        4 if code.starts_with(['X', 'Z']) => &code[1..],
        _ => code,
    };
    ASSET_ALIASES
        .iter()
        .find(|(_, kraken)| *kraken == trimmed)
        .map(|(common, _)| (*common).to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

/// Build the pair string to send to Kraken from a coin id and quote currency.
pub(crate) fn pair(base_id: &str, vs_currency: &str) -> String {
    format!(
        "{}{}",
        kraken_asset(&resolve_ticker(base_id)),
        kraken_asset(vs_currency)
    )
}

/// Split a concatenated market into `(base, quote)` on a known quote asset.
pub(crate) fn split_pair(symbol: &str) -> Option<(&str, &str)> {
    coins::split_on_quote(symbol, QUOTE_ASSETS)
}

/// Normalise any of the library's symbol spellings into a Kraken pair.
///
/// Accepts a separated pair (`BTC-USD`, `SOL/USD`) or a concatenated market
/// (`XBTUSD`, `SOLUSD`). A bare asset with no quote returns `None`.
pub(crate) fn parse_market(symbol: &str) -> Option<String> {
    let symbol = symbol.trim();
    if let Some((base, quote)) = symbol
        .split_once(['-', '/', '_'])
        .filter(|(b, q)| !b.is_empty() && !q.is_empty())
    {
        return Some(pair(base, quote));
    }
    let upper = symbol.to_uppercase();
    let (base, quote) = split_pair(&upper)?;
    Some(format!("{}{}", kraken_asset(base), kraken_asset(quote)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kraken_keeps_its_own_codes_for_btc_and_doge() {
        assert_eq!(kraken_asset("BTC"), "XBT");
        assert_eq!(kraken_asset("doge"), "XDG");
        assert_eq!(kraken_asset("ETH"), "ETH");
        assert_eq!(kraken_asset("usd"), "USD");
    }

    #[test]
    fn legacy_class_prefixes_are_stripped_on_the_way_back() {
        assert_eq!(from_kraken_asset("XXBT"), "BTC");
        assert_eq!(from_kraken_asset("ZUSD"), "USD");
        assert_eq!(from_kraken_asset("XETH"), "ETH");
        // Modern three-character codes have no prefix to strip.
        assert_eq!(from_kraken_asset("SOL"), "SOL");
        assert_eq!(from_kraken_asset("ADA"), "ADA");
        // A four-character code that isn't prefixed must survive intact.
        assert_eq!(from_kraken_asset("AAVE"), "AAVE");
    }

    #[test]
    fn pairs_use_kraken_codes_on_both_sides() {
        assert_eq!(pair("bitcoin", "usd"), "XBTUSD");
        assert_eq!(pair("dogecoin", "eur"), "XDGEUR");
        assert_eq!(pair("SOL", "USD"), "SOLUSD");
    }

    #[test]
    fn separated_and_concatenated_spellings_both_parse() {
        assert_eq!(parse_market("BTC-USD").as_deref(), Some("XBTUSD"));
        assert_eq!(parse_market("bitcoin/eur").as_deref(), Some("XBTEUR"));
        assert_eq!(parse_market("SOLUSD").as_deref(), Some("SOLUSD"));
        // An already-Kraken-coded market survives the round trip.
        assert_eq!(parse_market("XBTUSD").as_deref(), Some("XBTUSD"));
        assert_eq!(parse_market("BTC"), None);
    }

    #[test]
    fn quote_assets_split_longest_first() {
        assert_eq!(split_pair("XBTUSDT"), Some(("XBT", "USDT")));
        assert_eq!(split_pair("SOLUSD"), Some(("SOL", "USD")));
        assert_eq!(split_pair("USD"), None);
    }
}
