/// Search endpoint
///
/// Search for quotes and news on Yahoo Finance.
use crate::client::YahooClient;
use crate::constants::endpoints;
use crate::error::Result;
use tracing::info;

/// Search for quotes and news
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `query` - Search query string
/// * `hits` - Maximum number of results to return
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::search;
/// let results = search::fetch(&client, "Apple", 6).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, query: &str, hits: u32) -> Result<serde_json::Value> {
    if query.trim().is_empty() {
        return Err(crate::error::YahooError::InvalidParameter(
            "Empty search query".to_string(),
        ));
    }

    info!("Searching for: {}", query);

    let params = [("q", query), ("quotesCount", &hits.to_string())];
    let response = client
        .request_with_params(endpoints::SEARCH, &params)
        .await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_search() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "Apple", 5).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quotes").is_some());
    }

    #[tokio::test]
    async fn test_empty_query() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", 5).await;
        assert!(result.is_err());
    }
}
