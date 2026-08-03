//! Symbol normalisation for Binance spot markets.
//!
//! Binance names a market as one concatenated string (`BTCUSDT`) with no
//! separator, while the rest of the library speaks in coin ids (`bitcoin`),
//! tickers (`BTC`), and separated pairs (`BTC-USD`). Everything is funnelled
//! through here so the two conventions meet in exactly one place.

/// Quote assets Binance actually lists, longest first.
///
/// Order matters: `BTCUSDT` must split as `BTC`/`USDT`, not `BTCUSD`/`T`.
const QUOTE_ASSETS: &[&str] = &[
    "FDUSD", "USDT", "USDC", "TUSD", "BUSD", "TRY", "BRL", "EUR", "GBP", "JPY", "ARS", "BNB",
    "BTC", "ETH", "DAI",
];

/// CoinGecko-style ids for the majors, so `providers.crypto("bitcoin")` works
/// on a Binance route as well as a CoinGecko one.
///
/// Deliberately partial — Binance has no id concept, and mirroring CoinGecko's
/// full catalogue here would rot. Anything absent falls through as a ticker.
const COIN_IDS: &[(&str, &str, &str)] = &[
    ("bitcoin", "BTC", "Bitcoin"),
    ("ethereum", "ETH", "Ethereum"),
    ("binancecoin", "BNB", "BNB"),
    ("ripple", "XRP", "XRP"),
    ("cardano", "ADA", "Cardano"),
    ("solana", "SOL", "Solana"),
    ("dogecoin", "DOGE", "Dogecoin"),
    ("polkadot", "DOT", "Polkadot"),
    ("tron", "TRX", "TRON"),
    ("avalanche-2", "AVAX", "Avalanche"),
    ("chainlink", "LINK", "Chainlink"),
    ("polygon-ecosystem-token", "POL", "Polygon"),
    ("litecoin", "LTC", "Litecoin"),
    ("shiba-inu", "SHIB", "Shiba Inu"),
    ("uniswap", "UNI", "Uniswap"),
    ("stellar", "XLM", "Stellar"),
    ("cosmos", "ATOM", "Cosmos"),
    ("monero", "XMR", "Monero"),
    ("ethereum-classic", "ETC", "Ethereum Classic"),
    ("filecoin", "FIL", "Filecoin"),
    ("aptos", "APT", "Aptos"),
    ("arbitrum", "ARB", "Arbitrum"),
    ("optimism", "OP", "Optimism"),
    ("near", "NEAR", "NEAR Protocol"),
    ("injective-protocol", "INJ", "Injective"),
];

/// Resolve a coin id or ticker to its Binance base asset.
pub(crate) fn base_asset(id: &str) -> String {
    let id = id.trim();
    COIN_IDS
        .iter()
        .find(|(coin_id, _, _)| coin_id.eq_ignore_ascii_case(id))
        .map(|(_, ticker, _)| (*ticker).to_string())
        .unwrap_or_else(|| id.to_uppercase())
}

/// The display name for a base asset, when it is one of the majors.
pub(crate) fn asset_name(ticker: &str) -> Option<&'static str> {
    COIN_IDS
        .iter()
        .find(|(_, t, _)| t.eq_ignore_ascii_case(ticker))
        .map(|(_, _, name)| *name)
}

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
    format!("{}{}", base_asset(base_id), quote_asset(vs_currency))
}

/// Split a Binance market symbol back into `(base, quote)`.
///
/// Returns `None` when the symbol ends in no recognised quote asset.
pub(crate) fn split_pair(symbol: &str) -> Option<(&str, &str)> {
    QUOTE_ASSETS
        .iter()
        .find(|q| symbol.len() > q.len() && symbol.ends_with(*q))
        .map(|q| symbol.split_at(symbol.len() - q.len()))
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
        return Some(format!("{}{}", base_asset(base), quote_asset(quote)));
    }
    let upper = symbol.to_uppercase();
    split_pair(&upper).is_some().then_some(upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_ids_resolve_to_tickers() {
        assert_eq!(base_asset("bitcoin"), "BTC");
        assert_eq!(base_asset("Ethereum"), "ETH");
        // Unknown ids fall through as tickers rather than failing.
        assert_eq!(base_asset("wif"), "WIF");
        assert_eq!(base_asset("btc"), "BTC");
    }

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
