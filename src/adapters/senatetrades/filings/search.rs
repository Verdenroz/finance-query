//! Search-page workflow: accept the disclaimer, filter to Senator PTRs, and
//! read the resulting filer/report rows out of the DataTable.

use chromiumoxide::element::Element;
use chrono::NaiveDate;

use crate::error::Result;

use super::super::client::{
    SENATE_BASE, SenateTradesClient, find_element_if_present, wait_for_element, wait_for_elements,
};
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
    let mut texts = Vec::with_capacity(cells.len());
    for cell in &cells {
        texts.push(cell.inner_text().await.ok().flatten());
    }
    let href = match cells.get(3) {
        Some(cell) => match cell.find_element("a").await {
            Ok(link) => link.attribute("href").await.ok()?,
            Err(_) => None,
        },
        None => None,
    };
    entry_from_cells(&texts, href)
}

fn entry_from_cells(cells: &[Option<String>], href: Option<String>) -> Option<SenateFilingEntry> {
    let first_name = cells.first()?.as_deref()?;
    let last_name = cells.get(1)?.as_deref()?;
    let filed_date = cells.get(4)?.as_deref()?;
    Some(SenateFilingEntry {
        first_name: first_name.trim().to_string(),
        last_name: last_name.trim().to_string(),
        filed_date: filed_date.trim().to_string(),
        path: href?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(texts: &[&str]) -> Vec<Option<String>> {
        texts.iter().map(|t| Some(t.to_string())).collect()
    }

    #[test]
    fn entry_from_cells_reads_name_date_and_link() {
        let row = cells(&[
            " Jane ",
            "Doe",
            "Doe, Jane (Senator)",
            "Report",
            " 03/14/2025 ",
        ]);
        let entry = entry_from_cells(&row, Some("/search/view/ptr/abc/".into())).unwrap();
        assert_eq!(entry.first_name, "Jane");
        assert_eq!(entry.last_name, "Doe");
        assert_eq!(entry.filed_date, "03/14/2025");
        assert_eq!(entry.path, "/search/view/ptr/abc/");
    }

    #[test]
    fn entry_from_cells_ignores_unused_cell_without_text() {
        let mut row = cells(&["Jane", "Doe", "x", "y", "03/14/2025"]);
        row[2] = None;
        assert!(entry_from_cells(&row, Some("/x/".into())).is_some());
    }

    #[test]
    fn entry_from_cells_rejects_short_row() {
        let row = cells(&["Jane", "Doe", "Doe, Jane"]);
        assert!(entry_from_cells(&row, Some("/x/".into())).is_none());
    }

    #[test]
    fn entry_from_cells_rejects_missing_link() {
        let row = cells(&["Jane", "Doe", "x", "y", "03/14/2025"]);
        assert!(entry_from_cells(&row, None).is_none());
    }

    #[test]
    fn entry_from_cells_rejects_empty_cell_text() {
        let mut row = cells(&["Jane", "Doe", "x", "y", "03/14/2025"]);
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
