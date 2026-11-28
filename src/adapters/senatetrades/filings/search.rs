//! Search-page workflow: accept the disclaimer, filter to Senator PTRs, and
//! read the resulting filer/report rows out of the DataTable.

use chromiumoxide::element::Element;
use chrono::NaiveDate;

use crate::error::Result;

use super::super::client::{
    SENATE_BASE, SenateTradesClient, find_element_if_present, wait_for_element, wait_for_elements,
};
use super::super::models::SenateFilingEntry;
use super::cell_text;

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

    if let Some(checkbox) = find_element_if_present(&page, "#agree_statement").await {
        checkbox.click().await.ok();
    }

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
    let texts = [
        cell_text(&cells, 0).await?,
        cell_text(&cells, 1).await?,
        cell_text(&cells, 4).await?,
    ];
    if texts.iter().any(Option::is_none) {
        return None;
    }
    let link = cells.get(3)?.find_element("a").await.ok()?;
    let href = link.attribute("href").await.ok()?;
    entry_from_cells(&texts, href)
}

/// `cells` holds the first name, last name and filed date, in that order.
fn entry_from_cells(
    cells: &[Option<String>; 3],
    href: Option<String>,
) -> Option<SenateFilingEntry> {
    let [first_name, last_name, filed_date] = cells;
    Some(SenateFilingEntry {
        first_name: first_name.as_deref()?.trim().to_string(),
        last_name: last_name.as_deref()?.trim().to_string(),
        filed_date: filed_date.as_deref()?.trim().to_string(),
        path: href?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::cells;

    #[test]
    fn entry_from_cells_reads_name_date_and_link() {
        let row = cells([" Jane ", "Doe", " 03/14/2025 "]);
        let entry = entry_from_cells(&row, Some("/search/view/ptr/abc/".into())).unwrap();
        assert_eq!(entry.first_name, "Jane");
        assert_eq!(entry.last_name, "Doe");
        assert_eq!(entry.filed_date, "03/14/2025");
        assert_eq!(entry.path, "/search/view/ptr/abc/");
    }

    #[test]
    fn entry_from_cells_rejects_missing_link() {
        assert!(entry_from_cells(&cells(["Jane", "Doe", "03/14/2025"]), None).is_none());
    }

    #[test]
    fn entry_from_cells_rejects_empty_cell_text() {
        let mut row = cells(["Jane", "Doe", "03/14/2025"]);
        row[1] = None;
        assert!(entry_from_cells(&row, Some("/x/".into())).is_none());
    }

    #[test]
    fn parses_us_date() {
        assert_eq!(
            parse_us_date("03/14/2025"),
            NaiveDate::from_ymd_opt(2025, 3, 14)
        );
    }

    #[test]
    fn parse_us_date_trims_whitespace() {
        assert_eq!(
            parse_us_date("  12/01/2024\n"),
            NaiveDate::from_ymd_opt(2024, 12, 1)
        );
    }

    #[test]
    fn parse_us_date_rejects_non_us_formats() {
        assert_eq!(parse_us_date("2025-03-14"), None);
        assert_eq!(parse_us_date("14/03/2025"), None);
        assert_eq!(parse_us_date(""), None);
    }
}
