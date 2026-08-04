//! Form 3/4/5 insider transactions parsed from the filed ownership XML.
//!
//! EDGAR's JSON APIs only list these filings; the transaction detail lives in
//! the filing's XML document, so each filing costs one index lookup plus one
//! document fetch on top of the shared submissions call.

use futures::stream::{self, StreamExt};

use super::xml::{self, XmlNode};
use crate::adapters::edgar::client::EdgarClient;
use crate::adapters::edgar::{build_client, submissions_for_symbol_with};
use crate::error::Result;
use crate::models::filings::InsiderTrade;

/// Forms that report insider ownership changes, amendments included.
const OWNERSHIP_FORMS: [&str; 6] = ["3", "4", "5", "3/A", "4/A", "5/A"];

/// EDGAR allows 10 req/sec; each filing costs two, so stay well under.
const CONCURRENCY: usize = 4;

/// A flag element is `1`/`true` when set; anything else (including absent) is false.
fn flag(node: &XmlNode, path: &[&str]) -> bool {
    matches!(
        node.text_at(path).as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

/// Map one `<nonDerivativeTransaction>` / `<derivativeTransaction>` element.
fn transaction_to_trade(tx: &XmlNode, header: &InsiderTrade, is_derivative: bool) -> InsiderTrade {
    InsiderTrade {
        security_title: tx.text_at(&["securityTitle", "value"]),
        transaction_date: tx.text_at(&["transactionDate", "value"]),
        transaction_code: tx.text_at(&["transactionCoding", "transactionCode"]),
        acquired_disposed: tx.text_at(&[
            "transactionAmounts",
            "transactionAcquiredDisposedCode",
            "value",
        ]),
        shares: tx
            .number_at(&["transactionAmounts", "transactionShares", "value"])
            .or_else(|| tx.number_at(&["transactionAmounts", "transactionTotalValue", "value"])),
        price_per_share: tx.number_at(&["transactionAmounts", "transactionPricePerShare", "value"]),
        shares_owned_after: tx.number_at(&[
            "postTransactionAmounts",
            "sharesOwnedFollowingTransaction",
            "value",
        ]),
        is_derivative,
        ..header.clone()
    }
}

/// Parse one ownership document into its transaction lines.
///
/// A Form 3 (initial statement of holdings) reports positions rather than
/// transactions, so it legitimately yields no rows.
pub(crate) fn parse_ownership_document(
    bytes: &[u8],
    accession_number: Option<String>,
    url: Option<String>,
) -> Result<Vec<InsiderTrade>> {
    let doc = xml::parse(bytes)?;

    let owner = doc.child("reportingOwner");
    let header = InsiderTrade {
        symbol: doc.text_at(&["issuer", "issuerTradingSymbol"]),
        issuer_name: doc.text_at(&["issuer", "issuerName"]),
        insider_name: owner.and_then(|o| o.text_at(&["reportingOwnerId", "rptOwnerName"])),
        insider_cik: owner.and_then(|o| o.text_at(&["reportingOwnerId", "rptOwnerCik"])),
        is_director: owner.is_some_and(|o| flag(o, &["reportingOwnerRelationship", "isDirector"])),
        is_officer: owner.is_some_and(|o| flag(o, &["reportingOwnerRelationship", "isOfficer"])),
        is_ten_percent_owner: owner
            .is_some_and(|o| flag(o, &["reportingOwnerRelationship", "isTenPercentOwner"])),
        officer_title: owner
            .and_then(|o| o.text_at(&["reportingOwnerRelationship", "officerTitle"])),
        form_type: doc.text_at(&["documentType"]),
        accession_number,
        url,
        ..Default::default()
    };

    let mut trades = Vec::new();
    for (table, derivative) in [("nonDerivativeTable", false), ("derivativeTable", true)] {
        let Some(table) = doc.child(table) else {
            continue;
        };
        let element = if derivative {
            "derivativeTransaction"
        } else {
            "nonDerivativeTransaction"
        };
        trades.extend(
            table
                .children_named(element)
                .map(|tx| transaction_to_trade(tx, &header, derivative)),
        );
    }
    Ok(trades)
}

/// Pick the filed ownership XML from a filing's index listing.
///
/// The `xsl*/` entries are SEC's human-readable renderings of the same
/// document, and `*-index.xml` is the directory itself — neither parses as an
/// ownership document.
pub(crate) fn pick_ownership_xml(names: &[String]) -> Option<&String> {
    names.iter().find(|n| {
        let lower = n.to_ascii_lowercase();
        lower.ends_with(".xml") && !lower.starts_with("xsl") && !lower.ends_with("-index.xml")
    })
}

/// Fetch and parse recent Form 3/4/5 filings for a symbol, newest first.
pub async fn fetch_insider_trades_response(symbol: &str, limit: u32) -> Result<Vec<InsiderTrade>> {
    // One client for the whole call: `limit` filings cost two requests each, and
    // a fresh client per request discards the pool before it is ever reused.
    let client = build_client()?;
    let client = &client;
    let subs = submissions_for_symbol_with(client, symbol).await?;
    let cik = subs.cik.clone().unwrap_or_default();
    let filings = subs
        .filings
        .and_then(|f| f.recent)
        .map(|r| r.to_filings())
        .unwrap_or_default();

    let wanted: Vec<_> = filings
        .into_iter()
        .filter(|f| OWNERSHIP_FORMS.contains(&f.form.as_str()))
        .take(limit as usize)
        .collect();

    let trades: Vec<Vec<InsiderTrade>> = stream::iter(wanted)
        .map(|filing| {
            let cik = cik.clone();
            async move {
                let accession = filing.accession_number.clone();
                match fetch_one(client, &cik, &accession).await {
                    Ok(t) => t,
                    // One unparseable filing must not sink the whole history —
                    // ownership XML schemas vary across two decades of filings.
                    Err(e) => {
                        tracing::debug!("EDGAR insider filing {accession} skipped: {e}");
                        Vec::new()
                    }
                }
            }
        })
        .buffered(CONCURRENCY)
        .collect()
        .await;

    Ok(trades.into_iter().flatten().collect())
}

async fn fetch_one(client: &EdgarClient, cik: &str, accession: &str) -> Result<Vec<InsiderTrade>> {
    let index = client.filing_index(accession).await?;
    let names: Vec<String> = index
        .directory
        .item
        .into_iter()
        .map(|item| item.name)
        .collect();
    let Some(doc_name) = pick_ownership_xml(&names) else {
        return Ok(Vec::new());
    };

    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
        cik.trim_start_matches('0'),
        accession.replace('-', ""),
        doc_name
    );
    let bytes = client.get_document(&url).await?;
    parse_ownership_document(&bytes, Some(accession.to_string()), Some(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORM4: &[u8] = br#"<?xml version="1.0"?>
    <ownershipDocument>
      <documentType>4</documentType>
      <issuer>
        <issuerCik>0000320193</issuerCik>
        <issuerName>Apple Inc.</issuerName>
        <issuerTradingSymbol>AAPL</issuerTradingSymbol>
      </issuer>
      <reportingOwner>
        <reportingOwnerId>
          <rptOwnerCik>0001214128</rptOwnerCik>
          <rptOwnerName>COOK TIMOTHY D</rptOwnerName>
        </reportingOwnerId>
        <reportingOwnerRelationship>
          <isDirector>1</isDirector>
          <isOfficer>1</isOfficer>
          <isTenPercentOwner>0</isTenPercentOwner>
          <officerTitle>Chief Executive Officer</officerTitle>
        </reportingOwnerRelationship>
      </reportingOwner>
      <nonDerivativeTable>
        <nonDerivativeTransaction>
          <securityTitle><value>Common Stock</value></securityTitle>
          <transactionDate><value>2024-04-01</value></transactionDate>
          <transactionCoding><transactionFormType>4</transactionFormType><transactionCode>S</transactionCode></transactionCoding>
          <transactionAmounts>
            <transactionShares><value>511000</value></transactionShares>
            <transactionPricePerShare><value>170.1234</value></transactionPricePerShare>
            <transactionAcquiredDisposedCode><value>D</value></transactionAcquiredDisposedCode>
          </transactionAmounts>
          <postTransactionAmounts>
            <sharesOwnedFollowingTransaction><value>3280000</value></sharesOwnedFollowingTransaction>
          </postTransactionAmounts>
        </nonDerivativeTransaction>
      </nonDerivativeTable>
      <derivativeTable>
        <derivativeTransaction>
          <securityTitle><value>Restricted Stock Unit</value></securityTitle>
          <transactionDate><value>2024-04-01</value></transactionDate>
          <transactionCoding><transactionCode>M</transactionCode></transactionCoding>
          <transactionAmounts>
            <transactionShares><value>1000</value></transactionShares>
            <transactionAcquiredDisposedCode><value>A</value></transactionAcquiredDisposedCode>
          </transactionAmounts>
        </derivativeTransaction>
      </derivativeTable>
    </ownershipDocument>"#;

    #[test]
    fn parses_both_transaction_tables_with_shared_header() {
        let trades =
            parse_ownership_document(FORM4, Some("0000320193-24-000123".into()), None).unwrap();

        assert_eq!(trades.len(), 2);
        for t in &trades {
            assert_eq!(t.symbol.as_deref(), Some("AAPL"));
            assert_eq!(t.insider_name.as_deref(), Some("COOK TIMOTHY D"));
            assert_eq!(t.form_type.as_deref(), Some("4"));
            assert_eq!(t.accession_number.as_deref(), Some("0000320193-24-000123"));
            assert!(t.is_director && t.is_officer && !t.is_ten_percent_owner);
            assert_eq!(t.officer_title.as_deref(), Some("Chief Executive Officer"));
        }
    }

    #[test]
    fn maps_transaction_detail_and_flags_derivatives() {
        let trades = parse_ownership_document(FORM4, None, None).unwrap();

        let sale = &trades[0];
        assert!(!sale.is_derivative);
        assert_eq!(sale.security_title.as_deref(), Some("Common Stock"));
        assert_eq!(sale.transaction_date.as_deref(), Some("2024-04-01"));
        assert_eq!(sale.transaction_code.as_deref(), Some("S"));
        assert_eq!(sale.acquired_disposed.as_deref(), Some("D"));
        assert_eq!(sale.shares, Some(511_000.0));
        assert_eq!(sale.price_per_share, Some(170.1234));
        assert_eq!(sale.shares_owned_after, Some(3_280_000.0));

        let exercise = &trades[1];
        assert!(exercise.is_derivative);
        assert_eq!(exercise.transaction_code.as_deref(), Some("M"));
        assert_eq!(exercise.acquired_disposed.as_deref(), Some("A"));
        assert_eq!(exercise.price_per_share, None);
    }

    /// A Form 3 lists holdings, not transactions — an empty result is correct,
    /// not a parse failure.
    #[test]
    fn form_3_without_transactions_yields_no_rows() {
        let form3 = br#"<ownershipDocument>
            <documentType>3</documentType>
            <issuer><issuerTradingSymbol>AAPL</issuerTradingSymbol></issuer>
            <nonDerivativeTable>
              <nonDerivativeHolding>
                <securityTitle><value>Common Stock</value></securityTitle>
              </nonDerivativeHolding>
            </nonDerivativeTable>
        </ownershipDocument>"#;
        assert!(
            parse_ownership_document(form3, None, None)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn picks_the_filed_xml_over_the_rendered_and_index_ones() {
        let names: Vec<String> = [
            "xslF345X03/wf-form4_171.xml",
            "0000320193-24-000123-index.xml",
            "wf-form4_171.xml",
            "primary_doc.html",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            pick_ownership_xml(&names).map(String::as_str),
            Some("wf-form4_171.xml")
        );
        assert_eq!(pick_ownership_xml(&[]), None);
    }

    #[test]
    fn missing_owner_block_degrades_to_empty_fields() {
        let doc = br#"<ownershipDocument><documentType>4</documentType>
            <nonDerivativeTable><nonDerivativeTransaction>
              <transactionAmounts><transactionShares><value>5</value></transactionShares></transactionAmounts>
            </nonDerivativeTransaction></nonDerivativeTable>
        </ownershipDocument>"#;
        let trades = parse_ownership_document(doc, None, None).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].insider_name, None);
        assert!(!trades[0].is_officer);
        assert_eq!(trades[0].shares, Some(5.0));
    }
}
