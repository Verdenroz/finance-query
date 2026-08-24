//! Index-constituent table scraping.
//!
//! Only `MajorIndex::Sp500`'s Wikipedia article carries an inline
//! constituents table (`table#constituents`, confirmed against the live
//! page) — the Nasdaq-100 and Dow Jones articles list their components only
//! inside a navbox template at the bottom of the page (ticker links grouped
//! by sector, no headquarters/CIK/founding-year columns), a meaningfully
//! different and thinner shape not worth a bespoke parser for. Both stay
//! `NotSupported` here rather than shipping a fragile navbox scrape.
//! Likewise, no Wikipedia article in this set currently carries a
//! constituent-changes history table, so `IndexConstituentChanges` is
//! unrouted for every index.

use crate::error::{FinanceError, Result};
use crate::models::indices::{IndexConstituent, MajorIndex};
use crate::scrapers::html;

const SP500_ARTICLE: &str = "List_of_S%26P_500_companies";

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
}
