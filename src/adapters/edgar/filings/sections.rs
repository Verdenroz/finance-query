//! Sectioned filing text and risk factors from 10-K/8-K HTML documents.
//! Heuristic heading detection; returns best-effort results on malformed filings.

use std::sync::LazyLock;

use regex::Regex;

use crate::adapters::edgar::{accession_parts, build_client, submissions_for_symbol};
use crate::error::{FinanceError, Result};
use crate::models::filings::{FilingSection, FilingSectionForm, RiskFactor};

/// Block-level tags become line breaks; inline tags are stripped without breaks
/// to keep words split across them adjacent.
static BLOCK_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)</?(?:p|div|tr|li|br|table|h[1-6])(?:\s[^>]*)?/?>").unwrap()
});

static SCRIPT_OR_STYLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>").unwrap()
});

static ANY_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<[^>]+>").unwrap());

/// Matches 10-K/10-Q item headings at line start, handling varied title placement.
static TEN_K_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?mi)^[ \t]*item[ \t]+(\d{1,2}[a-c]?)\b").unwrap());

/// Matches an 8-K item heading ("Item 5.02", "ITEM 9.01").
static EIGHT_K_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?mi)^[ \t]*item[ \t]+(\d{1,2}\.\d{2})\b").unwrap());

/// Matches an exhibit filename ("ex99_1.htm", "a8-kex991...htm"). The
/// directory listing's `type` field is a file-icon class, not the exhibit
/// number, so exhibits are excluded by filename instead.
static EXHIBIT_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)ex-?\d").unwrap());

/// Matches SEC's auto-generated per-fact XBRL viewer pages ("R1.htm",
/// "R23.htm") — these can be larger than the actual filed document.
static XBRL_VIEWER_PAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^r\d+\.html?$").unwrap());

/// Numeric HTML entities (e.g. `&#8212;`, `&#x2014;`) — filed HTML leans on
/// these for curly quotes and dashes far more than the named entities below.
static NUMERIC_ENTITY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)&#(x[0-9a-f]+|[0-9]+);").unwrap());

/// Flatten filing HTML to plain text with block tags as line breaks for heading extraction.
pub(super) fn html_to_text(html: &str) -> String {
    let no_script = SCRIPT_OR_STYLE.replace_all(html, "");
    let with_breaks = BLOCK_TAG.replace_all(&no_script, "\n");
    let flat = ANY_TAG.replace_all(&with_breaks, "");
    decode_entities(&flat)
}

fn decode_entities(text: &str) -> String {
    let named = text
        .replace("&nbsp;", " ")
        .replace("&rsquo;", "'")
        .replace("&lsquo;", "'")
        .replace("&rdquo;", "\"")
        .replace("&ldquo;", "\"")
        .replace("&mdash;", "-")
        .replace("&ndash;", "-")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    NUMERIC_ENTITY
        .replace_all(&named, |caps: &regex::Captures| {
            let code = &caps[1];
            let codepoint = code
                .strip_prefix('x')
                .or_else(|| code.strip_prefix('X'))
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| code.parse().ok());
            codepoint
                .and_then(char::from_u32)
                .map(String::from)
                .unwrap_or_else(|| caps[0].to_string())
        })
        .into_owned()
}

fn heading_matches(text: &str, heading: &Regex) -> Vec<(String, usize)> {
    heading
        .captures_iter(text)
        .map(|c| {
            let label = c.get(1).unwrap().as_str().to_uppercase();
            (label, c.get(0).unwrap().start())
        })
        .collect()
}

/// Keep only the last occurrence of each item label (real section after TOC).
fn last_occurrence_per_label(matches: Vec<(String, usize)>) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut last: HashMap<String, usize> = HashMap::new();
    for (label, start) in matches {
        last.insert(label, start);
    }
    let mut out: Vec<(String, usize)> = last.into_iter().collect();
    out.sort_by_key(|(_, start)| *start);
    out
}

fn slice_sections(text: &str, heading: &Regex) -> Vec<FilingSection> {
    let matches = last_occurrence_per_label(heading_matches(text, heading));
    matches
        .iter()
        .enumerate()
        .map(|(i, (label, start))| {
            let heading_end = text[*start..]
                .find('\n')
                .map(|off| start + off)
                .unwrap_or(text.len());
            let content_end = matches.get(i + 1).map(|(_, s)| *s).unwrap_or(text.len());
            let content = text[heading_end..content_end].trim();
            FilingSection {
                section: Some(format!("item_{}", label.to_lowercase().replace('.', "_"))),
                content: (!content.is_empty()).then(|| content.to_string()),
            }
        })
        .collect()
}

fn primary_document_name(index: &crate::models::filings::EdgarFilingIndex) -> Option<String> {
    index
        .directory
        .item
        .iter()
        .filter(|i| {
            let name = i.name.to_lowercase();
            (name.ends_with(".htm") || name.ends_with(".html"))
                && !EXHIBIT_NAME.is_match(&name)
                && !XBRL_VIEWER_PAGE.is_match(&name)
        })
        .max_by_key(|i| i.size.unwrap_or(0))
        .map(|i| i.name.clone())
}

pub async fn fetch_filing_sections_response(
    accession_number: &str,
    form: FilingSectionForm,
) -> Result<Vec<FilingSection>> {
    let client = build_client()?;
    let index = client.filing_index(accession_number).await?;

    let form_str = match form {
        FilingSectionForm::TenK => "10-K",
        FilingSectionForm::EightK => "8-K",
    };
    let filename =
        primary_document_name(&index).ok_or_else(|| FinanceError::ResponseStructureError {
            field: "directory.item".to_string(),
            context: format!("no primary {form_str} document found for {accession_number}"),
        })?;

    let (cik, accession_no_dashes) = accession_parts(accession_number)?;
    let url =
        format!("https://www.sec.gov/Archives/edgar/data/{cik}/{accession_no_dashes}/{filename}");
    let bytes = client.get_document(&url).await?;
    let text = html_to_text(&String::from_utf8_lossy(&bytes));

    let heading = match form {
        FilingSectionForm::TenK => &*TEN_K_ITEM,
        FilingSectionForm::EightK => &*EIGHT_K_ITEM,
    };
    Ok(slice_sections(&text, heading))
}

/// Split text into risk factors on heading-shaped lines; fallback to one factor with full text.
fn split_risk_factors(content: &str, filing_date: Option<&str>) -> Vec<RiskFactor> {
    fn looks_like_heading(line: &str) -> bool {
        let len = line.chars().count();
        (10..140).contains(&len)
            && !line.ends_with(['.', ',', ';'])
            && line.chars().next().is_some_and(char::is_uppercase)
    }

    let mut factors = Vec::new();
    let mut title: Option<String> = None;
    let mut body = String::new();

    for line in content.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if looks_like_heading(line) {
            if title.is_some() || !body.is_empty() {
                factors.push(RiskFactor {
                    title: title.take(),
                    text: (!body.is_empty()).then(|| body.trim().to_string()),
                    category: None,
                    filing_date: filing_date.map(str::to_string),
                });
                body.clear();
            }
            title = Some(line.to_string());
        } else {
            if !body.is_empty() {
                body.push(' ');
            }
            body.push_str(line);
        }
    }
    if title.is_some() || !body.is_empty() {
        factors.push(RiskFactor {
            title,
            text: (!body.is_empty()).then(|| body.trim().to_string()),
            category: None,
            filing_date: filing_date.map(str::to_string),
        });
    }
    factors
}

/// Fetch risk factors from a symbol's most recent 10-K.
pub async fn fetch_risk_factors_response(symbol: &str) -> Result<Vec<RiskFactor>> {
    let subs = submissions_for_symbol(symbol).await?;
    let filing = subs
        .filings
        .and_then(|f| f.recent)
        .map(|r| r.to_filings())
        .unwrap_or_default()
        .into_iter()
        .find(|f| f.form == "10-K" && !f.primary_document.is_empty());

    let Some(filing) = filing else {
        return Ok(Vec::new());
    };

    let (cik, accession_no_dashes) = accession_parts(&filing.accession_number)?;
    let url = format!(
        "https://www.sec.gov/Archives/edgar/data/{cik}/{accession_no_dashes}/{}",
        filing.primary_document
    );
    let client = build_client()?;
    let bytes = client.get_document(&url).await?;
    let text = html_to_text(&String::from_utf8_lossy(&bytes));

    let risk_section = slice_sections(&text, &TEN_K_ITEM)
        .into_iter()
        .find(|s| s.section.as_deref() == Some("item_1a"));

    Ok(match risk_section.and_then(|s| s.content) {
        Some(content) => split_risk_factors(&content, Some(&filing.filing_date)),
        None => Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_tags_become_breaks_inline_tags_stay_adjacent() {
        let html = "<html><body><p><span>Item 1A.</span> <span>Risk Factors</span></p>\
                     <p>We face intense competition.</p></body></html>";
        let text = html_to_text(html);
        assert!(text.contains("Item 1A. Risk Factors"));
        assert!(text.contains("We face intense competition."));
    }

    #[test]
    fn entities_decode() {
        assert_eq!(
            decode_entities("Tom&#39;s &amp; Jerry&nbsp;Co."),
            "Tom's & Jerry Co."
        );
    }

    #[test]
    fn primary_document_ignores_exhibits_and_xbrl_viewer_pages() {
        use crate::models::filings::EdgarFilingIndex;
        use crate::models::filings::filing_index::{
            EdgarFilingIndexDirectory, EdgarFilingIndexItem,
        };

        let item = |name: &str, size: u64| EdgarFilingIndexItem {
            name: name.to_string(),
            item_type: "text.gif".to_string(),
            size: Some(size),
        };
        let index = EdgarFilingIndex {
            directory: EdgarFilingIndexDirectory {
                item: vec![
                    item("a8-kex991q3202606272026.htm", 173_484),
                    item("aapl-20260730.htm", 38_350),
                    item("R1.htm", 55_284),
                ],
            },
        };
        assert_eq!(
            primary_document_name(&index),
            Some("aapl-20260730.htm".to_string())
        );
    }

    #[test]
    fn numeric_entities_decode_decimal_and_hex() {
        assert_eq!(
            decode_entities("Tim Cook&#8217;s &#8220;record&#8221; quarter&#8212;strong"),
            "Tim Cook\u{2019}s \u{201c}record\u{201d} quarter\u{2014}strong"
        );
        assert_eq!(decode_entities("&#x2014;"), "\u{2014}");
    }

    #[test]
    fn last_occurrence_wins_over_a_table_of_contents_entry() {
        let text = "Item 1A. Risk Factors .......... 12\n\
                     \n\
                     Item 1A. Risk Factors\n\
                     Our business faces many risks.\n\
                     Item 2. Properties\n\
                     We lease office space.\n";
        let sections = slice_sections(text, &TEN_K_ITEM);
        let risk = sections
            .iter()
            .find(|s| s.section.as_deref() == Some("item_1a"))
            .unwrap();
        assert_eq!(
            risk.content.as_deref(),
            Some("Our business faces many risks.")
        );
    }

    #[test]
    fn eight_k_items_use_dotted_labels() {
        let text = "Item 5.02. Departure of Directors\n\
                     Jane Doe resigned as CFO.\n\
                     Item 9.01. Financial Statements and Exhibits\n\
                     See attached exhibits.\n";
        let sections = slice_sections(text, &EIGHT_K_ITEM);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].section.as_deref(), Some("item_5_02"));
        assert_eq!(sections[1].section.as_deref(), Some("item_9_01"));
    }

    #[test]
    fn risk_factors_split_on_heading_shaped_lines() {
        let content = "Risks Related to Our Business\n\
                        We face intense competition from larger rivals.\n\
                        Our margins may decline as a result.\n\
                        Our Supply Chain Is Concentrated\n\
                        A single supplier accounts for most of our inventory.\n";
        let factors = split_risk_factors(content, Some("2026-02-01"));
        assert_eq!(factors.len(), 2);
        assert_eq!(
            factors[0].title.as_deref(),
            Some("Risks Related to Our Business")
        );
        assert_eq!(
            factors[0].text.as_deref(),
            Some(
                "We face intense competition from larger rivals. Our margins may decline as a result."
            )
        );
        assert_eq!(factors[0].filing_date.as_deref(), Some("2026-02-01"));
        assert_eq!(
            factors[1].title.as_deref(),
            Some("Our Supply Chain Is Concentrated")
        );
    }

    #[test]
    fn risk_factors_without_heading_shaped_lines_fall_back_to_one_factor() {
        let content = "This is a single long paragraph with no clear sub-headings, \
                        just prose that keeps going and going about risk in general.";
        let factors = split_risk_factors(content, None);
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].title, None);
        assert_eq!(factors[0].text.as_deref(), Some(content));
    }
}
