//! Company-issued press releases from 8-K exhibits (Exhibit 99.1/99.2).

use crate::adapters::edgar::filings::sections::html_to_text;
use crate::adapters::edgar::{accession_parts, build_client, submissions_for_symbol};
use crate::error::Result;
use crate::models::corporate::press_release::PressRelease;
use crate::models::filings::EdgarFiling;
use crate::models::filings::filing_index::EdgarFilingIndexItem;

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
    let exhibit = select_exhibit(&index.directory.item)?;

    let (cik, accession_no_dashes) = accession_parts(&filing.accession_number).ok()?;
    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{cik}/{accession_no_dashes}/{}",
        exhibit.name
    );
    let bytes = client.get_document(&url).await.ok()?;
    let text = html_to_text(&String::from_utf8_lossy(&bytes));
    let body = strip_leading_chrome(text.trim());
    if body.is_empty() {
        return None;
    }

    Some(PressRelease {
        symbol: Some(symbol.to_string()),
        date: Some(filing.filing_date),
        title: extract_title(&body),
        text: Some(body),
    })
}

/// index.json's `type` field is a file-icon class (e.g. "text.gif"), not the
/// exhibit number, so EX-99 exhibits are identified by filename instead
/// (e.g. "a8-kex991q3202606272026.htm", "msft-ex99_1.htm").
fn is_ex99_exhibit(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.contains("ex99") || name.contains("ex-99")
}

/// Some filers split the exhibit across multiple ex99-named entries (an
/// image asset alongside the document); pick the largest as the real one.
fn select_exhibit(items: &[EdgarFilingIndexItem]) -> Option<&EdgarFilingIndexItem> {
    items
        .iter()
        .filter(|i| is_ex99_exhibit(&i.name))
        .max_by_key(|i| i.size.unwrap_or(0))
}

/// Some filers embed the SEC inline-viewer's own chrome (exhibit label,
/// page number, filename) as visible text at the top of the exhibit HTML,
/// ahead of the real headline — skip those lines rather than the content.
fn is_viewer_chrome(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let caption_digits = lower
        .strip_prefix("exhibit ")
        .or_else(|| lower.strip_prefix("ex-"))
        .unwrap_or_default();
    line.chars().all(|c| c.is_ascii_digit())
        || lower.ends_with(".htm")
        || lower.ends_with(".html")
        || lower == "document"
        || (!caption_digits.is_empty()
            && caption_digits
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.'))
}

fn extract_title(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !is_viewer_chrome(l))
        .map(str::to_string)
}

/// Drop the inline-viewer chrome lines from the front of the body, not just
/// when picking a title — the same junk otherwise leads every extracted
/// press release's `text`.
fn strip_leading_chrome(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty() && !is_viewer_chrome(trimmed)
        })
        .unwrap_or(lines.len());
    lines[start..].join("\n")
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

    #[test]
    fn title_skips_the_inline_viewers_own_chrome() {
        let text = "EX-99.1\n2\na8-kex991q3202606272026.htm\nEX-99.1\n\nDocument\n\nExhibit 99.1\n\nApple reports third quarter results\n\nCUPERTINO, CALIFORNIA...";
        assert_eq!(
            extract_title(text),
            Some("Apple reports third quarter results".to_string())
        );
    }

    #[test]
    fn strip_leading_chrome_drops_viewer_boilerplate_and_keeps_the_body() {
        let text = "EX-99.1\n2\na8-kex991q3202606272026.htm\nEX-99.1\n\nDocument\n\nExhibit 99.1\n\nApple reports third quarter results\n\nCUPERTINO, CALIFORNIA...";
        assert_eq!(
            strip_leading_chrome(text),
            "Apple reports third quarter results\n\nCUPERTINO, CALIFORNIA..."
        );
    }

    #[test]
    fn strip_leading_chrome_is_a_no_op_when_there_is_none() {
        let text = "Apple reports third quarter results\n\nCUPERTINO, CALIFORNIA...";
        assert_eq!(strip_leading_chrome(text), text);
    }

    #[test]
    fn select_exhibit_picks_the_largest_when_multiple_ex99_entries_match() {
        let item = |name: &str, size: u64| EdgarFilingIndexItem {
            name: name.to_string(),
            item_type: "text.gif".to_string(),
            size: Some(size),
        };
        let items = vec![
            item("aapl-ex99_2.htm", 1_200),
            item("aapl-ex99_1.htm", 45_000),
            item("aapl-ex99_1_g1.jpg", 8_000),
        ];
        assert_eq!(
            select_exhibit(&items).map(|i| &i.name),
            Some(&"aapl-ex99_1.htm".to_string())
        );
    }

    #[test]
    fn ex99_exhibit_names_are_recognized_regardless_of_filer_convention() {
        assert!(is_ex99_exhibit("a8-kex991q3202606272026.htm"));
        assert!(is_ex99_exhibit("msft-ex99_1.htm"));
        assert!(is_ex99_exhibit("EX-99.1.htm"));
        assert!(!is_ex99_exhibit("aapl-20260730.htm"));
        assert!(!is_ex99_exhibit("aapl-20260730_g1.jpg"));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_live_press_releases() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let releases = fetch_press_releases_response("AAPL", 10).await.unwrap();
        assert!(!releases.is_empty());
        assert!(releases[0].title.is_some());
        let text = releases[0].text.as_deref().unwrap();
        assert!(!text.starts_with("EX-99"));
        assert_eq!(text.lines().next(), releases[0].title.as_deref());
    }
}
