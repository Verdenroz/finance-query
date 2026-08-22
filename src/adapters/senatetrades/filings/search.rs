//! Search-page workflow: accept the disclaimer, filter to Senator PTRs, and
//! read the resulting filer/report rows out of the DataTable.

use chromiumoxide::element::Element;
use chrono::NaiveDate;

use crate::error::Result;

use super::super::client::{SENATE_BASE, SenateTradesClient, wait_for_element, wait_for_elements};
use super::super::models::SenateFilingEntry;

/// Rows per DataTable page once bumped up via its own JS API — the default
/// is 25; the largest built-in option is 100, which keeps every recent
/// filing to one page load instead of walking pagination.
const PAGE_LENGTH: &str = "100";

/// `MM/DD/YYYY` to `NaiveDate`.
pub(super) fn parse_us_date(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%m/%d/%Y").ok()
}

/// Run the search and return every filer/report row found, newest filing
/// first. Order isn't guaranteed by the site (its default sort is
/// alphabetical by filer name), so this always sorts explicitly.
pub(super) async fn search_recent_ptrs(
    client: &SenateTradesClient,
) -> Result<Vec<SenateFilingEntry>> {
    let page = client
        .new_page(&format!("{SENATE_BASE}/search/home/"))
        .await?;

    wait_for_element(&page, "#agree_statement")
        .await?
        .click()
        .await
        .ok();

    wait_for_element(&page, "#filerTypeLabelSenator")
        .await?
        .click()
        .await
        .ok();
    wait_for_element(&page, "#reportTypeLabelPtr")
        .await?
        .click()
        .await
        .ok();
    wait_for_element(&page, "#searchForm button[type=submit]")
        .await?
        .click()
        .await
        .ok();

    wait_for_elements(&page, "#filedReports tbody tr").await?;
    page.evaluate(format!(
        "$('#filedReports').DataTable().page.len({PAGE_LENGTH}).draw();"
    ))
    .await
    .ok();
    let rows = wait_for_elements(&page, "#filedReports tbody tr").await?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(entry) = parse_row(row).await {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|e| std::cmp::Reverse(parse_us_date(&e.filed_date)));
    Ok(entries)
}

async fn parse_row(row: &Element) -> Option<SenateFilingEntry> {
    let cells = row.find_elements("td").await.ok()?;
    let first_name = cells.first()?.inner_text().await.ok()??;
    let last_name = cells.get(1)?.inner_text().await.ok()??;
    let filed_date = cells.get(4)?.inner_text().await.ok()??;
    let link = cells.get(3)?.find_element("a").await.ok()?;
    let path = link.attribute("href").await.ok()??;
    Some(SenateFilingEntry {
        first_name: first_name.trim().to_string(),
        last_name: last_name.trim().to_string(),
        filed_date: filed_date.trim().to_string(),
        path,
    })
}
