//! Bounded live measurement, never an ingestion job. Run one explicit mode per invocation.
//! Reads FMP_API_KEY / POLYGON_API_KEY from the environment, never prints them.
use std::time::{Duration, Instant};

use finance_query::{
    Capability, CompanyProfile, FinanceError, Provider, Providers, StockListingRequest,
};
use futures::{StreamExt, stream};
use serde_json::{Value, json};

// Fixed before the run: large/small, dual share classes, ADRs and two delisted names.
// Provider-specific punctuation is deliberate; these are test requests, not identity mappings.
const SYMBOLS: [&str; 24] = [
    "AAPL", "MSFT", "GOOGL", "GOOG", "AMZN", "META", "NVDA", "TSLA", "JPM", "V", "XOM", "JNJ",
    "WMT", "KO", "DIS", "BRK-B", "BABA", "TSM", "ASML", "ABNB", "SNOW", "UBER", "TWTR", "LEH",
];

fn error_kind(error: &FinanceError) -> &'static str {
    match error {
        FinanceError::AuthenticationFailed { .. } => "access_or_authentication",
        FinanceError::RateLimited { .. } => "rate_limited",
        FinanceError::SymbolNotFound { .. } => "not_found",
        FinanceError::ResponseStructureError { .. } => "response_structure",
        FinanceError::ServerError { .. } => "provider_server",
        FinanceError::Timeout { .. } => "timeout",
        FinanceError::NetworkError { .. } => "network",
        _ => "other_request_error",
    }
}

fn present(value: &Option<String>) -> bool {
    value.as_ref().is_some_and(|s| !s.trim().is_empty())
}

fn profiles_summary(rows: &[CompanyProfile]) -> Value {
    let selected: Vec<_> = rows
        .iter()
        .filter(|row| row.symbol.as_deref().is_some_and(|s| SYMBOLS.contains(&s)))
        .map(|row| {
            json!({
                "symbol":row.symbol,"name":row.name,"cik":row.cik,"cusip":row.cusip,
                "isin":row.isin,"ipo_date":row.ipo_date,"exchange":row.exchange,
                "active":row.active,"is_adr":row.is_adr,"is_etf":row.is_etf,"is_fund":row.is_fund
            })
        })
        .collect();
    let missing: Vec<_> = SYMBOLS
        .iter()
        .filter(|s| !rows.iter().any(|r| r.symbol.as_deref() == Some(**s)))
        .collect();
    json!({"rows":rows.len(),"fixed_dataset_size":SYMBOLS.len(),"missing_symbols":missing,"selected":selected,
        "field_counts":{"name":rows.iter().filter(|r|present(&r.name)).count(),
        "cik":rows.iter().filter(|r|present(&r.cik)).count(),"cusip":rows.iter().filter(|r|present(&r.cusip)).count(),
        "isin":rows.iter().filter(|r|present(&r.isin)).count(),"ipo_date":rows.iter().filter(|r|present(&r.ipo_date)).count(),
        "exchange":rows.iter().filter(|r|present(&r.exchange)).count(),"active":rows.iter().filter(|r|r.active.is_some()).count()}})
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("");
    if !matches!(mode, "single" | "bulk" | "directories" | "polygon") {
        return Err(
            "usage: fmp_catalog_benchmark single|directories|bulk <part>|polygon <YYYY-MM-DD>"
                .into(),
        );
    }
    let part = if mode == "bulk" {
        Some(
            args.get(1)
                .ok_or("bulk requires an explicit part")?
                .parse::<u32>()?,
        )
    } else {
        None
    };
    let date = if mode == "polygon" {
        Some(
            args.get(1)
                .ok_or("polygon requires an explicit date")?
                .as_str(),
        )
    } else {
        None
    };
    let provider = if mode == "polygon" {
        Provider::Polygon
    } else {
        Provider::Fmp
    };
    let key_name = if mode == "polygon" {
        "POLYGON_API_KEY"
    } else {
        "FMP_API_KEY"
    };
    let key = std::env::var(key_name).map_err(|_| "required API key is absent")?;
    let providers = Providers::builder()
        .providers([provider])
        .api_key(provider, key)
        .route(Capability::DISCOVERY, [provider])
        .route(Capability::FUNDAMENTALS, [provider])
        .requests_per_minute(provider, 300)
        .timeout(Duration::from_secs(180))
        .build()
        .await?;
    let started = Instant::now();
    let mut requests = 0;
    let data = match mode {
        "single" => {
            let results = stream::iter(SYMBOLS)
                .map(|symbol| {
                    let providers = &providers;
                    async move {
                        let result = match providers.ticker(symbol).build().await {
                            Ok(ticker) => ticker.company_profile().await,
                            Err(error) => Err(error),
                        };
                        (symbol, result)
                    }
                })
                .buffer_unordered(5)
                .collect::<Vec<_>>()
                .await;
            requests = SYMBOLS.len();
            let mut rows = Vec::new();
            let mut errors = Vec::new();
            for (symbol, result) in results {
                match result {
                    Ok(row) => rows.push(row),
                    Err(error) => errors.push(json!({"symbol":symbol,"error":error_kind(&error)})),
                }
            }
            rows.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            json!({"summary":profiles_summary(&rows),"errors":errors})
        }
        "bulk" => {
            requests = 1;
            match providers
                .discovery()
                .company_profiles_bulk(part.unwrap())
                .await
            {
                Ok(rows) => json!({"part":part,"summary":profiles_summary(&rows)}),
                Err(error) => json!({"part":part,"error":error_kind(&error)}),
            }
        }
        "directories" => {
            let mut results = Vec::new();
            for active in [false, true] {
                let start = Instant::now();
                requests += 1;
                let result = if active {
                    providers.discovery().listing_status(true).await
                } else {
                    providers.discovery().stock_list().await
                };
                results.push(match result {
                    Ok(rows) => json!({"active_only":active,"elapsed_ms":start.elapsed().as_millis(),"rows":rows.len(),
                        "names":rows.iter().filter(|r|present(&r.name)).count(),"exchanges":rows.iter().filter(|r|present(&r.exchange)).count(),
                        "missing_symbols":SYMBOLS.iter().filter(|s|!rows.iter().any(|r|r.symbol==**s)).collect::<Vec<_>>() }),
                    Err(error) => json!({"active_only":active,"elapsed_ms":start.elapsed().as_millis(),"error":error_kind(&error)})
                });
            }
            json!(results)
        }
        "polygon" => {
            let mut results = Vec::new();
            for active in [true, false] {
                let request = StockListingRequest::new(date.unwrap(), active)?
                    .stock_type("CS")
                    .page_size(10)?;
                requests += 1;
                results.push(match providers.discovery().stock_listings_page(&request,None).await {
                    Ok(page) => json!({"active":active,"rows":page.items.len(),"has_more":page.next.is_some(),
                        "figi":page.items.iter().filter(|r|present(&r.composite_figi)).count(),
                        "names":page.items.iter().filter(|r|present(&r.name)).count()}),
                    Err(error) => json!({"active":active,"error":error_kind(&error)})
                });
            }
            json!({"date":date,"pages":results})
        }
        _ => unreachable!(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"mode":mode,"recorded_at":chrono::Utc::now().to_rfc3339(),
        "dataset":SYMBOLS,"requests":requests,"retries":0,"concurrency":if mode=="single"{5}else{1},
        "timeout_seconds":180,"requests_per_minute":300,"elapsed_ms":started.elapsed().as_millis(),"data":data})
        )?
    );
    Ok(())
}
