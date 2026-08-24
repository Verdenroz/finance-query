//! Executive compensation extracted from DEF 14A proxy statements' Summary
//! Compensation Table (Item 402(c) of Regulation S-K) — a plain HTML table,
//! not XBRL-tagged, whose exact column set varies by filer (e.g. Apple
//! omits Bonus/Option Awards entirely; some filers add a pension-value
//! column this parser doesn't model). Best-effort: unrecognized columns are
//! ignored rather than failing the row.

use std::sync::LazyLock;

use regex::Regex;

use crate::adapters::edgar::{build_client, submissions_for_symbol};
use crate::error::{FinanceError, Result};
use crate::models::corporate::governance::ExecutiveCompensation;
use crate::scrapers::html;

/// Block-level boundaries produce no text-flow separator when tags are
/// stripped, so `Name<br/>Position` or `<div>Stock</div><div>Awards</div>`
/// would otherwise read as one run-on word. Normalizing to a space before
/// extraction fixes every cell at once, for every filer's line-wrap style.
static BLOCK_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)</?(?:br|div|p)(?:\s[^>]*)?/?>").unwrap());

/// A cell holding only footnote markers (e.g. `(3)(4)`) attached to the
/// figure in an adjacent cell — filtered out like the other empty spacer
/// cells these tables pad column widths with.
static FOOTNOTE_MARKER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\(\d{1,3}\))+$").unwrap());

#[derive(Debug, Clone, Copy, PartialEq)]
enum Column {
    Name,
    Year,
    Salary,
    Bonus,
    StockAward,
    OptionAward,
    IncentivePlan,
    OtherComp,
    Total,
    Unknown,
}

fn classify_header(text: &str) -> Column {
    // Stacked block-boundary replacements can leave runs of whitespace
    // (e.g. "Stock  Awards"), which would break a multi-word match.
    let t = text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if t.contains("name") {
        Column::Name
    } else if t.contains("year") {
        Column::Year
    } else if t.contains("salary") {
        Column::Salary
    } else if t.contains("bonus") {
        Column::Bonus
    } else if t.contains("stock award") {
        Column::StockAward
    } else if t.contains("option award") {
        Column::OptionAward
    } else if t.contains("incentive") {
        Column::IncentivePlan
    } else if t.contains("other") && t.contains("compensation") {
        Column::OtherComp
    } else if t.contains("total") {
        Column::Total
    } else {
        Column::Unknown
    }
}

fn parse_dollar(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    (!cleaned.is_empty())
        .then(|| cleaned.parse().ok())
        .flatten()
}

/// Non-empty, non-footnote-marker cell text for one `<tr>` — the same
/// "skip the invisible spacer columns" filtering these SEC tables need for
/// both header and data rows.
fn row_cells(row_html: &str) -> Vec<String> {
    html::find_all(row_html, "td")
        .iter()
        .map(|c| c.text().trim().to_string())
        .filter(|t| !t.is_empty() && !FOOTNOTE_MARKER.is_match(t))
        .collect()
}

/// Index and cells of the first row carrying real content. Some filers open
/// a table with a `<tr>` of self-closing, content-less `<td/>` cells that
/// exist only to hint column widths via CSS — that row precedes the real
/// header and must be skipped, not read as it.
fn first_nonempty_row(rows: &[html::Element<'_>]) -> Option<(usize, Vec<String>)> {
    rows.iter().enumerate().find_map(|(i, r)| {
        let cells = row_cells(r.inner);
        (!cells.is_empty()).then_some((i, cells))
    })
}

/// Find the Summary Compensation Table by header shape rather than by
/// searching for the heading text — the heading phrase also appears in the
/// table of contents and cross-references, none of which precede a table
/// with this column set.
fn find_summary_compensation_table<'a>(page: &'a str) -> Option<html::Element<'a>> {
    html::find_all(page, "table").into_iter().find(|t| {
        let rows = html::find_all(t.inner, "tr");
        let Some((_, header)) = first_nonempty_row(&rows) else {
            return false;
        };
        let columns: Vec<Column> = header.iter().map(|h| classify_header(h)).collect();
        columns.contains(&Column::Name)
            && columns.contains(&Column::Year)
            && columns.contains(&Column::Salary)
    })
}

fn parse_summary_compensation_table(page: &str) -> Result<Vec<ExecutiveCompensation>> {
    let normalized = BLOCK_BREAK.replace_all(page, " ");

    let table = find_summary_compensation_table(&normalized).ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "table".to_string(),
            context: "no Summary Compensation Table found in the DEF 14A filing".to_string(),
        }
    })?;

    let rows = html::find_all(table.inner, "tr");
    let Some((header_index, header_cells)) = first_nonempty_row(&rows) else {
        return Err(FinanceError::ResponseStructureError {
            field: "table.rows".to_string(),
            context: "Summary Compensation Table had no header row".to_string(),
        });
    };
    let columns: Vec<Column> = header_cells.iter().map(|h| classify_header(h)).collect();

    let mut out = Vec::new();
    let mut current_name: Option<String> = None;

    for row in rows.iter().skip(header_index + 1) {
        let cells = row_cells(row.inner);
        if cells.is_empty() {
            continue;
        }

        let offset = columns.len().saturating_sub(cells.len());
        if offset > 1 || cells.len() > columns.len() {
            continue;
        }
        if offset == 0 {
            current_name = Some(cells[0].clone());
        }

        let mut comp = ExecutiveCompensation {
            name_and_position: current_name.clone(),
            ..Default::default()
        };
        for (col, cell) in columns[offset..].iter().zip(cells.iter()) {
            match col {
                Column::Year => comp.year = cell.parse().ok(),
                Column::Salary => comp.salary = parse_dollar(cell),
                Column::Bonus => comp.bonus = parse_dollar(cell),
                Column::StockAward => comp.stock_award = parse_dollar(cell),
                Column::OptionAward => comp.option_award = parse_dollar(cell),
                Column::IncentivePlan => comp.incentive_plan_compensation = parse_dollar(cell),
                Column::OtherComp => comp.other_compensation = parse_dollar(cell),
                Column::Total => comp.total = parse_dollar(cell),
                Column::Name | Column::Unknown => {}
            }
        }

        if comp.year.is_some() {
            out.push(comp);
        }
    }

    if out.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "table.rows".to_string(),
            context: "Summary Compensation Table had no parseable data rows".to_string(),
        });
    }
    Ok(out)
}

/// Fetch canonical executive compensation from the most recent DEF 14A,
/// most recent fiscal year first.
pub async fn fetch_executive_compensation_response(
    symbol: &str,
) -> Result<Vec<ExecutiveCompensation>> {
    let subs = submissions_for_symbol(symbol).await?;
    let cik = subs.cik.clone().unwrap_or_default();
    let company_name = subs.name.clone();
    let filing = subs
        .filings
        .and_then(|f| f.recent)
        .map(|r| r.to_filings())
        .unwrap_or_default()
        .into_iter()
        .find(|f| f.form == "DEF 14A" && !f.primary_document.is_empty());

    let Some(filing) = filing else {
        return Err(FinanceError::ResponseStructureError {
            field: "filings".to_string(),
            context: format!("no DEF 14A filing found for {symbol}"),
        });
    };

    // Uses the registrant's own CIK, not the accession number's filer-agent
    // prefix — proxy statements are often filed through a third-party agent.
    let accession_no_dashes = filing.accession_number.replace('-', "");
    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{accession_no_dashes}/{}",
        cik.trim_start_matches('0'),
        filing.primary_document
    );
    let client = build_client()?;
    let bytes = client.get_document(&url).await?;
    let page = String::from_utf8_lossy(&bytes);

    let mut rows = parse_summary_compensation_table(&page)?;
    for row in &mut rows {
        row.symbol = Some(symbol.to_string());
        row.cik = Some(cik.clone());
        row.company_name = company_name.clone();
        row.filing_date = Some(filing.filing_date.clone());
        row.url = Some(url.clone());
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.year));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_page(rows: &str) -> String {
        format!(
            r#"<html><body>
            <p>See "Summary Compensation Table" below.</p>
            <table><tr><td>Summary Compensation Table</td><td>12</td></tr></table>
            <table>
            <tr>
            <td><b>Name and Principal Position</b></td>
            <td>&#160;&#160;</td>
            <td><b>Year</b></td>
            <td>&#160;&#160;</td>
            <td><b>Salary</b><br/><b>($)</b></td>
            <td>&#160;&#160;</td>
            <td><b>Stock Awards</b><br/><b>($)</b></td>
            <td>&#160;&#160;</td>
            <td><b>Non-Equity Incentive Plan Compensation</b><br/><b>($)</b></td>
            <td>&#160;&#160;</td>
            <td><b>All Other Compensation</b><br/><b>($)</b></td>
            <td>&#160;&#160;</td>
            <td><b>Total</b><br/><b>($)</b></td>
            </tr>
            {rows}
            </table>
            </body></html>"#
        )
    }

    #[test]
    fn parses_a_full_row_with_name_and_a_continuation_row_without_one() {
        let page = table_page(
            r#"<tr>
            <td rowspan="2">Tim&#160;Cook<br/>Chief Executive Officer</td>
            <td>&#160;</td><td>2025</td><td>&#160;</td><td>3,000,000</td><td>&#160;</td>
            <td>57,535,293</td><td>&#160;</td><td>12,000,000</td><td>&#160;</td>
            <td>1,759,518</td><td>(3)(4)</td><td>74,294,811</td>
            </tr>
            <tr>
            <td>2024</td><td>&#160;</td><td>3,000,000</td><td>&#160;</td>
            <td>58,088,946</td><td>&#160;</td><td>12,000,000</td><td>&#160;</td>
            <td>1,520,856</td><td>&#160;</td><td>74,609,802</td>
            </tr>"#,
        );

        let rows = parse_summary_compensation_table(&page).unwrap();
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(
            first.name_and_position.as_deref(),
            Some("Tim\u{a0}Cook Chief Executive Officer")
        );
        assert_eq!(first.year, Some(2025));
        assert_eq!(first.salary, Some(3_000_000.0));
        assert_eq!(first.stock_award, Some(57_535_293.0));
        assert_eq!(first.incentive_plan_compensation, Some(12_000_000.0));
        assert_eq!(first.other_compensation, Some(1_759_518.0));
        assert_eq!(first.total, Some(74_294_811.0));
        assert_eq!(first.bonus, None);
        assert_eq!(first.option_award, None);

        let second = &rows[1];
        assert_eq!(
            second.name_and_position.as_deref(),
            Some("Tim\u{a0}Cook Chief Executive Officer"),
            "continuation row without its own name cell inherits the prior row's name"
        );
        assert_eq!(second.year, Some(2024));
        assert_eq!(second.total, Some(74_609_802.0));
    }

    #[test]
    fn a_page_with_no_matching_table_errors() {
        let page = "<html><body><table><tr><td>unrelated</td></tr></table></body></html>";
        let err = parse_summary_compensation_table(page).unwrap_err();
        assert!(matches!(err, FinanceError::ResponseStructureError { .. }));
    }

    #[test]
    fn extra_columns_this_parser_does_not_model_are_ignored() {
        let page = r#"<html><body>
            <table>
            <tr>
            <td>Name &amp; Principal Position</td><td>Year</td><td>Salary($)</td>
            <td>Change in Pension Value and Non-Qualified Deferred Compensation Earnings</td>
            <td>All Other Compensation($)</td><td>Total ($)</td>
            </tr>
            <tr>
            <td>Jane Doe, CEO</td><td>2025</td><td>1,000,000</td><td>500,000</td>
            <td>10,000</td><td>1,510,000</td>
            </tr>
            </table>
            </body></html>"#;

        let rows = parse_summary_compensation_table(page).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].salary, Some(1_000_000.0));
        assert_eq!(rows[0].other_compensation, Some(10_000.0));
        assert_eq!(rows[0].total, Some(1_510_000.0));
    }

    #[test]
    fn a_leading_column_width_hint_row_is_skipped_not_read_as_the_header() {
        let page = r#"<html><body>
            <table>
            <tr><td style="width:1%"/><td style="width:20%"/><td style="width:10%"/><td style="width:10%"/></tr>
            <tr>
            <td>Name &amp; Principal Position</td><td>Year</td><td>Salary($)</td><td>Total ($)</td>
            </tr>
            <tr>
            <td>Jane Doe, CEO</td><td>2025</td><td>1,000,000</td><td>1,200,000</td>
            </tr>
            </table>
            </body></html>"#;

        let rows = parse_summary_compensation_table(page).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name_and_position.as_deref(), Some("Jane Doe, CEO"));
        assert_eq!(rows[0].salary, Some(1_000_000.0));
        assert_eq!(rows[0].total, Some(1_200_000.0));
    }

    #[test]
    fn a_header_split_across_block_elements_with_irregular_spacing_still_classifies() {
        let page = r#"<html><body>
            <table>
            <tr>
            <td>Name</td><td>Year</td><td>Salary($)</td>
            <td><div>Stock</div>  <div>Awards</div>  ($)</td>
            <td>Total ($)</td>
            </tr>
            <tr>
            <td>Jane Doe, CEO</td><td>2025</td><td>1,000,000</td><td>250,000</td><td>1,250,000</td>
            </tr>
            </table>
            </body></html>"#;

        let rows = parse_summary_compensation_table(page).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].stock_award, Some(250_000.0));
    }

    #[test]
    fn parse_dollar_ignores_commas_and_dashes() {
        assert_eq!(parse_dollar("3,000,000"), Some(3_000_000.0));
        assert_eq!(parse_dollar("\u{2014}"), None);
        assert_eq!(parse_dollar(""), None);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_apple_compensation_from_live_edgar() {
        let rows = fetch_executive_compensation_response("AAPL").await.unwrap();
        assert!(
            rows.len() >= 3,
            "expected multiple NEOs, got {}",
            rows.len()
        );

        let cook_2025 = rows
            .iter()
            .find(|r| {
                r.year == Some(2025)
                    && r.name_and_position
                        .as_deref()
                        .is_some_and(|n| n.contains("Cook"))
            })
            .expect("Tim Cook FY2025 row");
        assert_eq!(cook_2025.salary, Some(3_000_000.0));
        assert_eq!(cook_2025.stock_award, Some(57_535_293.0));
        assert_eq!(cook_2025.incentive_plan_compensation, Some(12_000_000.0));
        assert_eq!(cook_2025.other_compensation, Some(1_759_518.0));
        assert_eq!(cook_2025.total, Some(74_294_811.0));
        assert_eq!(cook_2025.cik.as_deref(), Some("0000320193"));

        let cook_2024 = rows
            .iter()
            .find(|r| {
                r.year == Some(2024)
                    && r.name_and_position
                        .as_deref()
                        .is_some_and(|n| n.contains("Cook"))
            })
            .expect("Tim Cook FY2024 continuation row");
        assert_eq!(cook_2024.total, Some(74_609_802.0));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_huntington_ingalls_compensation_from_live_edgar() {
        let rows = fetch_executive_compensation_response("HII").await.unwrap();
        assert!(!rows.is_empty());

        let kastner_2025 = rows
            .iter()
            .find(|r| {
                r.year == Some(2025)
                    && r.name_and_position
                        .as_deref()
                        .is_some_and(|n| n.contains("Kastner"))
            })
            .expect("Kastner FY2025 row");
        assert_eq!(kastner_2025.salary, Some(1_303_745.0));
        assert_eq!(kastner_2025.stock_award, Some(8_499_752.0));
        assert_eq!(kastner_2025.incentive_plan_compensation, Some(2_769_000.0));
        assert_eq!(kastner_2025.other_compensation, Some(170_482.0));
        assert_eq!(kastner_2025.total, Some(13_809_406.0));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_microsoft_compensation_from_live_edgar() {
        let rows = fetch_executive_compensation_response("MSFT").await.unwrap();
        assert!(!rows.is_empty());

        let nadella_2025 = rows
            .iter()
            .find(|r| {
                r.year == Some(2025)
                    && r.name_and_position
                        .as_deref()
                        .is_some_and(|n| n.contains("Nadella"))
            })
            .expect("Nadella FY2025 row");
        assert_eq!(nadella_2025.salary, Some(2_500_000.0));
        assert_eq!(nadella_2025.bonus, Some(0.0));
        assert_eq!(nadella_2025.stock_award, Some(84_245_496.0));
        assert_eq!(nadella_2025.incentive_plan_compensation, Some(9_555_000.0));
        assert_eq!(nadella_2025.other_compensation, Some(196_294.0));
        assert_eq!(nadella_2025.total, Some(96_496_790.0));
    }
}
