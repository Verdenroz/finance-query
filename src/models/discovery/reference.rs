//! Provider-routed symbol reference models.
//!
//! Returned by the [`Capability::DISCOVERY`](crate::Capability::DISCOVERY)
//! route via [`Providers::discovery`](crate::Providers::discovery). These are
//! provider-neutral shapes — unlike [`SearchResults`](super::search::SearchResults),
//! which mirrors Yahoo's response for the [`crate::finance::search`] shortcut.

use serde::{Deserialize, Serialize};

/// A symbol matched by a search or listing query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SymbolMatch {
    /// Ticker symbol.
    pub symbol: String,
    /// Security or company name.
    pub name: Option<String>,
    /// Listing exchange, as reported by the provider.
    pub exchange: Option<String>,
    /// Asset type (e.g. `"CS"` for common stock, `"ETF"`).
    pub asset_type: Option<String>,
    /// Quote currency.
    pub currency: Option<String>,
    /// Whether the symbol is currently active/tradable.
    pub active: Option<bool>,
}

/// Detailed reference data for a single symbol.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SymbolDetails {
    /// Ticker symbol.
    pub symbol: String,
    /// Company name.
    pub name: Option<String>,
    /// Business description.
    pub description: Option<String>,
    /// Primary listing exchange.
    pub exchange: Option<String>,
    /// Asset type.
    pub asset_type: Option<String>,
    /// SEC Central Index Key.
    pub cik: Option<String>,
    /// SIC classification code.
    pub sic_code: Option<String>,
    /// SIC classification description.
    pub sic_description: Option<String>,
    /// Company homepage.
    pub homepage_url: Option<String>,
    /// Total employees.
    pub employees: Option<u64>,
    /// Market capitalisation.
    pub market_cap: Option<f64>,
    /// Date the security was listed (`YYYY-MM-DD`).
    pub list_date: Option<String>,
    /// Shares outstanding, weighted across share classes.
    pub shares_outstanding: Option<f64>,
}

/// A tradable exchange.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExchangeInfo {
    /// Provider-assigned exchange ID.
    pub id: Option<i64>,
    /// Exchange name.
    pub name: Option<String>,
    /// ISO 10383 Market Identifier Code.
    pub mic: Option<String>,
    /// Operating MIC of the parent venue.
    pub operating_mic: Option<String>,
    /// Asset class traded (e.g. `"stocks"`, `"options"`).
    pub asset_class: Option<String>,
    /// Locale (e.g. `"us"`).
    pub locale: Option<String>,
    /// Venue type (e.g. `"exchange"`, `"TRF"`).
    pub exchange_type: Option<String>,
    /// Exchange homepage.
    pub url: Option<String>,
}

/// A symbol matched by a screener query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScreenerMatch {
    /// Ticker symbol.
    pub symbol: String,
    /// Company name.
    pub name: Option<String>,
    /// Latest price.
    pub price: Option<f64>,
    /// Market capitalisation.
    pub market_cap: Option<f64>,
    /// GICS-style sector.
    pub sector: Option<String>,
    /// Industry classification.
    pub industry: Option<String>,
    /// Beta against the broad market.
    pub beta: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Listing exchange.
    pub exchange: Option<String>,
    /// Country of domicile.
    pub country: Option<String>,
    /// Whether the symbol is an ETF.
    pub is_etf: Option<bool>,
    /// Whether the symbol is actively trading.
    pub is_actively_trading: Option<bool>,
}

/// Filters for a provider-routed screener query.
///
/// All fields are optional; unset filters are omitted from the request. Build
/// with [`ScreenerFilters::new`] and the chainable setters.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct ScreenerFilters {
    /// Minimum market capitalisation.
    pub market_cap_min: Option<f64>,
    /// Maximum market capitalisation.
    pub market_cap_max: Option<f64>,
    /// Minimum price.
    pub price_min: Option<f64>,
    /// Maximum price.
    pub price_max: Option<f64>,
    /// Minimum trading volume.
    pub volume_min: Option<f64>,
    /// Minimum beta.
    pub beta_min: Option<f64>,
    /// Maximum beta.
    pub beta_max: Option<f64>,
    /// Sector name filter.
    pub sector: Option<String>,
    /// Industry name filter.
    pub industry: Option<String>,
    /// Exchange filter.
    pub exchange: Option<String>,
    /// Country filter.
    pub country: Option<String>,
    /// Restrict to actively trading symbols.
    pub actively_trading: Option<bool>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl ScreenerFilters {
    /// An empty filter set — matches the provider's default universe.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restrict market capitalisation to `[min, max]`.
    pub fn market_cap(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.market_cap_min = min;
        self.market_cap_max = max;
        self
    }

    /// Restrict price to `[min, max]`.
    pub fn price(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.price_min = min;
        self.price_max = max;
        self
    }

    /// Restrict beta to `[min, max]`.
    pub fn beta(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.beta_min = min;
        self.beta_max = max;
        self
    }

    /// Require at least `min` traded volume.
    pub fn volume_min(mut self, min: f64) -> Self {
        self.volume_min = Some(min);
        self
    }

    /// Restrict to a sector.
    pub fn sector(mut self, sector: impl Into<String>) -> Self {
        self.sector = Some(sector.into());
        self
    }

    /// Restrict to an industry.
    pub fn industry(mut self, industry: impl Into<String>) -> Self {
        self.industry = Some(industry.into());
        self
    }

    /// Restrict to an exchange.
    pub fn exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = Some(exchange.into());
        self
    }

    /// Restrict to a country of domicile.
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Restrict to symbols that are actively trading.
    pub fn actively_trading(mut self, actively_trading: bool) -> Self {
        self.actively_trading = Some(actively_trading);
        self
    }

    /// Cap the number of results returned.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Render as provider query-string pairs, omitting unset filters.
    ///
    /// Uses Financial Modeling Prep's parameter names — the only provider
    /// currently routed for `Capability::DISCOVERY` screening.
    pub(crate) fn to_query(&self) -> Vec<(&'static str, String)> {
        let mut q: Vec<(&'static str, String)> = Vec::new();
        let mut num = |k: &'static str, v: Option<f64>| {
            if let Some(v) = v {
                q.push((k, v.to_string()));
            }
        };
        num("marketCapMoreThan", self.market_cap_min);
        num("marketCapLowerThan", self.market_cap_max);
        num("priceMoreThan", self.price_min);
        num("priceLowerThan", self.price_max);
        num("volumeMoreThan", self.volume_min);
        num("betaMoreThan", self.beta_min);
        num("betaLowerThan", self.beta_max);
        for (k, v) in [
            ("sector", &self.sector),
            ("industry", &self.industry),
            ("exchange", &self.exchange),
            ("country", &self.country),
        ] {
            if let Some(v) = v {
                q.push((k, v.clone()));
            }
        }
        if let Some(v) = self.actively_trading {
            q.push(("isActivelyTrading", v.to_string()));
        }
        if let Some(v) = self.limit {
            q.push(("limit", v.to_string()));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filters_render_no_query_params() {
        assert!(ScreenerFilters::new().to_query().is_empty());
    }

    #[test]
    fn set_filters_render_with_provider_parameter_names() {
        let q = ScreenerFilters::new()
            .market_cap(Some(1e9), None)
            .price(None, Some(50.0))
            .sector("Technology")
            .actively_trading(true)
            .limit(25)
            .to_query();

        assert_eq!(
            q,
            vec![
                ("marketCapMoreThan", "1000000000".to_string()),
                ("priceLowerThan", "50".to_string()),
                ("sector", "Technology".to_string()),
                ("isActivelyTrading", "true".to_string()),
                ("limit", "25".to_string()),
            ]
        );
    }
}
