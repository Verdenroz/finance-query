use super::urls::api;
/// Lookup endpoint
///
/// Type-filtered symbol lookup on Yahoo Finance.
/// Unlike search, lookup specializes in discovering tickers by type
/// (equity, ETF, mutual fund, index, future, currency, cryptocurrency).
use crate::client::YahooClient;
use crate::constants::Country;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::info;

/// Asset types available for lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LookupType {
    /// All asset types
    #[default]
    All,
    /// Stocks/equities
    Equity,
    /// Mutual funds
    #[serde(rename = "mutualfund")]
    MutualFund,
    /// Exchange-traded funds
    #[serde(rename = "etf")]
    Etf,
    /// Market indices
    Index,
    /// Futures contracts
    Future,
    /// Fiat currencies
    Currency,
    /// Cryptocurrencies
    Cryptocurrency,
}

impl fmt::Display for LookupType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LookupType::All => write!(f, "all"),
            LookupType::Equity => write!(f, "equity"),
            LookupType::MutualFund => write!(f, "mutualfund"),
            LookupType::Etf => write!(f, "etf"),
            LookupType::Index => write!(f, "index"),
            LookupType::Future => write!(f, "future"),
            LookupType::Currency => write!(f, "currency"),
            LookupType::Cryptocurrency => write!(f, "cryptocurrency"),
        }
    }
}

/// Lookup configuration options
#[derive(Debug, Clone)]
pub struct LookupOptions {
    /// Asset type to search for (default: All)
    pub lookup_type: LookupType,
    /// Maximum number of results (default: 25)
    pub count: u32,
    /// Include logo URLs by fetching from quotes endpoint (default: false)
    /// Note: This requires an additional API call for symbols returned
    pub include_logo: bool,
    /// Include pricing data (default: true)
    pub fetch_pricing_data: bool,
    /// Country for language/region settings. If None, uses client default.
    pub country: Option<Country>,
}

impl Default for LookupOptions {
    fn default() -> Self {
        Self {
            lookup_type: LookupType::All,
            count: 25,
            include_logo: false,
            fetch_pricing_data: true,
            country: None,
        }
    }
}

impl LookupOptions {
    /// Create new lookup options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the asset type to look up
    pub fn lookup_type(mut self, lookup_type: LookupType) -> Self {
        self.lookup_type = lookup_type;
        self
    }

    /// Set maximum number of results
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Enable or disable logo URL fetching
    /// Note: When enabled, an additional API call is made to fetch logos
    pub fn include_logo(mut self, include: bool) -> Self {
        self.include_logo = include;
        self
    }

    /// Enable or disable pricing data
    pub fn fetch_pricing_data(mut self, fetch: bool) -> Self {
        self.fetch_pricing_data = fetch;
        self
    }

    /// Set country for language/region settings
    pub fn country(mut self, country: Country) -> Self {
        self.country = Some(country);
        self
    }
}

/// Fetch lookup results for a query
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `query` - Search query string
/// * `options` - Lookup configuration options
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::lookup::{fetch, LookupOptions, LookupType};
/// let options = LookupOptions::new()
///     .lookup_type(LookupType::Equity)
///     .count(10)
///     .include_logo(true);
/// let results = fetch(&client, "Apple", &options).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    query: &str,
    options: &LookupOptions,
) -> Result<serde_json::Value> {
    if query.trim().is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "query".to_string(),
            reason: "Empty lookup query".to_string(),
        });
    }

    info!(
        "Looking up: {} (type: {}, count: {}, include_logo: {})",
        query, options.lookup_type, options.count, options.include_logo
    );

    let count = options.count.to_string();
    let lookup_type = options.lookup_type.to_string();
    let fetch_pricing = options.fetch_pricing_data.to_string();

    // Use provided country's lang/region or fall back to client config
    let lang = options
        .country
        .as_ref()
        .map(|c| c.lang().to_string())
        .unwrap_or_else(|| client.config().lang.clone());
    let region = options
        .country
        .as_ref()
        .map(|c| c.region().to_string())
        .unwrap_or_else(|| client.config().region.clone());

    let params = [
        ("query", query),
        ("type", &lookup_type),
        ("start", "0"),
        ("count", &count),
        ("formatted", "false"),
        ("fetchPricingData", &fetch_pricing),
        ("lang", &lang),
        ("region", &region),
    ];

    let response = client.request_with_params(api::LOOKUP, &params).await?;

    let mut json: serde_json::Value = response.json().await?;

    // If logo is requested, fetch logos for the returned symbols
    if options.include_logo {
        json = enrich_with_logos(client, json).await?;
    }

    Ok(json)
}

/// Enrich lookup results with logo URLs by fetching from quotes endpoint
async fn enrich_with_logos(
    client: &YahooClient,
    mut json: serde_json::Value,
) -> Result<serde_json::Value> {
    // Extract symbols from the response
    let symbols: Vec<String> = json
        .get("finance")
        .and_then(|f| f.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("documents"))
        .and_then(|docs| docs.as_array())
        .map(|docs| {
            docs.iter()
                .filter_map(|doc| doc.get("symbol").and_then(|s| s.as_str()))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    if symbols.is_empty() {
        return Ok(json);
    }

    info!("Fetching logos for {} symbols", symbols.len());

    // Fetch logos from quotes endpoint
    let symbol_refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
    let logo_fields = ["logoUrl", "companyLogoUrl"];
    let logos_json = crate::endpoints::quotes::fetch_with_fields(
        client,
        &symbol_refs,
        Some(&logo_fields),
        false,
        true, // include_logo = true to get logo dimensions
    )
    .await?;

    // Build a map of symbol -> logo URLs
    let logo_map: std::collections::HashMap<String, (Option<String>, Option<String>)> = logos_json
        .get("quoteResponse")
        .and_then(|qr| qr.get("result"))
        .and_then(|r| r.as_array())
        .map(|quotes| {
            quotes
                .iter()
                .filter_map(|q| {
                    let symbol = q.get("symbol")?.as_str()?.to_string();
                    let logo_url = q.get("logoUrl").and_then(|u| u.as_str()).map(String::from);
                    let company_logo_url = q
                        .get("companyLogoUrl")
                        .and_then(|u| u.as_str())
                        .map(String::from);
                    Some((symbol, (logo_url, company_logo_url)))
                })
                .collect()
        })
        .unwrap_or_default();

    // Inject logos into the lookup response
    if let Some(documents) = json
        .get_mut("finance")
        .and_then(|f| f.get_mut("result"))
        .and_then(|r| r.as_array_mut())
        .and_then(|arr| arr.first_mut())
        .and_then(|first| first.get_mut("documents"))
        .and_then(|docs| docs.as_array_mut())
    {
        for doc in documents.iter_mut() {
            if let Some(symbol) = doc.get("symbol").and_then(|s| s.as_str())
                && let Some((logo_url, company_logo_url)) = logo_map.get(symbol)
            {
                if let Some(url) = logo_url {
                    doc.as_object_mut()
                        .map(|obj| obj.insert("logoUrl".to_string(), serde_json::json!(url)));
                }
                if let Some(url) = company_logo_url {
                    doc.as_object_mut().map(|obj| {
                        obj.insert("companyLogoUrl".to_string(), serde_json::json!(url))
                    });
                }
            }
        }
    }

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_lookup() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = LookupOptions::new().count(5);
        let result = fetch(&client, "Apple", &options).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("finance").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_lookup_equity() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = LookupOptions::new()
            .lookup_type(LookupType::Equity)
            .count(5);
        let result = fetch(&client, "NVDA", &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_lookup_with_logo() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = LookupOptions::new()
            .lookup_type(LookupType::Equity)
            .count(3)
            .include_logo(true);
        let result = fetch(&client, "Apple", &options).await;
        assert!(result.is_ok());
        // Check that logos were enriched
        let json = result.unwrap();
        if let Some(doc) = json
            .get("finance")
            .and_then(|f| f.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("documents"))
            .and_then(|docs| docs.as_array())
            .and_then(|docs| docs.first())
        {
            // Logo should be present if symbol was found in quotes
            println!("Document with logo: {:?}", doc);
        }
    }

    #[tokio::test]
    async fn test_empty_query() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = LookupOptions::new();
        let result = fetch(&client, "", &options).await;
        assert!(result.is_err());
    }
}
