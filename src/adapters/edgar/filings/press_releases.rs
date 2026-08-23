//! Company-issued press releases from 8-K exhibits (Exhibit 99.1/99.2).

use crate::adapters::edgar::filings::sections::html_to_text;
use crate::adapters::edgar::{accession_parts, build_client, submissions_for_symbol};
use crate::error::Result;
use crate::models::corporate::press_release::PressRelease;
use crate::models::filings::EdgarFiling;

/// Fetch the symbol's most recent 8-K exhibits tagged `EX-99*`, newest first.
pub async fn fetch_press_releases_response(symbol: &str, limit: u32) -> Result<Vec<PressRelease>> {
    let subs = submissions_for_symbol(symbol).await?;
    let eight_ks: Vec<EdgarFiling> = subs
        .filings
        .and_then(|f| f.recent)
        .map(|r| r.to_filings())
        .unwrap_or_default()
        .into_iter()
        .filter(|f| f.form == "8-K")
        .take(limit as usize)
        .collect();

    let fetches = eight_ks
        .into_iter()
        .map(|filing| async move { fetch_one(symbol, filing).await });
    Ok(futures::future::join_all(fetches)
        .await
        .into_iter()
        .flatten()
        .collect())
}

async fn fetch_one(symbol: &str, filing: EdgarFiling) -> Option<PressRelease> {
    let client = build_client().ok()?;
    let index = client.filing_index(&filing.accession_number).await.ok()?;
    let exhibit = index
        .directory
        .item
        .iter()
        .find(|i| i.item_type.to_ascii_uppercase().starts_with("EX-99"))?;

    let (cik, accession_no_dashes) = accession_parts(&filing.accession_number).ok()?;
    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{cik}/{accession_no_dashes}/{}",
        exhibit.name
    );
    let bytes = client.get_document(&url).await.ok()?;
    let text = html_to_text(&String::from_utf8_lossy(&bytes));
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    Some(PressRelease {
        symbol: Some(symbol.to_string()),
        date: Some(filing.filing_date),
        title: extract_title(text),
        text: Some(text.to_string()),
    })
}

fn extract_title(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_the_first_non_empty_line_of_the_exhibit_text() {
        let text = "\n  \nApple Reports Fourth Quarter Results\n\nCUPERTINO, Calif. ...";
        assert_eq!(
            extract_title(text),
            Some("Apple Reports Fourth Quarter Results".to_string())
        );
    }

    #[test]
    fn blank_text_has_no_title() {
        assert_eq!(extract_title("   \n\n  "), None);
    }
}
