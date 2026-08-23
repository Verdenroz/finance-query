//! External financial data source adapters (internal — use `Ticker` or re-export modules).
//!
//! Adapters are `pub(crate)` — users interact with data through [`Ticker`](crate::Ticker),
//! [`Tickers`](crate::Tickers), and the public re-export modules ([`edgar`](crate::edgar),
//! [`fred`](crate::fred), [`crypto`](crate::crypto)). The provider system automatically
//! dispatches to the right adapter based on configured providers.
//!
//! Each adapter follows the standard structure (mod, client, models, optional endpoints)
//! with capability-mapped subdirectories.
//!
//! # Available adapters
//!
//! | Feature | Provider | Free tier | Endpoints | Coverage |
//! |---------|----------|-----------|-----------|----------|
//! | `alphavantage` | [Alpha Vantage](https://www.alphavantage.co/) | 25 req/day | ~100 | Stocks, forex, crypto, commodities, economic indicators, 50+ technical indicators |
//! | `polygon` | [Massive](https://massive.com/) | 5 req/sec | ~100 | Stocks, options, forex, crypto, indices, futures, economy, analyst data, WebSocket streaming |
//! | `fmp` | [Financial Modeling Prep](https://financialmodelingprep.com/) | 250 req/day | ~100 | Fundamentals, DCF/ratings, insider trading, institutional holdings, screener, 60+ exchanges |
//! | `crypto` | [CoinGecko](https://www.coingecko.com/) | 30 req/min | 2 | Top coins by market cap, single coin quotes (keyless) |
//! | `fred` | [FRED](https://fred.stlouisfed.org/) | 120 req/min | 2+ | 800k+ macro time series, US Treasury yield curve |
//! | `worldbank` | [World Bank Open Data](https://data.worldbank.org/) | Keyless | 1 | ~1,600 global development/macro indicators across 200+ economies |
//! | `fiscaldata` | [US Treasury FiscalData](https://fiscaldata.treasury.gov/) | Keyless | 50+ datasets | Federal debt, average interest rates, daily Treasury statements |
//! | `bls` | [BLS Public Data](https://www.bls.gov/developers/) | Keyless (25/day) or keyed (500/day) | 1 | CPI, unemployment, payrolls, wages, PPI from the primary source |
//! | `frankfurter` | [Frankfurter](https://frankfurter.dev/) | Keyless | 1 | ECB daily reference exchange rates — the only keyless forex route |
//! | `binance` | [Binance public data](https://data-api.binance.vision/) | Keyless | 2 | Exchange-grade crypto quotes and arbitrary-interval OHLCV (geo-blocked in some regions) |
//! | `kraken` | [Kraken public data](https://api.kraken.com/) | Keyless | 2 | Crypto quotes and OHLC candles; US-accessible complement to Binance |
//! | `finra` | [FINRA Query API](https://developer.finra.org/) | Keyless (non-commercial) | 1 | Daily short-sale volume by security, from the primary source |
//! | `openfigi` | [OpenFIGI](https://www.openfigi.com/api) | Keyless (25 req/min) | 1 | CUSIP/ISIN/SEDOL/FIGI to ticker resolution |
//! | `defi` | [DefiLlama](https://defillama.com/docs/api) | Keyless | 3 | Protocol TVL (current + history), chain rankings, stablecoin supplies |
//! | `gdelt` | [GDELT DOC 2.0](https://www.gdeltproject.org/) | Keyless (~1 req/5s) | 1 | Worldwide online news search across 65 languages, updated every 15 minutes |
//! | `cftc` | [CFTC Public Reporting](https://publicreporting.cftc.gov/) | Keyless | 1 | Weekly Commitments of Traders futures positioning (disaggregated futures-only report) |
//! | `housetrades` | [House Clerk Financial Disclosure](https://disclosures-clerk.house.gov/FinancialDisclosure) | Keyless | 1 | House of Representatives Periodic Transaction Report (PTR) stock-trade disclosures |
//! | `senatetrades` | [Senate eFD](https://efdsearch.senate.gov/) | Keyless (headless-Chromium) | 1 | Senate Periodic Transaction Report (PTR) stock-trade disclosures |
//! | *(always)* | [SEC EDGAR](https://www.sec.gov/edgar) | 10 req/sec | 5+ | Filing history, XBRL financials, full-text search (keyless, requires contact email) |
//!
//! # Quick comparison
//!
//! - **Alpha Vantage**: Best free option for technical indicators (50+) and economic data. Lowest rate limits.
//! - **Massive (formerly Polygon.io)**: Tick-level trades/quotes, SEC filings, and real-time WebSocket streams. Best for market microstructure.
//! - **CoinGecko**: Best for cryptocurrency data. Keyless (public API). Rate-limited to 30 req/min.
//! - **FRED**: 800k+ US macro-economic time series (GDP, CPI, employment, interest rates). Free API key required.
//! - **SEC EDGAR**: Company filings, XBRL financials, full-text search. Always available, no feature flag needed.
//!
//! All adapters share:
//! - Singleton pattern with `init(api_key)` / `init_with_timeout(api_key, timeout)`
//! - Built-in rate limiting via shared token-bucket limiter
//! - Error mapping to [`crate::FinanceError`] variants
//! - Full mockito test coverage (no API key needed to run tests)

pub(crate) mod common;
pub(crate) mod singleton;

/// Alpha Vantage financial data API (requires `alphavantage` feature).
#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;

/// Massive financial data API (formerly Polygon.io; requires `polygon` feature).
#[cfg(feature = "polygon")]
pub(crate) mod polygon;

/// Financial Modeling Prep (FMP) financial data API (requires `fmp` feature).
#[cfg(feature = "fmp")]
pub(crate) mod fmp;

/// CoinGecko cryptocurrency data API (keyless, requires `crypto` feature).
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;

/// FRED economic data API (requires `fred` feature).
#[cfg(feature = "fred")]
pub(crate) mod fred;

/// World Bank Open Data global macro indicators (keyless, requires `worldbank` feature).
#[cfg(feature = "worldbank")]
pub(crate) mod worldbank;

/// US Treasury FiscalData federal fiscal statistics (keyless, requires `fiscaldata` feature).
#[cfg(feature = "fiscaldata")]
pub(crate) mod fiscaldata;

/// BLS labor and inflation statistics (keyless v1 / keyed v2, requires `bls` feature).
#[cfg(feature = "bls")]
pub(crate) mod bls;

/// Frankfurter ECB reference exchange rates (keyless, requires `frankfurter` feature).
#[cfg(feature = "frankfurter")]
pub(crate) mod frankfurter;

/// Binance public market data (keyless, requires `binance` feature).
#[cfg(feature = "binance")]
pub(crate) mod binance;

/// Kraken public market data (keyless, requires `kraken` feature).
#[cfg(feature = "kraken")]
pub(crate) mod kraken;

/// FINRA daily short-sale volume (keyless, requires `finra` feature).
#[cfg(feature = "finra")]
pub(crate) mod finra;

/// OpenFIGI security-identifier mapping (keyless, requires `openfigi` feature).
#[cfg(feature = "openfigi")]
pub(crate) mod openfigi;

/// DefiLlama DeFi TVL and stablecoin data (keyless, requires `defi` feature).
#[cfg(feature = "defi")]
pub(crate) mod defillama;

/// GDELT DOC 2.0 global news search (keyless, requires `gdelt` feature).
#[cfg(feature = "gdelt")]
pub(crate) mod gdelt;

/// CFTC Commitments of Traders futures positioning (keyless, requires `cftc` feature).
#[cfg(feature = "cftc")]
pub(crate) mod cftc;

/// House of Representatives PTR stock-trade disclosures (keyless, requires `housetrades` feature).
#[cfg(feature = "housetrades")]
pub(crate) mod housetrades;

/// Senate PTR stock-trade disclosures (keyless, browser-driven, requires `senatetrades` feature).
#[cfg(feature = "senatetrades")]
pub(crate) mod senatetrades;

/// SEC EDGAR API client (always available, requires init with contact email).
pub(crate) mod edgar;

/// Yahoo Finance API endpoints (always available — the canonical data source).
pub(crate) mod yahoo;
