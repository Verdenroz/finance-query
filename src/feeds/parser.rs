//! Minimal, dependency-free RSS 2.0 / Atom 1.0 entry extractor.
//!
//! We only need to identify a fixed handful of well-known tag names such as
//! `title`,`link`, `pubDate`/`published`/`updated`,
//! `description`/`summary`/`content`/`content:encoded`) and the `href`
//! attribute on `<link>` — a small enough surface that owning the tokenizer
//! outright removes an entire class of transitive dependency risk (see
//! RUSTSEC-2026-0195, which lived in a third-party XML crate's namespace
//! resolution — logic this parser never had and now cannot reintroduce).
//! There is also no DTD/entity-expansion support (only the 5 predefined XML
//! entities + numeric character refs, each capped at 10 bytes), so billion-laughs-style
//! entity bombs aren't a parseable construct here at all.
//!
//! Input is assumed to already be valid UTF-8 (it comes from
//! `reqwest::Response::text()`, which performs charset decoding), so byte
//! offsets from ASCII delimiter scans (`<`, `>`, `&`, `;`, quotes) are always
//! safe UTF-8 boundaries — none of those bytes can appear inside a
//! multi-byte UTF-8 continuation sequence.

use chrono::DateTime;

use super::FeedEntry;
use crate::error::{FinanceError, Result};

#[derive(Default)]
struct PartialEntry {
    title: Option<String>,
    url: Option<String>,
    published_raw: Option<String>,
    updated_raw: Option<String>,
    summary_raw: Option<String>,
    content_raw: Option<String>,
}

impl PartialEntry {
    fn finish(self, source: &str) -> Option<FeedEntry> {
        let title = self.title?.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let url = self.url?.trim().to_string();
        if url.is_empty() {
            return None;
        }
        let published = self
            .published_raw
            .or(self.updated_raw)
            .and_then(|raw| parse_date(&raw));
        let summary = self
            .summary_raw
            .or(self.content_raw)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Some(FeedEntry {
            title,
            url,
            published,
            summary,
            source: source.to_string(),
        })
    }
}

/// Which leaf field an open tag's text content should accumulate into.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Link,
    Published,
    Updated,
    Summary,
    Content,
}

fn field_for(tag: &[u8]) -> Option<Field> {
    match tag {
        b"title" => Some(Field::Title),
        b"pubDate" | b"published" => Some(Field::Published),
        b"updated" => Some(Field::Updated),
        b"description" | b"summary" => Some(Field::Summary),
        b"content" | b"content:encoded" => Some(Field::Content),
        _ => None,
    }
}

fn append(entry: &mut PartialEntry, field: Field, text: String) {
    let slot = match field {
        Field::Title => &mut entry.title,
        Field::Link => &mut entry.url,
        Field::Published => &mut entry.published_raw,
        Field::Updated => &mut entry.updated_raw,
        Field::Summary => &mut entry.summary_raw,
        Field::Content => &mut entry.content_raw,
    };
    match slot {
        Some(existing) => existing.push_str(&text),
        None => *slot = Some(text),
    }
}

/// RSS 2.0 `pubDate` is RFC 2822; Atom `published`/`updated` is RFC 3339.
/// Invalid or unrecognized dates are dropped rather than propagated, matching
/// prior (feed-rs-based) behavior where an unparsable date left the field `None`.
fn parse_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    // Some real-world feed generators emit a weekday name that doesn't match
    // the actual date (an off-by-one bug in the generator); chrono's rfc2822
    // parser validates day-name/date agreement and rejects the whole value
    // on a mismatch. We never use the weekday, so strip it before parsing
    // rather than losing an otherwise-valid date over a cosmetic error.
    let without_weekday = raw
        .find(',')
        .filter(|&comma| comma <= 4)
        .map_or(raw, |comma| raw[comma + 1..].trim_start());

    DateTime::parse_from_rfc2822(without_weekday)
        .or_else(|_| DateTime::parse_from_rfc3339(raw))
        .ok()
        .map(|dt| dt.to_rfc3339())
}

fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn find_byte(haystack: &[u8], from: usize, byte: u8) -> Option<usize> {
    if from > haystack.len() {
        return None;
    }
    haystack[from..]
        .iter()
        .position(|&b| b == byte)
        .map(|p| p + from)
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes[start..]
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map(|p| start + p + 1)
        .unwrap_or(start);
    &bytes[start..end]
}

/// Decode the 5 predefined XML entities and numeric character references
/// (`&#NN;` / `&#xHH;`). Unrecognized or malformed entities are left as a
/// literal `&` rather than erroring — real-world feeds are inconsistent here.
fn unescape(raw: &[u8]) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    loop {
        match find_byte(raw, i, b'&') {
            Some(amp) => {
                out.push_str(&String::from_utf8_lossy(&raw[i..amp]));
                let decoded = find_byte(raw, amp, b';')
                    .filter(|&semi| semi - amp <= 10)
                    .and_then(|semi| decode_entity(&raw[amp + 1..semi]).map(|ch| (ch, semi)));
                match decoded {
                    Some((ch, semi)) => {
                        out.push(ch);
                        i = semi + 1;
                    }
                    None => {
                        out.push('&');
                        i = amp + 1;
                    }
                }
            }
            None => {
                out.push_str(&String::from_utf8_lossy(&raw[i..]));
                break;
            }
        }
    }
    out
}

fn decode_entity(entity: &[u8]) -> Option<char> {
    match entity {
        b"amp" => Some('&'),
        b"lt" => Some('<'),
        b"gt" => Some('>'),
        b"quot" => Some('"'),
        b"apos" => Some('\''),
        _ if entity.starts_with(b"#x") || entity.starts_with(b"#X") => {
            let hex = std::str::from_utf8(&entity[2..]).ok()?;
            u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
        }
        _ if entity.starts_with(b"#") => {
            let dec = std::str::from_utf8(&entity[1..]).ok()?;
            dec.parse::<u32>().ok().and_then(char::from_u32)
        }
        _ => None,
    }
}

/// Find an attribute's value within a start tag's raw bytes (after the tag
/// name). Handles both `"` and `'` quoting and arbitrary intervening
/// whitespace/newlines; requires a whitespace boundary before the attribute
/// name so e.g. `xhref` doesn't match a lookup for `href`.
fn find_attr(tag_bytes: &[u8], attr: &[u8]) -> Option<String> {
    let mut i = 0;
    while i < tag_bytes.len() {
        if tag_bytes[i..].starts_with(attr) {
            let boundary_ok = i == 0 || tag_bytes[i - 1].is_ascii_whitespace();
            if boundary_ok {
                let mut j = i + attr.len();
                while tag_bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                if tag_bytes.get(j) == Some(&b'=') {
                    j += 1;
                    while tag_bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                        j += 1;
                    }
                    if let Some(&quote) = tag_bytes.get(j)
                        && (quote == b'"' || quote == b'\'')
                        && let Some(end) = find_byte(tag_bytes, j + 1, quote)
                    {
                        return Some(unescape(&tag_bytes[j + 1..end]));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Parse already-fetched RSS/Atom bytes into entries.
pub(super) fn parse(bytes: &[u8], source: &str) -> Result<Vec<FeedEntry>> {
    let len = bytes.len();
    let mut i = 0usize;
    let mut entries = Vec::new();
    let mut current: Option<PartialEntry> = None;
    let mut capturing: Option<(Field, &[u8])> = None;
    let mut stack: Vec<&[u8]> = Vec::new();

    let err = |context: String| -> FinanceError {
        FinanceError::FeedParseError {
            url: source.to_string(),
            context,
        }
    };

    while i < len {
        if bytes[i] != b'<' {
            let end = find_byte(bytes, i, b'<').unwrap_or(len);
            if let (Some((field, _)), Some(cur)) = (capturing, current.as_mut()) {
                append(cur, field, unescape(&bytes[i..end]));
            }
            i = end;
            continue;
        }

        if bytes[i..].starts_with(b"<!--") {
            let end =
                find(bytes, i + 4, b"-->").ok_or_else(|| err("unterminated comment".into()))?;
            i = end + 3;
        } else if bytes[i..].starts_with(b"<![CDATA[") {
            let start = i + 9;
            let end = find(bytes, start, b"]]>")
                .ok_or_else(|| err("unterminated CDATA section".into()))?;
            if let (Some((field, _)), Some(cur)) = (capturing, current.as_mut()) {
                append(
                    cur,
                    field,
                    String::from_utf8_lossy(&bytes[start..end]).into_owned(),
                );
            }
            i = end + 3;
        } else if bytes[i..].starts_with(b"<?") {
            let end = find(bytes, i + 2, b"?>")
                .ok_or_else(|| err("unterminated processing instruction".into()))?;
            i = end + 2;
        } else if bytes[i..].starts_with(b"<!") {
            let end = find_byte(bytes, i + 2, b'>')
                .ok_or_else(|| err("unterminated declaration".into()))?;
            i = end + 1;
        } else if bytes.get(i + 1) == Some(&b'/') {
            let end =
                find_byte(bytes, i + 2, b'>').ok_or_else(|| err("unterminated end tag".into()))?;
            let name = trim_ascii(&bytes[i + 2..end]);

            match stack.pop() {
                Some(top) if top == name => {}
                Some(top) => {
                    return Err(err(format!(
                        "mismatched closing tag: expected </{}>, found </{}>",
                        String::from_utf8_lossy(top),
                        String::from_utf8_lossy(name)
                    )));
                }
                None => {
                    return Err(err(format!(
                        "unexpected closing tag </{}> with no open element",
                        String::from_utf8_lossy(name)
                    )));
                }
            }

            if name == b"item" || name == b"entry" {
                if let Some(cur) = current.take()
                    && let Some(entry) = cur.finish(source)
                {
                    entries.push(entry);
                }
                capturing = None;
            } else if let Some((_, tag)) = capturing
                && name == tag
            {
                capturing = None;
            }
            i = end + 1;
        } else {
            let end = find_byte(bytes, i + 1, b'>')
                .ok_or_else(|| err("unterminated start tag".into()))?;
            let self_closing = end > i + 1 && bytes[end - 1] == b'/';
            let content_end = if self_closing { end - 1 } else { end };
            let tag_bytes = &bytes[i + 1..content_end];
            let name_end = tag_bytes
                .iter()
                .position(u8::is_ascii_whitespace)
                .unwrap_or(tag_bytes.len());
            let name = &tag_bytes[..name_end];

            if !self_closing {
                stack.push(name);
            }

            if name == b"item" || name == b"entry" {
                current = Some(PartialEntry::default());
            } else if name == b"link" && current.is_some() {
                if let Some(href) = find_attr(tag_bytes, b"href") {
                    let cur = current.as_mut().expect("checked is_some above");
                    if cur.url.is_none() {
                        cur.url = Some(href);
                    }
                } else if !self_closing {
                    capturing = Some((Field::Link, name));
                }
            } else if current.is_some()
                && !self_closing
                && let Some(field) = field_for(name)
            {
                capturing = Some((field, name));
            }
            i = end + 1;
        }
    }

    if !stack.is_empty() {
        return Err(err(format!(
            "unexpected end of document with {} unclosed tag(s)",
            stack.len()
        )));
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rss_2_0() {
        let xml = br#"<?xml version="1.0"?>
        <rss version="2.0">
          <channel>
            <title>Feed Title</title>
            <link>https://example.com</link>
            <item>
              <title>Fed holds rates steady</title>
              <link>https://example.com/news/fed-holds-rates</link>
              <pubDate>Sat, 13 Jun 2026 13:30:00 GMT</pubDate>
              <description>Some &amp; summary &lt;text&gt;</description>
            </item>
            <item>
              <title>Second item</title>
              <link>https://example.com/news/second</link>
              <pubDate>Sat, 13 Jun 2026 12:00:00 GMT</pubDate>
              <description><![CDATA[<p>HTML body</p>]]></description>
            </item>
          </channel>
        </rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Fed holds rates steady");
        assert_eq!(entries[0].url, "https://example.com/news/fed-holds-rates");
        assert_eq!(entries[0].source, "Test");
        assert_eq!(entries[0].summary.as_deref(), Some("Some & summary <text>"));
        assert_eq!(
            entries[0].published.as_deref(),
            Some("2026-06-13T13:30:00+00:00")
        );
        assert_eq!(entries[1].summary.as_deref(), Some("<p>HTML body</p>"));
    }

    #[test]
    fn parses_atom_1_0() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <title>Example Feed</title>
          <link href="https://example.com/"/>
          <entry>
            <title>Atom Entry</title>
            <link rel="alternate" href="https://example.com/entry1"/>
            <published>2026-06-13T13:30:00Z</published>
            <updated>2026-06-13T14:00:00Z</updated>
            <summary>An atom summary</summary>
          </entry>
          <entry>
            <title>No summary, has content</title>
            <link href="https://example.com/entry2"/>
            <updated>2026-06-13T15:00:00Z</updated>
            <content type="html">Body content</content>
          </entry>
        </feed>"#;

        let entries = parse(xml, "AtomSource").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].url, "https://example.com/entry1");
        assert_eq!(
            entries[0].published.as_deref(),
            Some("2026-06-13T13:30:00+00:00")
        );
        assert_eq!(entries[0].summary.as_deref(), Some("An atom summary"));

        assert_eq!(entries[1].url, "https://example.com/entry2");
        // No <published>, falls back to <updated>.
        assert_eq!(
            entries[1].published.as_deref(),
            Some("2026-06-13T15:00:00+00:00")
        );
        // No <summary>, falls back to <content>.
        assert_eq!(entries[1].summary.as_deref(), Some("Body content"));
    }

    #[test]
    fn handles_comments_and_processing_instructions() {
        let xml = br#"<?xml version="1.0"?>
        <!-- top-level comment -->
        <rss version="2.0"><channel>
            <!-- another comment -->
            <item>
              <title>Has PI</title>
              <link>https://example.com/pi</link>
              <?some-pi data?>
              <description>text</description>
            </item>
        </channel></rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Has PI");
    }

    #[test]
    fn decodes_numeric_entities() {
        let xml = br#"<rss version="2.0"><channel>
            <item>
              <title>Caf&#233; &#x2014; numeric refs</title>
              <link>https://example.com/numeric</link>
            </item>
        </channel></rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert_eq!(entries[0].title, "Café — numeric refs");
    }

    #[test]
    fn skips_entries_missing_title_or_link() {
        let xml = br#"<rss version="2.0"><channel>
            <item><link>https://example.com/no-title</link></item>
            <item><title>No link</title></item>
            <item><title>  </title><link>https://example.com/blank-title</link></item>
        </channel></rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn invalid_date_is_dropped_not_propagated() {
        let xml = br#"<rss version="2.0"><channel>
            <item>
              <title>Bad date</title>
              <link>https://example.com/bad-date</link>
              <pubDate>not a date</pubDate>
            </item>
        </channel></rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].published, None);
    }

    #[test]
    fn tolerates_pub_date_weekday_mismatch() {
        // 2026-06-13 is actually a Saturday; some feed generators get the
        // weekday label wrong. chrono's strict rfc2822 parser rejects that
        // mismatch outright, so the parser strips the weekday before parsing.
        let xml = br#"<rss version="2.0"><channel>
            <item>
              <title>Wrong weekday label</title>
              <link>https://example.com/wrong-weekday</link>
              <pubDate>Fri, 13 Jun 2026 13:30:00 GMT</pubDate>
            </item>
        </channel></rss>"#;

        let entries = parse(xml, "Test").unwrap();
        assert_eq!(
            entries[0].published.as_deref(),
            Some("2026-06-13T13:30:00+00:00")
        );
    }

    #[test]
    fn malformed_xml_is_an_error() {
        let xml = b"<rss><channel><item><title>Unclosed";
        assert!(parse(xml, "Test").is_err());
    }

    #[test]
    fn mismatched_closing_tag_is_an_error() {
        let xml = b"<rss><channel></item></channel></rss>";
        assert!(parse(xml, "Test").is_err());
    }
}
