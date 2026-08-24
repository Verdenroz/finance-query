//! Index-constituent table scraping.
//!
//! Only `MajorIndex::Sp500`'s Wikipedia article carries an inline
//! constituents table (`table#constituents`, confirmed against the live
//! page) — the Nasdaq-100 and Dow Jones articles list their components only
//! inside a navbox template at the bottom of the page (ticker links grouped
//! by sector, no headquarters/CIK/founding-year columns), a meaningfully
//! different and thinner shape not worth a bespoke parser for. Both stay
//! `NotSupported` here rather than shipping a fragile navbox scrape.
//! Constituent changes are similarly S&P-500-only, scraped from a separate
//! article ("Historical components of the S&P 500") whose `table#changes`
//! tracks additions/removals.

use crate::error::{FinanceError, Result};
use crate::models::indices::{IndexConstituent, IndexConstituentChange, MajorIndex};
use crate::scrapers::html;

const SP500_ARTICLE: &str = "List_of_S%26P_500_companies";
const SP500_CHANGES_ARTICLE: &str = "Historical_components_of_the_S%26P_500";

pub(crate) async fn fetch_index_constituents_response(
    index: MajorIndex,
) -> Result<Vec<IndexConstituent>> {
    match index {
        MajorIndex::Sp500 => {
            let page = super::client()?.page_html(SP500_ARTICLE).await?;
            parse_sp500_constituents(&page)
        }
        MajorIndex::Nasdaq100 | MajorIndex::DowJones => {
            Err(crate::providers::Operation::IndexConstituents
                .not_supported(crate::Provider::Wikipedia))
        }
    }
}

pub(crate) async fn fetch_index_constituent_changes_response(
    index: MajorIndex,
) -> Result<Vec<IndexConstituentChange>> {
    match index {
        MajorIndex::Sp500 => {
            let page = super::client()?.page_html(SP500_CHANGES_ARTICLE).await?;
            parse_sp500_changes(&page)
        }
        MajorIndex::Nasdaq100 | MajorIndex::DowJones => {
            Err(crate::providers::Operation::IndexConstituentChanges
                .not_supported(crate::Provider::Wikipedia))
        }
    }
}

fn parse_sp500_constituents(page: &str) -> Result<Vec<IndexConstituent>> {
    let table =
        html::find_first(page, "table").ok_or_else(|| FinanceError::ResponseStructureError {
            field: "table".to_string(),
            context: "no constituents table found on the S&P 500 Wikipedia page".to_string(),
        })?;

    let constituents: Vec<IndexConstituent> = html::find_all(table.inner, "tr")
        .into_iter()
        .filter_map(|row| {
            let cells = html::find_all(row.inner, "td");
            (cells.len() >= 8).then(|| {
                let text: Vec<String> = cells.iter().map(|c| c.text().trim().to_string()).collect();
                IndexConstituent {
                    symbol: text[0].clone(),
                    name: non_empty(&text[1]),
                    sector: non_empty(&text[2]),
                    sub_sector: non_empty(&text[3]),
                    headquarters: non_empty(&text[4]),
                    date_first_added: non_empty(&text[5]),
                    cik: non_empty(&text[6]),
                    founded: non_empty(&text[7]),
                }
            })
        })
        .collect();

    if constituents.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "table.rows".to_string(),
            context: "S&P 500 constituents table had no data rows".to_string(),
        });
    }
    Ok(constituents)
}

fn parse_sp500_changes(page: &str) -> Result<Vec<IndexConstituentChange>> {
    let table = html::find_all(page, "table")
        .into_iter()
        .find(|t| t.attr("id").as_deref() == Some("changes"))
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "table#changes".to_string(),
            context: "no constituent-changes table found on the historical S&P 500 Wikipedia page"
                .to_string(),
        })?;

    let changes: Vec<IndexConstituentChange> = html::find_all(table.inner, "tr")
        .into_iter()
        .filter_map(|row| {
            let cells = html::find_all(row.inner, "td");
            (cells.len() >= 6).then(|| {
                let text: Vec<String> = cells.iter().map(|c| c.text().trim().to_string()).collect();
                IndexConstituentChange {
                    date: parse_wiki_date(&text[0]),
                    symbol: non_empty(&text[1]),
                    added_security: non_empty(&text[2]),
                    removed_ticker: non_empty(&text[3]),
                    removed_security: non_empty(&text[4]),
                    reason: non_empty(&text[5]),
                }
            })
        })
        .collect();

    if changes.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "table#changes.rows".to_string(),
            context: "S&P 500 changes table had no data rows".to_string(),
        });
    }
    Ok(changes)
}

/// Parse a `"Month D, YYYY"` date (e.g. `"August 5, 2026"`) into `YYYY-MM-DD`.
fn parse_wiki_date(s: &str) -> Option<String> {
    let (month_day, year) = s.trim().rsplit_once(',')?;
    let (month, day) = month_day.trim().split_once(' ')?;
    let month_num = match month {
        "January" => "01",
        "February" => "02",
        "March" => "03",
        "April" => "04",
        "May" => "05",
        "June" => "06",
        "July" => "07",
        "August" => "08",
        "September" => "09",
        "October" => "10",
        "November" => "11",
        "December" => "12",
        _ => return None,
    };
    Some(format!("{}-{month_num}-{:0>2}", year.trim(), day.trim()))
}

fn non_empty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_page() -> String {
        format!(
            r#"<html><body>
            <table class="wikitable sortable" id="constituents">
            <tbody><tr>
            <th>Symbol</th><th>Security</th><th>GICS Sector</th><th>GICS Sub-Industry</th>
            <th>Headquarters Location</th><th>Date added</th><th>CIK</th><th>Founded</th>
            </tr>
            <tr>
            <td><a href="/wiki/3M">MMM</a></td>
            <td><a href="/wiki/3M">3M</a></td>
            <td>Industrials</td>
            <td>Industrial Conglomerates</td>
            <td><a href="/wiki/Saint_Paul">Saint Paul, Minnesota</a></td>
            <td>1957-03-04</td>
            <td>0000066740</td>
            <td>1902</td>
            </tr>
            </tbody></table>
            {}
            </body></html>"#,
            "<table><tr><td>unrelated</td></tr></table>"
        )
    }

    #[test]
    fn constituents_table_parses_into_rows_skipping_the_header() {
        let constituents = parse_sp500_constituents(&fixture_page()).unwrap();
        assert_eq!(constituents.len(), 1);
        let mmm = &constituents[0];
        assert_eq!(mmm.symbol, "MMM");
        assert_eq!(mmm.name.as_deref(), Some("3M"));
        assert_eq!(mmm.sector.as_deref(), Some("Industrials"));
        assert_eq!(mmm.sub_sector.as_deref(), Some("Industrial Conglomerates"));
        assert_eq!(mmm.headquarters.as_deref(), Some("Saint Paul, Minnesota"));
        assert_eq!(mmm.date_first_added.as_deref(), Some("1957-03-04"));
        assert_eq!(mmm.cik.as_deref(), Some("0000066740"));
        assert_eq!(mmm.founded.as_deref(), Some("1902"));
    }

    #[test]
    fn a_page_with_no_table_errors() {
        let err = parse_sp500_constituents("<html><body>no tables here</body></html>").unwrap_err();
        assert!(matches!(err, FinanceError::ResponseStructureError { .. }));
    }

    fn changes_fixture_page(row: &str) -> String {
        format!(
            r#"<html><body>
            <table><tr><td>unrelated</td></tr></table>
            <table class="wikitable sortable" id="changes">
            <tbody><tr>
            <th rowspan="2">Effective Date</th><th colspan="2">Added</th><th colspan="2">Removed</th>
            <th rowspan="2">Reason</th><th rowspan="2">Refs</th></tr>
            <tr><th>Ticker</th><th>Security</th><th>Ticker</th><th>Security</th></tr>
            {row}
            </tbody></table>
            </body></html>"#
        )
    }

    #[test]
    fn changes_table_parses_a_full_row() {
        let page = changes_fixture_page(
            r#"<tr>
            <td>August 18, 2026</td>
            <td>RDDT</td>
            <td><a href="/wiki/Reddit">Reddit</a></td>
            <td>AVB</td>
            <td><a href="/wiki/AvalonBay">AvalonBay Communities</a></td>
            <td>Equity Residential acquired AvalonBay Communities.</td>
            <td>[2]</td>
            </tr>"#,
        );

        let changes = parse_sp500_changes(&page).unwrap();
        assert_eq!(changes.len(), 1);
        let change = &changes[0];
        assert_eq!(change.date.as_deref(), Some("2026-08-18"));
        assert_eq!(change.symbol.as_deref(), Some("RDDT"));
        assert_eq!(change.added_security.as_deref(), Some("Reddit"));
        assert_eq!(change.removed_ticker.as_deref(), Some("AVB"));
        assert_eq!(
            change.removed_security.as_deref(),
            Some("AvalonBay Communities")
        );
        assert_eq!(
            change.reason.as_deref(),
            Some("Equity Residential acquired AvalonBay Communities.")
        );
    }

    #[test]
    fn changes_table_row_without_refs_column_still_parses() {
        let page = changes_fixture_page(
            r#"<tr>
            <td>March 23, 2015</td>
            <td>SLG</td>
            <td>SL Green Realty</td>
            <td>NBR</td>
            <td>Nabors Industries</td>
            <td>Index rebalancing.</td>
            </tr>"#,
        );

        let changes = parse_sp500_changes(&page).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].date.as_deref(), Some("2015-03-23"));
    }

    #[test]
    fn a_changes_page_with_no_changes_table_errors() {
        let err =
            parse_sp500_changes("<html><body><table><tr><td>x</td></tr></table></body></html>")
                .unwrap_err();
        assert!(matches!(err, FinanceError::ResponseStructureError { .. }));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_changes_from_the_live_wikipedia_article() {
        let changes = fetch_index_constituent_changes_response(MajorIndex::Sp500)
            .await
            .unwrap();
        assert!(changes.len() > 100);
        assert!(changes[0].date.as_deref().unwrap() >= "2015-01-01");
    }

    #[test]
    fn wiki_dates_parse_single_and_double_digit_days() {
        assert_eq!(
            parse_wiki_date("August 5, 2026"),
            Some("2026-08-05".to_string())
        );
        assert_eq!(
            parse_wiki_date("August 18, 2026"),
            Some("2026-08-18".to_string())
        );
    }
}
