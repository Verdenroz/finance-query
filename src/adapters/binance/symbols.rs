//! Symbol normalisation for Binance spot markets.
//!
//! Binance names a market as one concatenated string (`BTCUSDT`) with no
//! separator, while the rest of the library speaks in coin ids (`bitcoin`),
//! tickers (`BTC`), and separated pairs (`BTC-USD`). Everything is funnelled
//! through here so the two conventions meet in exactly one place; the coin id
//! table itself is shared with Kraken in [`crate::adapters::common::coins`].

use crate::adapters::common::coins;

pub(crate) use coins::asset_name;

/// Quote assets Binance actually lists, longest first.
///
/// Order matters: `BTCUSDT` must split as `BTC`/`USDT`, not `BTCUSD`/`T`.
const QUOTE_ASSETS: &[&str] = &[
    "FDUSD", "USDT", "USDC", "TUSD", "BUSD", "TRY", "BRL", "EUR", "GBP", "JPY", "ARS", "BNB",
    "BTC", "ETH", "DAI",
];

/// Resolve a quote currency to the asset Binance actually lists it as.
///
/// Binance spot has **no USD markets** — dollar pairs are quoted in the USDT
/// stablecoin. `"USD"` is therefore mapped to `"USDT"`, which is what every
/// caller means in practice, but is not literally the same asset.
pub(crate) fn quote_asset(vs_currency: &str) -> String {
    match vs_currency.trim().to_uppercase().as_str() {
        "USD" => "USDT".to_string(),
        other => other.to_string(),
    }
}

/// Build a Binance market symbol from a coin id/ticker and a quote currency.
pub(crate) fn pair(base_id: &str, vs_currency: &str) -> String {
    format!(
        "{}{}",
        coins::resolve_ticker(base_id),
        quote_asset(vs_currency)
    )
}

/// Split a Binance market symbol back into `(base, quote)`.
///
/// Returns `None` when the symbol ends in no recognised quote asset.
pub(crate) fn split_pair(symbol: &str) -> Option<(&str, &str)> {
    coins::split_on_quote(symbol, QUOTE_ASSETS)
}

/// Normalise any of the symbol spellings the library uses into a Binance
/// market symbol.
///
/// Accepts a separated pair (`BTC-USD`, `BTC/USDT`, `BTC_USD`) or an already
/// concatenated market (`BTCUSDT`). A bare asset with no quote (`BTC`) is
/// ambiguous and returns `None`.
pub(crate) fn parse_market(symbol: &str) -> Option<String> {
    let symbol = symbol.trim();
    if let Some((base, quote)) = symbol
        .split_once(['-', '/', '_'])
        .filter(|(b, q)| !b.is_empty() && !q.is_empty())
    {
        return Some(pair(base, quote));
    }
    let upper = symbol.to_uppercase();
    split_pair(&upper).is_some().then_some(upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_maps_to_the_usdt_stablecoin() {
        // Binance spot lists no USD markets.
        assert_eq!(quote_asset("usd"), "USDT");
        assert_eq!(quote_asset("USDT"), "USDT");
        assert_eq!(quote_asset("eur"), "EUR");
    }

    #[test]
    fn pairs_are_built_from_id_and_quote() {
        assert_eq!(pair("bitcoin", "usd"), "BTCUSDT");
        assert_eq!(pair("ETH", "EUR"), "ETHEUR");
    }

    #[test]
    fn concatenated_markets_split_on_the_longest_quote_asset() {
        // "BTCUSDT" must not split as BTCUSD + T.
        assert_eq!(split_pair("BTCUSDT"), Some(("BTC", "USDT")));
        assert_eq!(split_pair("ETHBTC"), Some(("ETH", "BTC")));
        assert_eq!(split_pair("SOLFDUSD"), Some(("SOL", "FDUSD")));
        assert_eq!(split_pair("USDT"), None, "a bare quote asset is not a pair");
        assert_eq!(split_pair("BTC"), None);
    }

    #[test]
    fn separated_and_concatenated_spellings_both_parse() {
        assert_eq!(parse_market("BTC-USD").as_deref(), Some("BTCUSDT"));
        assert_eq!(parse_market("btc/usdt").as_deref(), Some("BTCUSDT"));
        assert_eq!(parse_market("bitcoin_eur").as_deref(), Some("BTCEUR"));
        assert_eq!(parse_market("BTCUSDT").as_deref(), Some("BTCUSDT"));
        assert_eq!(parse_market("ethbtc").as_deref(), Some("ETHBTC"));
        // A bare asset names no market.
        assert_eq!(parse_market("BTC"), None);
    }

    #[test]
    fn asset_names_are_available_for_the_majors() {
        assert_eq!(asset_name("BTC"), Some("Bitcoin"));
        assert_eq!(asset_name("WIF"), None);
    }
}
