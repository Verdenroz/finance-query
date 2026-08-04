//! Coin vocabulary shared by the keyless crypto exchange adapters.
//!
//! Binance and Kraken both name markets as one concatenated string
//! (`BTCUSDT`, `XBTUSD`) while the rest of the library speaks in CoinGecko-style
//! ids (`bitcoin`) and tickers (`BTC`). The id table and the splitting rule are
//! identical for both, so they live here — a coin added for one exchange then
//! resolves on the other too. Each adapter keeps only what is genuinely its
//! own: its quote-asset list and its exchange-specific asset aliases.

/// CoinGecko-style ids for the majors, as `(id, ticker, display name)`.
///
/// Deliberately partial — exchanges have no id concept, and mirroring
/// CoinGecko's full catalogue here would rot. Anything absent falls through as
/// a ticker. A listed coin an exchange does not trade simply fails upstream as
/// an unknown market, which is the same answer the user would get anyway.
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
    ("algorand", "ALGO", "Algorand"),
    ("tezos", "XTZ", "Tezos"),
    ("aave", "AAVE", "Aave"),
];

/// Resolve a coin id or ticker to its common ticker (uppercase).
///
/// Unknown ids are uppercased and passed through rather than rejected, so a
/// coin missing from the table still reaches the exchange.
pub(crate) fn resolve_ticker(id: &str) -> String {
    let id = id.trim();
    COIN_IDS
        .iter()
        .find(|(coin_id, _, _)| coin_id.eq_ignore_ascii_case(id))
        .map(|(_, ticker, _)| (*ticker).to_string())
        .unwrap_or_else(|| id.to_uppercase())
}

/// The display name for a ticker, when it is one of the majors.
pub(crate) fn asset_name(ticker: &str) -> Option<&'static str> {
    COIN_IDS
        .iter()
        .find(|(_, t, _)| t.eq_ignore_ascii_case(ticker))
        .map(|(_, _, name)| *name)
}

/// Split a concatenated market symbol into `(base, quote)` on the first
/// matching entry of `quotes`.
///
/// `quotes` must be ordered longest-first, or `BTCUSDT` splits as `BTCUSD`/`T`.
/// Returns `None` when the symbol ends in no listed quote asset, or is nothing
/// but a quote asset.
pub(crate) fn split_on_quote<'a>(symbol: &'a str, quotes: &[&str]) -> Option<(&'a str, &'a str)> {
    quotes
        .iter()
        .find(|q| symbol.len() > q.len() && symbol.ends_with(*q))
        .map(|q| symbol.split_at(symbol.len() - q.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_resolve_to_tickers_case_insensitively() {
        assert_eq!(resolve_ticker("bitcoin"), "BTC");
        assert_eq!(resolve_ticker("Ethereum"), "ETH");
        assert_eq!(resolve_ticker(" solana "), "SOL");
    }

    #[test]
    fn unknown_ids_fall_through_as_tickers() {
        assert_eq!(resolve_ticker("wif"), "WIF");
        assert_eq!(resolve_ticker("btc"), "BTC");
    }

    #[test]
    fn names_are_available_for_the_majors() {
        assert_eq!(asset_name("BTC"), Some("Bitcoin"));
        assert_eq!(asset_name("aave"), Some("Aave"));
        assert_eq!(asset_name("WIF"), None);
    }

    #[test]
    fn every_id_and_ticker_is_listed_once() {
        for (i, (id, ticker, _)) in COIN_IDS.iter().enumerate() {
            let dupe = COIN_IDS
                .iter()
                .skip(i + 1)
                .any(|(o_id, o_ticker, _)| o_id == id || o_ticker == ticker);
            assert!(!dupe, "{id}/{ticker} appears twice");
        }
    }

    #[test]
    fn markets_split_on_the_longest_quote_asset() {
        let quotes = &["USDT", "USDC", "USD", "BTC"];
        assert_eq!(split_on_quote("BTCUSDT", quotes), Some(("BTC", "USDT")));
        assert_eq!(split_on_quote("SOLUSD", quotes), Some(("SOL", "USD")));
        assert_eq!(split_on_quote("ETHBTC", quotes), Some(("ETH", "BTC")));
    }

    #[test]
    fn a_bare_quote_asset_is_not_a_market() {
        let quotes = &["USDT", "USD"];
        assert_eq!(split_on_quote("USDT", quotes), None);
        assert_eq!(split_on_quote("BTC", quotes), None);
    }
}
