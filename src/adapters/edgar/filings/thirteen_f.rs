//! 13F-HR institutional holdings parsed from the filed information table XML.
//!
//! The information table is a sibling document of the 13F cover page, so a
//! fetch costs one submissions call, one index lookup, and one document fetch.

use super::xml::{self, XmlNode};
use crate::adapters::edgar::{build_client, submissions_for_symbol_with};
use crate::error::{FinanceError, Result};
use crate::models::filings::InstitutionalHolding;

/// Forms carrying an information table. `13F-NT` (notice) never does.
const HOLDINGS_FORMS: [&str; 2] = ["13F-HR", "13F-HR/A"];

fn info_table_to_holding(
    row: &XmlNode,
    accession_number: &Option<String>,
    report_date: &Option<String>,
) -> InstitutionalHolding {
    InstitutionalHolding {
        issuer_name: row.text_at(&["nameOfIssuer"]),
        title_of_class: row.text_at(&["titleOfClass"]),
        cusip: row.text_at(&["cusip"]),
        value: row.number_at(&["value"]),
        shares: row.number_at(&["shrsOrPrnAmt", "sshPrnamt"]),
        share_type: row.text_at(&["shrsOrPrnAmt", "sshPrnamtType"]),
        put_call: row.text_at(&["putCall"]),
        investment_discretion: row.text_at(&["investmentDiscretion"]),
        voting_sole: row.number_at(&["votingAuthority", "Sole"]),
        voting_shared: row.number_at(&["votingAuthority", "Shared"]),
        voting_none: row.number_at(&["votingAuthority", "None"]),
        accession_number: accession_number.clone(),
        report_date: report_date.clone(),
    }
}

/// Parse a 13F information table into its position rows.
pub(crate) fn parse_information_table(
    bytes: &[u8],
    accession_number: Option<String>,
    report_date: Option<String>,
) -> Result<Vec<InstitutionalHolding>> {
    let doc = xml::parse(bytes)?;
    // `infoTable` sits directly under the root in practice, but scanning
    // descendants keeps the parse working if a wrapper element is added.
    let mut rows = Vec::new();
    doc.descendants("infoTable", &mut rows);
    Ok(rows
        .into_iter()
        .map(|row| info_table_to_holding(row, &accession_number, &report_date))
        .collect())
}

/// Pick the information table from a 13F filing's index listing.
///
/// The cover page (`primary_doc.xml`) is the other XML in the directory and
/// carries no positions, so it is excluded by name rather than by guessing.
pub(crate) fn pick_information_table(names: &[String]) -> Option<&String> {
    let is_candidate = |n: &&String| {
        let lower = n.to_ascii_lowercase();
        lower.ends_with(".xml") && !lower.starts_with("xsl") && !lower.ends_with("-index.xml")
    };
    names
        .iter()
        .filter(is_candidate)
        .find(|n| n.to_ascii_lowercase().contains("infotable"))
        .or_else(|| {
            names
                .iter()
                .filter(is_candidate)
                .find(|n| !n.to_ascii_lowercase().contains("primary_doc"))
        })
}

/// Fetch and parse the latest 13F-HR information table filed by `symbol`'s CIK.
pub async fn fetch_institutional_holdings_response(
    symbol: &str,
) -> Result<Vec<InstitutionalHolding>> {
    // One client for all three requests (CIK lookup, index, document) so the
    // connection pool survives the whole call.
    let client = build_client()?;
    let subs = submissions_for_symbol_with(&client, symbol).await?;
    let cik = subs.cik.clone().unwrap_or_default();
    let filing = subs
        .filings
        .and_then(|f| f.recent)
        .map(|r| r.to_filings())
        .unwrap_or_default()
        .into_iter()
        .find(|f| HOLDINGS_FORMS.contains(&f.form.as_str()))
        .ok_or_else(|| FinanceError::SymbolNotFound {
            symbol: Some(symbol.to_string()),
            context: format!("{symbol} has no 13F-HR filing on EDGAR"),
        })?;

    let accession = filing.accession_number.clone();
    let index = client.filing_index(&accession).await?;
    let names: Vec<String> = index
        .directory
        .item
        .into_iter()
        .map(|item| item.name)
        .collect();
    let doc_name =
        pick_information_table(&names).ok_or_else(|| FinanceError::ResponseStructureError {
            field: "informationTable".to_string(),
            context: format!("13F filing {accession} has no information table document"),
        })?;

    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
        cik.trim_start_matches('0'),
        accession.replace('-', ""),
        doc_name
    );
    let bytes = client.get_document(&url).await?;
    let report_date = (!filing.report_date.is_empty()).then(|| filing.report_date.clone());
    parse_information_table(&bytes, Some(accession), report_date)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &[u8] = br#"<?xml version="1.0"?>
    <informationTable xmlns="http://www.sec.gov/edgar/document/thirteenf/informationtable">
      <infoTable>
        <nameOfIssuer>APPLE INC</nameOfIssuer>
        <titleOfClass>COM</titleOfClass>
        <cusip>037833100</cusip>
        <value>174346000</value>
        <shrsOrPrnAmt><sshPrnamt>915560382</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
        <investmentDiscretion>DFND</investmentDiscretion>
        <votingAuthority><Sole>915560382</Sole><Shared>0</Shared><None>0</None></votingAuthority>
      </infoTable>
      <infoTable>
        <nameOfIssuer>COCA COLA CO</nameOfIssuer>
        <titleOfClass>COM</titleOfClass>
        <cusip>191216100</cusip>
        <value>25464000</value>
        <shrsOrPrnAmt><sshPrnamt>400000000</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
        <putCall>CALL</putCall>
        <votingAuthority><Sole>0</Sole><Shared>400000000</Shared><None>0</None></votingAuthority>
      </infoTable>
    </informationTable>"#;

    #[test]
    fn parses_every_position_row() {
        let rows = parse_information_table(
            TABLE,
            Some("0000950123-24-001".into()),
            Some("2024-03-31".into()),
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].issuer_name.as_deref(), Some("APPLE INC"));
        assert_eq!(rows[0].cusip.as_deref(), Some("037833100"));
        assert_eq!(rows[0].value, Some(174_346_000.0));
        assert_eq!(rows[0].shares, Some(915_560_382.0));
        assert_eq!(rows[0].share_type.as_deref(), Some("SH"));
        assert_eq!(rows[0].investment_discretion.as_deref(), Some("DFND"));
        assert_eq!(rows[0].voting_sole, Some(915_560_382.0));
        assert_eq!(rows[0].report_date.as_deref(), Some("2024-03-31"));
        assert_eq!(
            rows[0].accession_number.as_deref(),
            Some("0000950123-24-001")
        );
    }

    #[test]
    fn carries_option_and_shared_voting_details() {
        let rows = parse_information_table(TABLE, None, None).unwrap();
        assert_eq!(rows[1].put_call.as_deref(), Some("CALL"));
        assert_eq!(rows[1].voting_shared, Some(400_000_000.0));
        assert_eq!(rows[1].voting_sole, Some(0.0));
        assert_eq!(rows[1].investment_discretion, None);
    }

    #[test]
    fn parses_namespace_prefixed_tables() {
        let prefixed = br#"<ns1:informationTable xmlns:ns1="x">
          <ns1:infoTable><ns1:cusip>037833100</ns1:cusip><ns1:value>5</ns1:value></ns1:infoTable>
        </ns1:informationTable>"#;
        let rows = parse_information_table(prefixed, None, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cusip.as_deref(), Some("037833100"));
    }

    #[test]
    fn prefers_the_information_table_over_the_cover_page() {
        let names: Vec<String> = [
            "xslForm13F_X02/infotable.xml",
            "0000950123-24-001-index.xml",
            "primary_doc.xml",
            "form13fInfoTable.xml",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            pick_information_table(&names).map(String::as_str),
            Some("form13fInfoTable.xml")
        );
    }

    /// Older filings name the table arbitrarily; anything that is not the cover
    /// page is still a better guess than giving up.
    #[test]
    fn falls_back_to_the_non_cover_xml() {
        let names: Vec<String> = ["primary_doc.xml", "holdings2019.xml"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            pick_information_table(&names).map(String::as_str),
            Some("holdings2019.xml")
        );
        assert_eq!(
            pick_information_table(&["primary_doc.xml".to_string()]),
            None
        );
    }

    #[test]
    fn empty_table_yields_no_rows() {
        let empty = br#"<informationTable></informationTable>"#;
        assert!(
            parse_information_table(empty, None, None)
                .unwrap()
                .is_empty()
        );
    }
}
