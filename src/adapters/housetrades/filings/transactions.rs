//! Transaction-table extraction from a PTR's extracted text lines.
//!
//! Rows keep the layout of the rendered table, so one transaction is the line
//! carrying the transaction type, both dates, and the amount, plus at most one
//! continuation line when the asset description or the amount wrapped:
//! ```text
//! SP 3M Company (MMM) [ST] S 05/14/2020 05/20/2020 $15,001 -
//! $50,000
//! ```
//! ```text
//! Treasury Bill (3-month, matures P 08/18/2025 08/18/2025 $15,001 -
//! 11/20/2025) [GS] $50,000
//! ```
//! The type-and-dates run is the anchor because it never wraps. Everything
//! left of it on the same line is the description; the asset-type code sits
//! either there or on the continuation line.

use std::sync::LazyLock;

use regex::Regex;

use crate::models::filings::CongressionalTrade;

use super::super::models::PtrIndexEntry;
use super::index::parse_us_date;

/// One row of the PTR's transaction table.
pub(crate) struct ParsedTransaction {
    pub symbol: Option<String>,
    pub asset_description: String,
    pub trade_type: Option<String>,
    pub transaction_date: Option<String>,
    pub amount: Option<String>,
}

static ANCHOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        (?:^|\s)
        (?P<ttype>[PSE])
        (?:\s*\([^)]*\))?                               # qualifier such as (partial)
        \s+ (?P<txdate>\d{1,2}/\d{1,2}/\d{4})
        \s+ \d{1,2}/\d{1,2}/\d{4}                       # notification date, unused
        \s+ (?P<amount>(?:Over\s+)?\$[\d,]+(?:\s*-\s*(?:\$[\d,]+)?)?)
        \s*$
        ",
    )
    .unwrap()
});

static ASSET_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[[A-Za-z]{1,3}\]").unwrap());

static TICKER_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(([A-Za-z][A-Za-z.]{0,5})\)\s*$").unwrap());

static OWNER_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:\d{6,}\s+)?(?:SP|DC|JT)\s+").unwrap());

static LEADING_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{6,}\s+").unwrap());

/// Extract every transaction row from one PTR's extracted text.
pub(super) fn parse_transactions(lines: &[String]) -> Vec<ParsedTransaction> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let caps = ANCHOR.captures(line)?;
            let head = &line[..caps.get(0).map(|m| m.start())?];
            let next = lines.get(i + 1).map(String::as_str).unwrap_or("");
            Some(build(head, &caps, next))
        })
        .collect()
}

/// Split a continuation line around the asset-type code into the trailing
/// description and the trailing amount.
fn split_continuation(next: &str) -> (Option<&str>, &str, Option<&str>) {
    match ASSET_CODE.find(next) {
        Some(m) => (
            Some(next[..m.start()].trim()).filter(|s| !s.is_empty()),
            m.as_str(),
            Some(next[m.end()..].trim()).filter(|s| !s.is_empty()),
        ),
        None => (None, "", Some(next.trim()).filter(|s| !s.is_empty())),
    }
}

fn build(head: &str, caps: &regex::Captures<'_>, next: &str) -> ParsedTransaction {
    let mut amount = caps["amount"].trim().to_string();
    let wrapped_amount = amount.ends_with('-');
    let has_code = ASSET_CODE.is_match(head);

    let (desc_tail, _, amount_tail) = split_continuation(next);
    let mut description = ASSET_CODE.replace(head, "").trim().to_string();

    if !has_code && let Some(tail) = desc_tail {
        description = format!("{description} {tail}").trim().to_string();
    }
    if wrapped_amount && let Some(tail) = amount_tail {
        amount = format!("{amount} {tail}");
    }

    let description = OWNER_PREFIX.replace(&description, "").trim().to_string();
    let description = LEADING_ID.replace(&description, "").trim().to_string();

    let symbol = TICKER_SUFFIX
        .captures(&description)
        .map(|c| c[1].to_uppercase());
    let asset_description = TICKER_SUFFIX.replace(&description, "").trim().to_string();

    ParsedTransaction {
        symbol,
        asset_description,
        trade_type: map_trade_type(&caps["ttype"]),
        transaction_date: parse_us_date(&caps["txdate"]).map(|d| d.to_string()),
        amount: Some(amount),
    }
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

    fn lines(rows: &[&str]) -> Vec<String> {
        rows.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn inline_ticker_and_owner_code_are_stripped_from_the_description() {
        let rows = parse_transactions(&lines(&[
            "SP Apple Inc. - Common Stock (AAPL) [ST] P 08/13/2026 08/18/2026 $1,001 - $15,000",
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(rows[0].trade_type.as_deref(), Some("Purchase"));
        assert_eq!(rows[0].transaction_date.as_deref(), Some("2026-08-13"));
        assert_eq!(rows[0].amount.as_deref(), Some("$1,001 - $15,000"));
        assert_eq!(rows[0].asset_description, "Apple Inc. - Common Stock");
    }

    #[test]
    fn description_wrapping_onto_the_next_line_is_rejoined() {
        let rows = parse_transactions(&lines(&[
            "Fidelity National Information S 05/04/2023 09/14/2023 $1,001 - $15,000",
            "Services, Inc. (FIS) [ST]",
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("FIS"));
        assert_eq!(
            rows[0].asset_description,
            "Fidelity National Information Services, Inc."
        );
    }

    #[test]
    fn amount_wrapping_onto_the_next_line_is_rejoined() {
        let rows = parse_transactions(&lines(&[
            "SP 3M Company (MMM) [ST] S 05/14/2020 05/20/2020 $15,001 -",
            "$50,000",
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].amount.as_deref(), Some("$15,001 - $50,000"));
        assert_eq!(rows[0].asset_description, "3M Company");
    }

    #[test]
    fn leading_filing_id_is_not_part_of_the_description() {
        let rows = parse_transactions(&lines(&[
            "2000086356 SP 3M Company (MMM) [ST] S 05/14/2020 05/20/2020 $1,001 - $15,000",
        ]));
        assert_eq!(rows[0].asset_description, "3M Company");
    }

    #[test]
    fn partial_qualifier_does_not_break_the_anchor() {
        let rows = parse_transactions(&lines(&[
            "MAI Managed [PS] S (partial) 01/10/2025 02/07/2025 $50,001 -",
            "$100,000",
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].trade_type.as_deref(), Some("Sale"));
        assert_eq!(rows[0].amount.as_deref(), Some("$50,001 - $100,000"));
        assert_eq!(rows[0].asset_description, "MAI Managed");
    }

    #[test]
    fn asset_code_on_the_continuation_line_keeps_both_halves() {
        let rows = parse_transactions(&lines(&[
            "Treasury Bill (3-month, matures P 08/18/2025 08/18/2025 $15,001 -",
            "11/20/2025) [GS] $50,000",
        ]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol, None);
        assert_eq!(rows[0].amount.as_deref(), Some("$15,001 - $50,000"));
        assert_eq!(
            rows[0].asset_description,
            "Treasury Bill (3-month, matures 11/20/2025)"
        );
    }

    #[test]
    fn a_word_ending_in_s_is_not_mistaken_for_a_sale() {
        let rows = parse_transactions(&lines(&[
            "Treasury Bill (3-month, matures P 08/18/2025 08/18/2025 $1,001 - $15,000",
        ]));
        assert_eq!(rows[0].trade_type.as_deref(), Some("Purchase"));
    }

    #[test]
    fn non_table_text_yields_no_rows() {
        assert!(
            parse_transactions(&lines(&["Digitally Signed: Hon. Ed Case , 08/18/2026"])).is_empty()
        );
        assert!(parse_transactions(&[]).is_empty());
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
        let tx = parse_transactions(&lines(&[
            "SP Apple Inc. - Common Stock (AAPL) [ST] P 08/13/2026 08/18/2026 $1,001 - $15,000",
        ]))
        .remove(0);
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
