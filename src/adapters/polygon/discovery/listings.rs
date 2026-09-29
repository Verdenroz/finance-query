//! Dated stock-directory pages, dated ticker details, ticker changes and
//! stock-type codes.

use super::{TickerDetailsResponseDTO, TickerRefDTO, details_to_canonical};
use crate::adapters::polygon::client::PathRule;
use crate::adapters::polygon::models::{PaginatedResponseDTO, StockTypeDTO};
use crate::adapters::polygon::{build_client, invalid_page};
use crate::models::discovery::listings::date;
use crate::{
    FinanceError, PageCursor, Provider, ProviderPage, Result, StockListing, StockListingRequest,
    StockType, SymbolDetails, TickerChange,
};

fn validate_delisting(value: Option<&str>) -> Result<()> {
    if let Some(value) = value
        && chrono::DateTime::parse_from_rfc3339(value).is_err()
        && date(value).is_err()
    {
        return Err(invalid_page("delisted_utc"));
    }
    Ok(())
}

/// Rows keep the provider's reported status, locale and type rather than the
/// request's filters, which Polygon applies as of the queried date.
fn listing_to_canonical(row: TickerRefDTO) -> Result<StockListing> {
    let symbol = row
        .ticker
        .filter(|ticker| !ticker.is_empty())
        .ok_or_else(|| invalid_page("ticker"))?;
    if let Some(list_date) = &row.list_date {
        date(list_date).map_err(|_| invalid_page("list_date"))?;
    }
    validate_delisting(row.delisted_utc.as_deref())?;
    Ok(StockListing {
        symbol,
        name: row.name,
        exchange: row.primary_exchange,
        stock_type: row.asset_type,
        locale: row.locale,
        active: row.active,
        cik: row.cik,
        composite_figi: row.composite_figi,
        share_class_figi: row.share_class_figi,
        list_date: row.list_date,
        delisted_utc: row.delisted_utc,
    })
}

fn stock_type_to_canonical(row: StockTypeDTO) -> Result<StockType> {
    if row.code.is_empty() {
        return Err(invalid_page("code"));
    }
    Ok(StockType {
        code: row.code,
        description: row.description,
        asset_class: row.asset_class,
        locale: row.locale,
    })
}

pub(crate) async fn fetch_stock_listings_page(
    request: &StockListingRequest,
    cursor: Option<&PageCursor>,
) -> Result<ProviderPage<StockListing>> {
    if request.locale.is_empty() || request.stock_type.as_ref().is_some_and(|s| s.is_empty()) {
        return Err(FinanceError::InvalidParameter {
            param: "stock_listings".into(),
            reason: "locale and stock type must not be empty".into(),
        });
    }
    let client = build_client()?;
    let active = request.active.to_string();
    let limit = request.limit.to_string();
    let mut params = vec![
        ("market", "stocks"),
        ("locale", request.locale.as_str()),
        ("active", active.as_str()),
        ("date", request.date.as_str()),
        ("limit", limit.as_str()),
        ("sort", "ticker"),
        ("order", "asc"),
    ];
    if let Some(kind) = &request.stock_type {
        params.push(("type", kind.as_str()));
    }
    let path = "/v3/reference/tickers";
    let (body, current): (PaginatedResponseDTO<TickerRefDTO>, _) =
        client.page(path, &params, PathRule::Exact, cursor).await?;
    let items = body
        .results
        .ok_or_else(|| invalid_page("results"))?
        .into_iter()
        .map(listing_to_canonical)
        .collect::<Result<Vec<_>>>()?;
    let next = client.continuation(body.next_url, path, &params, PathRule::Exact, &current)?;
    Ok(ProviderPage {
        items,
        next,
        provider_id: Provider::Polygon,
        request_id: body.request_id,
        results_count: body.results_count,
        query_count: body.query_count,
        reported_symbol: body.ticker,
        adjusted: body.adjusted,
    })
}

pub(crate) async fn fetch_symbol_details_at(symbol: &str, as_of: &str) -> Result<SymbolDetails> {
    date(as_of)?;
    if symbol.is_empty() || symbol == "." || symbol == ".." {
        return Err(FinanceError::InvalidParameter {
            param: "symbol".into(),
            reason: "stock symbol is required".into(),
        });
    }
    let client = build_client()?;
    let path = format!(
        "/v3/reference/tickers/{}",
        crate::adapters::common::encode_path_segment(symbol)
    );
    let (body, _): (TickerDetailsResponseDTO, _) = client
        .page(&path, &[("date", as_of)], PathRule::Exact, None)
        .await?;
    if !body
        .results
        .as_ref()
        .and_then(|r| r.ticker.as_deref())
        .is_some_and(|ticker| ticker.eq_ignore_ascii_case(symbol))
    {
        return Err(invalid_page("ticker"));
    }
    let details = details_to_canonical(symbol, body)?;
    validate_delisting(details.delisted_utc.as_deref())?;
    if let Some(list_date) = &details.list_date {
        date(list_date).map_err(|_| invalid_page("list_date"))?;
    }
    Ok(details)
}

#[derive(serde::Deserialize)]
struct TickerEventsDTO {
    results: Option<TickerEventsResultDTO>,
}
#[derive(serde::Deserialize)]
struct TickerEventsResultDTO {
    #[serde(default)]
    events: Vec<TickerEventDTO>,
}
#[derive(serde::Deserialize)]
struct TickerEventDTO {
    #[serde(rename = "type")]
    kind: Option<String>,
    date: Option<String>,
    ticker_change: Option<TickerChangeDTO>,
}
#[derive(serde::Deserialize)]
struct TickerChangeDTO {
    ticker: Option<String>,
}

pub(crate) async fn fetch_ticker_changes(id: &str) -> Result<Vec<TickerChange>> {
    if id.is_empty() || id == "." || id == ".." {
        return Err(FinanceError::InvalidParameter {
            param: "id".into(),
            reason: "a ticker or composite FIGI is required".into(),
        });
    }
    let client = build_client()?;
    let path = format!(
        "/vX/reference/tickers/{}/events",
        crate::adapters::common::encode_path_segment(id)
    );
    let (body, _): (TickerEventsDTO, _) = client
        .page(&path, &[("types", "ticker_change")], PathRule::Exact, None)
        .await?;
    let mut changes = body
        .results
        .ok_or_else(|| invalid_page("results"))?
        .events
        .into_iter()
        .filter(|event| event.kind.as_deref() == Some("ticker_change"))
        .map(|event| {
            let date = event.date.filter(|d| date(d).is_ok());
            let ticker = event
                .ticker_change
                .and_then(|change| change.ticker)
                .filter(|t| !t.is_empty());
            match (date, ticker) {
                (Some(date), Some(ticker)) => Ok(TickerChange { date, ticker }),
                _ => Err(invalid_page("ticker_change")),
            }
        })
        .collect::<Result<Vec<_>>>()?;
    changes.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(changes)
}

pub(crate) async fn fetch_stock_types(locale: &str) -> Result<Vec<StockType>> {
    let client = build_client()?;
    let (body, _): (PaginatedResponseDTO<StockTypeDTO>, _) = client
        .page(
            "/v3/reference/tickers/types",
            &[("asset_class", "stocks"), ("locale", locale)],
            PathRule::Exact,
            None,
        )
        .await?;
    if body.next_url.is_some() {
        return Err(invalid_page("next_url"));
    }
    body.results
        .ok_or_else(|| invalid_page("results"))?
        .into_iter()
        .map(stock_type_to_canonical)
        .collect()
}
