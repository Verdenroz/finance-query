use super::urls::api;
/// Search endpoint
///
/// Search for quotes, news, and research reports on Yahoo Finance.
use crate::client::YahooClient;
use crate::constants::Country;
use crate::error::Result;
use tracing::info;

/// Search configuration options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of quote results (default: 10)
    pub quotes_count: u32,
    /// Maximum number of news results (default: 0 = disabled)
    pub news_count: u32,
    /// Enable fuzzy matching for typos (default: false)
    pub enable_fuzzy_query: bool,
    /// Enable logo URLs in results (default: true)
    pub enable_logo_url: bool,
    /// Enable research reports in results (default: false)
    pub enable_research_reports: bool,
    /// Enable cultural assets (NFT indices) in results (default: false)
    pub enable_cultural_assets: bool,
    /// Recommended count (default: 5)
    pub recommend_count: u32,
    /// Country for language/region settings. If None, uses client default.
    pub country: Option<Country>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            quotes_count: 10,
            news_count: 0,
            enable_fuzzy_query: false,
            enable_logo_url: true,
            enable_research_reports: false,
            enable_cultural_assets: false,
            recommend_count: 5,
            country: None,
        }
    }
}

impl SearchOptions {
    /// Create new search options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum quote results
    pub fn quotes_count(mut self, count: u32) -> Self {
        self.quotes_count = count;
        self
    }

    /// Set maximum news results
    pub fn news_count(mut self, count: u32) -> Self {
        self.news_count = count;
        self
    }

    /// Enable or disable fuzzy query matching
    pub fn enable_fuzzy_query(mut self, enable: bool) -> Self {
        self.enable_fuzzy_query = enable;
        self
    }

    /// Enable or disable logo URLs
    pub fn enable_logo_url(mut self, enable: bool) -> Self {
        self.enable_logo_url = enable;
        self
    }

    /// Enable or disable research reports
    pub fn enable_research_reports(mut self, enable: bool) -> Self {
        self.enable_research_reports = enable;
        self
    }

    /// Enable or disable cultural assets (NFT indices)
    pub fn enable_cultural_assets(mut self, enable: bool) -> Self {
        self.enable_cultural_assets = enable;
        self
    }

    /// Set recommend count
    pub fn recommend_count(mut self, count: u32) -> Self {
        self.recommend_count = count;
        self
    }

    /// Set country for language/region settings
    pub fn country(mut self, country: Country) -> Self {
        self.country = Some(country);
        self
    }
}

/// Search for quotes, news, and research reports
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `query` - Search query string
/// * `options` - Search configuration options
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::search::{fetch, SearchOptions};
/// let options = SearchOptions::new().quotes_count(10).news_count(5);
/// let results = fetch(&client, "Apple", &options).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    query: &str,
    options: &SearchOptions,
) -> Result<serde_json::Value> {
    if query.trim().is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "query".to_string(),
            reason: "Empty search query".to_string(),
        });
    }

    info!("Searching for: {} (options: {:?})", query, options);

    let quotes_count = options.quotes_count.to_string();
    let news_count = options.news_count.to_string();
    let fuzzy = options.enable_fuzzy_query.to_string();
    let logo = options.enable_logo_url.to_string();
    let research = options.enable_research_reports.to_string();
    let cultural = options.enable_cultural_assets.to_string();
    let recommend = options.recommend_count.to_string();

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
        ("q", query),
        ("lang", &lang),
        ("region", &region),
        ("quotesCount", &quotes_count),
        ("newsCount", &news_count),
        ("enableFuzzyQuery", &fuzzy),
        ("enableLogoUrl", &logo),
        ("enableResearchReports", &research),
        ("enableCulturalAssets", &cultural),
        ("recommendedCount", &recommend),
        ("listsCount", "0"),         // Disable Yahoo-specific lists
        ("enableNavLinks", "false"), // Disable Yahoo navigation links
        ("enableEnhancedTrivialQuery", "true"),
    ];

    let response = client.request_with_params(api::SEARCH, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_search() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = SearchOptions::new().quotes_count(5);
        let result = fetch(&client, "Apple", &options).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quotes").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_search_with_news() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = SearchOptions::new()
            .quotes_count(5)
            .news_count(3)
            .enable_research_reports(true);
        let result = fetch(&client, "NVDA", &options).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quotes").is_some());
    }

    #[tokio::test]
    async fn test_empty_query() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = SearchOptions::new();
        let result = fetch(&client, "", &options).await;
        assert!(result.is_err());
    }
}
