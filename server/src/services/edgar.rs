use crate::cache::{self, Cache};

use super::{ServiceError, ServiceResult};

pub async fn get_cik(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("edgar_cik", &[&symbol.to_uppercase()]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let cik = finance_query::edgar::resolve_cik(&symbol).await?;
            let json = serde_json::json!({ "symbol": symbol, "cik": cik });
            Ok(json)
        })
        .await
}

pub async fn get_submissions(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("edgar_submissions", &[&symbol.to_uppercase()]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let cik = finance_query::edgar::resolve_cik(&symbol).await?;
            let submissions = finance_query::edgar::submissions(cik).await?;
            serde_json::to_value(&submissions).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

pub async fn get_facts(cache: &Cache, symbol: &str) -> ServiceResult {
    let cache_key = Cache::key("edgar_facts", &[&symbol.to_uppercase()]);
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let cik = finance_query::edgar::resolve_cik(&symbol).await?;
            let facts = finance_query::edgar::company_facts(cik).await?;
            serde_json::to_value(&facts).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}

pub async fn search_edgar(
    cache: &Cache,
    query: &str,
    forms: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    from: Option<usize>,
    size: Option<usize>,
) -> ServiceResult {
    let cache_parts = [
        query.to_string(),
        forms.unwrap_or_default().to_string(),
        start_date.unwrap_or_default().to_string(),
        end_date.unwrap_or_default().to_string(),
        from.map(|f| f.to_string()).unwrap_or_default(),
        size.map(|s| s.to_string()).unwrap_or_default(),
    ];
    let cache_key = Cache::key(
        "edgar_search",
        &cache_parts.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    );
    let query = query.to_string();
    let forms = forms.map(|s| s.to_string());
    let start_date = start_date.map(|s| s.to_string());
    let end_date = end_date.map(|s| s.to_string());

    cache
        .get_or_fetch(&cache_key, cache::ttl::ANALYSIS, false, || async move {
            let forms_vec: Option<Vec<&str>> = forms.as_ref().map(|f| f.split(',').collect());
            let results = finance_query::edgar::search(
                &query,
                forms_vec.as_deref(),
                start_date.as_deref(),
                end_date.as_deref(),
                from,
                size,
            )
            .await?;
            serde_json::to_value(&results).map_err(|e| Box::new(e) as ServiceError)
        })
        .await
}
