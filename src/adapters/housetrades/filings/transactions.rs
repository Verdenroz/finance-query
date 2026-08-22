//! Transaction-table extraction from a PTR's `pdf_extract`-extracted text.
//!
//! Only born-digital filings (typed through fd.house.gov's e-filing system)
//! carry a text layer; older or hand-signed PTRs are scanned images that
//! `pdf_extract` returns as empty text, so those simply produce no rows here
//! rather than being specially detected — OCR is out of scope.
//!
//! `pdf_extract` collapses each transaction to two physical lines regardless
//! of how the description wraps in the rendered PDF:
//! ```text
//! SP Apple Inc. - Common Stock (AAPL)
//! [ST] P 08/13/2026 08/18/2026 $1,001 - $15,000
//! ```
//! or, when the ticker doesn't fit on the description line:
//! ```text
//! GSK plc American Depositary Shares
//! (GSK) [ST] S 07/28/2025 08/11/2025 $1,001 - $15,000
//! ```
//! The second line — asset-type code, transaction type, two dates, amount —
//! is the reliable anchor; the description (and ticker, wherever it landed)
//! comes from the line immediately before it.

use std::sync::LazyLock;

use regex::Regex;

use crate::models::filings::CongressionalTrade;

use super::super::models::PtrIndexEntry;
use super::index::parse_us_date;

/// One row of the PTR's transaction table.
pub(super) struct ParsedTransaction {
    pub symbol: Option<String>,
    pub asset_description: String,
    pub trade_type: Option<String>,
    pub transaction_date: Option<String>,
    pub amount: Option<String>,
}

static ANCHOR_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        ^\s*
        (?:\((?P<ticker>[A-Za-z][A-Za-z.]{0,5})\)\s+)?  # ticker, when it spilled onto this line
        \[[^\]]*\]\s+                                    # asset-type code, e.g. [ST]
        (?P<ttype>[PSE])\s+
        (?P<txdate>\d{1,2}/\d{1,2}/\d{4})\s+
        \d{1,2}/\d{1,2}/\d{4}\s+                         # notification date, unused
        (?P<amount>Over\s+\$[\d,]+|\$[\d,]+(?:\s*-\s*\$[\d,]+)?)
        \s*$
        ",
    )
    .unwrap()
});

static TICKER_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(([A-Za-z][A-Za-z.]{0,5})\)\s*$").unwrap());

static OWNER_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:SP|DC|JT)\s+").unwrap());

/// Extract every transaction row from one PTR's extracted text.
pub(super) fn parse_transactions(text: &str) -> Vec<ParsedTransaction> {
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let caps = ANCHOR_LINE.captures(line)?;
            let desc_line = lines[..i].iter().rev().find(|l| !l.trim().is_empty())?;
            let desc_line = OWNER_PREFIX.replace(desc_line.trim_start(), "");

            let symbol = caps
                .name("ticker")
                .map(|m| m.as_str().to_uppercase())
                .or_else(|| {
                    TICKER_SUFFIX
                        .captures(desc_line.as_ref())
                        .map(|c| c[1].to_uppercase())
                });
            let asset_description = TICKER_SUFFIX
                .replace(desc_line.as_ref(), "")
                .trim()
                .to_string();

            Some(ParsedTransaction {
                symbol,
                asset_description,
                trade_type: map_trade_type(&caps["ttype"]),
                transaction_date: parse_us_date(&caps["txdate"]).map(|d| d.to_string()),
                amount: Some(caps["amount"].to_string()),
            })
        })
        .collect()
}

fn map_trade_type(code: &str) -> Option<String> {
    match code {
        "P" => Some("Purchase".to_string()),
        "S" => Some("Sale".to_string()),
        "E" => Some("Exchange".to_string()),
        _ => None,
    }
}

/// `"AL04"` to `"AL-04"`; passed through unsplit if it isn't 2 letters + digits.
fn format_district(state_dst: &str) -> String {
    let (state, district) = state_dst.split_at(state_dst.len().min(2));
    if district.chars().all(|c| c.is_ascii_digit()) && !district.is_empty() {
        format!("{state}-{district}")
    } else {
        state_dst.to_string()
    }
}

pub(super) fn to_congressional_trade(
    entry: &PtrIndexEntry,
    year: i32,
    tx: ParsedTransaction,
) -> CongressionalTrade {
    CongressionalTrade {
        symbol: tx.symbol,
        first_name: Some(entry.first.clone()),
        last_name: Some(entry.last.clone()),
        office: Some("House".to_string()),
        district: Some(format_district(&entry.state_dst)),
        trade_type: tx.trade_type,
        amount: tx.amount,
        asset_description: (!tx.asset_description.is_empty()).then_some(tx.asset_description),
        transaction_date: tx.transaction_date,
        disclosure_date: parse_us_date(&entry.filing_date).map(|d| d.to_string()),
        link: Some(format!(
            "https://disclosures-clerk.house.gov/public_disc/ptr-pdfs/{year}/{}.pdf",
            entry.doc_id
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verbatim (trimmed to the table) `pdf_extract` output for a real PTR
    /// where the ticker doesn't fit on the description line.
    fn aderholt_gsk_text() -> &'static str {
        "ID Owner Asset Transaction\n\
         Type Date Notification\n\
         Date Amount Cap.\n\
         Gains >\n\
         $200?\n\
         \n\
         GSK plc American Depositary Shares\n\
         (GSK) [ST] S 07/28/2025 08/11/2025 $1,001 - $15,000\n\
         \n\
         F      S     : New\n"
    }

    /// Verbatim `pdf_extract` output for a real PTR with an owner code and
    /// an inline ticker.
    fn ed_case_aapl_text() -> &'static str {
        "ID Owner Asset Transaction\n\
         Type Date Notification\n\
         Date Amount Cap.\n\
         Gains >\n\
         $200?\n\
         \n\
         SP Apple Inc. - Common Stock (AAPL)\n\
         [ST] P 08/13/2026 08/18/2026 $1,001 - $15,000\n\
         \n\
         F      S     : New\n\
         D          : Automatic stock dividend reinvestment.\n"
    }

    #[test]
    fn ticker_that_spills_onto_the_anchor_line_is_found() {
        let rows = parse_transactions(aderholt_gsk_text());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("GSK"));
        assert_eq!(rows[0].trade_type.as_deref(), Some("Sale"));
        assert_eq!(rows[0].transaction_date.as_deref(), Some("2025-07-28"));
        assert_eq!(rows[0].amount.as_deref(), Some("$1,001 - $15,000"));
        assert_eq!(
            rows[0].asset_description,
            "GSK plc American Depositary Shares"
        );
    }

    #[test]
    fn ticker_inline_with_the_description_and_owner_code_is_found() {
        let rows = parse_transactions(ed_case_aapl_text());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(rows[0].trade_type.as_deref(), Some("Purchase"));
        assert_eq!(rows[0].transaction_date.as_deref(), Some("2026-08-13"));
        assert_eq!(rows[0].asset_description, "Apple Inc. - Common Stock");
    }

    #[test]
    fn non_table_text_yields_no_rows() {
        assert!(parse_transactions("Digitally Signed: Hon. Ed Case , 08/18/2026").is_empty());
    }

    #[test]
    fn empty_text_yields_no_rows() {
        assert!(parse_transactions("").is_empty());
    }

    #[test]
    fn district_splits_state_and_number() {
        assert_eq!(format_district("AL04"), "AL-04");
        assert_eq!(format_district("HI01"), "HI-01");
    }

    #[test]
    fn to_congressional_trade_maps_index_and_row_fields() {
        let entry = PtrIndexEntry {
            last: "Case".to_string(),
            first: "Ed".to_string(),
            state_dst: "HI01".to_string(),
            year: 2026,
            filing_date: "8/18/2026".to_string(),
            doc_id: "20035275".to_string(),
        };
        let tx = parse_transactions(ed_case_aapl_text()).remove(0);
        let trade = to_congressional_trade(&entry, 2026, tx);

        assert_eq!(trade.symbol.as_deref(), Some("AAPL"));
        assert_eq!(trade.first_name.as_deref(), Some("Ed"));
        assert_eq!(trade.last_name.as_deref(), Some("Case"));
        assert_eq!(trade.office.as_deref(), Some("House"));
        assert_eq!(trade.district.as_deref(), Some("HI-01"));
        assert_eq!(trade.disclosure_date.as_deref(), Some("2026-08-18"));
        assert_eq!(
            trade.link.as_deref(),
            Some("https://disclosures-clerk.house.gov/public_disc/ptr-pdfs/2026/20035275.pdf")
        );
    }
}
