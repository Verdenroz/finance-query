//! Report-page transaction table extraction.
//!
//! A born-digital filing renders `table.table-striped` with columns `#`,
//! Transaction Date, Owner, Ticker, Asset Name, Asset Type, Type, Amount,
//! Comment. Older/scanned ("paper") filings render no such table; those
//! produce no rows here rather than being specially detected, the same
//! boundary the House adapter draws around non-text-layer PDFs.

use chromiumoxide::element::Element;

use crate::models::filings::CongressionalTrade;

use super::super::client::{SENATE_BASE, SenateTradesClient, wait_for_elements};
use super::super::models::SenateFilingEntry;
use super::search::parse_us_date;

struct ParsedTransaction {
    symbol: Option<String>,
    asset_description: Option<String>,
    trade_type: Option<String>,
    transaction_date: Option<String>,
    amount: Option<String>,
}

/// Open one filing's report page and return every transaction matching
/// `symbol`. Any failure along the way (navigation, missing table, a paper
/// filing) resolves to an empty list rather than an error.
pub(super) async fn matching_transactions(
    client: &SenateTradesClient,
    entry: SenateFilingEntry,
    symbol: &str,
) -> Vec<CongressionalTrade> {
    let Ok(page) = client
        .new_page(&format!("{SENATE_BASE}{}", entry.path))
        .await
    else {
        return Vec::new();
    };
    let Ok(rows) = wait_for_elements(&page, "table.table-striped tbody tr").await else {
        page.close().await.ok();
        return Vec::new();
    };

    let mut trades = Vec::new();
    for row in &rows {
        if let Some(tx) = parse_row(row).await
            && tx
                .symbol
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case(symbol))
        {
            trades.push(to_congressional_trade(&entry, tx));
        }
    }
    page.close().await.ok();
    trades
}

async fn parse_row(row: &Element) -> Option<ParsedTransaction> {
    let cells = row.find_elements("td").await.ok()?;
    let transaction_date = cells.get(1)?.inner_text().await.ok()??;
    let ticker = cells.get(3)?.inner_text().await.ok()?.unwrap_or_default();
    let asset_name = cells.get(4)?.inner_text().await.ok()??;
    let trade_type = cells.get(6)?.inner_text().await.ok()??;
    let amount = cells.get(7)?.inner_text().await.ok()??;

    let ticker = ticker.trim();
    Some(ParsedTransaction {
        symbol: (!ticker.is_empty() && ticker != "--").then(|| ticker.to_uppercase()),
        asset_description: normalize_whitespace(&asset_name),
        trade_type: normalize_whitespace(&trade_type),
        transaction_date: parse_us_date(&transaction_date).map(|d| d.to_string()),
        amount: normalize_whitespace(&amount),
    })
}

fn normalize_whitespace(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}

fn to_congressional_trade(entry: &SenateFilingEntry, tx: ParsedTransaction) -> CongressionalTrade {
    CongressionalTrade {
        symbol: tx.symbol,
        first_name: Some(entry.first_name.clone()),
        last_name: Some(entry.last_name.clone()),
        office: Some("Senate".to_string()),
        district: None,
        trade_type: tx.trade_type,
        amount: tx.amount,
        asset_description: tx.asset_description,
        transaction_date: tx.transaction_date,
        disclosure_date: parse_us_date(&entry.filed_date).map(|d| d.to_string()),
        link: Some(format!("{SENATE_BASE}{}", entry.path)),
    }
}
