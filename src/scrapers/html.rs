//! Minimal, dependency-free HTML element matcher.
//!
//! finance-query's scraping needs are narrow: find elements by tag name
//! (optionally scoped inside another element), read one attribute off a
//! start tag, and read an element's concatenated text content. That's a
//! small enough surface that owning it outright removes an entire class of
//! transitive dependency risk from a general-purpose HTML5 engine -- the
//! same reasoning behind `feeds::parser`'s hand-rolled RSS/Atom extractor,
//! whose byte-scanning approach (and entity decoding) this mirrors.
//!
//! This is NOT a spec-compliant parser: no implicit tag-closing rules, no
//! DOM tree, no CSS combinators beyond what callers build by chaining
//! `find_first`/`find_all` (e.g. `h3 a` is `find_first(scope, "h3")` then
//! `find_first(h3.inner, "a")`). Unclosed tags are tolerated -- the
//! element's content is treated as running to the end of its search scope
//! -- rather than erroring, since real-world third-party HTML is far less
//! predictable than the RSS/Atom feeds `feeds::parser` handles.

const VOID_TAGS: &[&[u8]] = &[
    b"area", b"base", b"br", b"col", b"embed", b"hr", b"img", b"input", b"link", b"meta", b"param",
    b"source", b"track", b"wbr",
];

fn is_void(tag: &[u8]) -> bool {
    VOID_TAGS.iter().any(|v| v.eq_ignore_ascii_case(tag))
}

/// One matched HTML element.
pub(crate) struct Element<'a> {
    open_tag: &'a [u8],
    pub(crate) inner: &'a str,
}

impl Element<'_> {
    /// The value of `name` on this element's opening tag, if present.
    pub(crate) fn attr(&self, name: &str) -> Option<String> {
        find_attr(self.open_tag, name.as_bytes())
    }

    /// Concatenated text content, tags stripped, entities decoded.
    pub(crate) fn text(&self) -> String {
        strip_tags(self.inner.as_bytes())
    }
}

/// Find the first `<tag ...>` in `scope` and return it with its content
/// span. Tag matching is case-insensitive, like real HTML.
pub(crate) fn find_first<'a>(scope: &'a str, tag: &str) -> Option<Element<'a>> {
    let bytes = scope.as_bytes();
    let start = find_tag_open(bytes, tag.as_bytes())?;
    parse_element(bytes, start, tag.as_bytes()).map(|(el, _)| el)
}

/// Find every `<tag ...>` in `scope`, at any nesting depth -- including
/// tags nested inside other matches of the same name (matching
/// `document.select(&Selector::parse("tag"))`'s "all descendants"
/// semantics).
pub(crate) fn find_all<'a>(scope: &'a str, tag: &str) -> Vec<Element<'a>> {
    let bytes = scope.as_bytes();
    let tag = tag.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(rel) = find_tag_open(&bytes[i..], tag) {
        let start = i + rel;
        match parse_element(bytes, start, tag) {
            // Resume right after this element's own opening tag (not past
            // its content) so nested same-name elements are still found.
            Some((el, open_tag_end)) => {
                i = open_tag_end;
                out.push(el);
            }
            None => i = start + 1,
        }
    }
    out
}

/// Find the first `<tag>` in `scope` whose opening tag carries `attr`.
pub(crate) fn find_first_with_attr<'a>(
    scope: &'a str,
    tag: &str,
    attr: &str,
) -> Option<Element<'a>> {
    find_all(scope, tag)
        .into_iter()
        .find(|el| el.attr(attr).is_some())
}

/// Byte offset of the next `<tag` occurrence in `haystack` at a real
/// tag-name boundary (not a prefix match -- searching for `a` must skip
/// `<article`).
fn find_tag_open(haystack: &[u8], tag: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(rel) = find_byte(haystack, i, b'<') {
        if match_open_name(haystack, rel, tag).is_some() {
            return Some(rel);
        }
        i = rel + 1;
    }
    None
}

/// If `haystack[idx..]` is `<tag` at a real name boundary, the index right
/// after the tag name.
fn match_open_name(haystack: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    let name_start = idx + 1;
    let name_end = name_start + tag.len();
    if haystack.len() < name_end || !haystack[name_start..name_end].eq_ignore_ascii_case(tag) {
        return None;
    }
    match haystack.get(name_end) {
        Some(b) if b.is_ascii_whitespace() || *b == b'>' || *b == b'/' => Some(name_end),
        _ => None,
    }
}

/// If `haystack[idx..]` is `</tag` at a real name boundary, the index right
/// after the tag name.
fn match_close_name(haystack: &[u8], idx: usize, tag: &[u8]) -> Option<usize> {
    if haystack.get(idx + 1) != Some(&b'/') {
        return None;
    }
    let name_start = idx + 2;
    let name_end = name_start + tag.len();
    if haystack.len() < name_end || !haystack[name_start..name_end].eq_ignore_ascii_case(tag) {
        return None;
    }
    match haystack.get(name_end) {
        Some(b) if b.is_ascii_whitespace() || *b == b'>' => Some(name_end),
        _ => None,
    }
}

/// Parse one element starting at `haystack[tag_start..]` (a `<tag`
/// boundary already confirmed by the caller via `find_tag_open`). Returns
/// the element plus the offset right after its own opening tag.
fn parse_element<'a>(
    haystack: &'a [u8],
    tag_start: usize,
    tag: &[u8],
) -> Option<(Element<'a>, usize)> {
    let name_end = match_open_name(haystack, tag_start, tag)?;
    let (open_tag, open_end, self_closing) = parse_open_tag(haystack, name_end)?;

    if self_closing || is_void(tag) {
        return Some((
            Element {
                open_tag,
                inner: "",
            },
            open_end,
        ));
    }

    let content_end = find_matching_close(haystack, open_end, tag);
    let inner = std::str::from_utf8(&haystack[open_end..content_end])
        .expect("span bounded by ASCII tag delimiters, always valid UTF-8");
    Some((Element { open_tag, inner }, open_end))
}

/// Parse `<tagname ...>` / `<tagname .../>` starting right after the tag
/// name (`from`). Returns (attribute text, index right after `>`, whether
/// self-closing). Tolerant of `>` inside quoted attribute values.
fn parse_open_tag(haystack: &[u8], from: usize) -> Option<(&[u8], usize, bool)> {
    let mut i = from;
    let mut quote: Option<u8> = None;
    while i < haystack.len() {
        let b = haystack[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None if b == b'"' || b == b'\'' => quote = Some(b),
            None if b == b'>' => {
                let self_closing = i > from && haystack[i - 1] == b'/';
                let attrs_end = if self_closing { i - 1 } else { i };
                return Some((&haystack[from..attrs_end], i + 1, self_closing));
            }
            None => {}
        }
        i += 1;
    }
    None
}

/// Find where `tag`'s content ends: the matching `</tag>` (tracking nested
/// same-name open tags), or the end of `haystack` if unclosed.
fn find_matching_close(haystack: &[u8], from: usize, tag: &[u8]) -> usize {
    let mut depth = 0usize;
    let mut i = from;
    while let Some(rel) = find_byte(haystack, i, b'<') {
        if let Some(name_end) = match_close_name(haystack, rel, tag) {
            if depth == 0 {
                return rel;
            }
            depth -= 1;
            i = find_byte(haystack, name_end, b'>')
                .map(|g| g + 1)
                .unwrap_or(haystack.len());
            continue;
        }
        if let Some(name_end) = match_open_name(haystack, rel, tag)
            && let Some((_, open_end, self_closing)) = parse_open_tag(haystack, name_end)
        {
            if !self_closing && !is_void(tag) {
                depth += 1;
            }
            i = open_end;
            continue;
        }
        i = rel + 1;
    }
    haystack.len()
}

/// Concatenate all text content within `html`, stripping tags/comments and
/// decoding entities.
fn strip_tags(html: &[u8]) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while let Some(rel) = find_byte(html, i, b'<') {
        out.push_str(&unescape(&html[i..rel]));
        i = if html[rel..].starts_with(b"<!--") {
            find(html, rel + 4, b"-->")
                .map(|p| p + 3)
                .unwrap_or(html.len())
        } else {
            skip_tag(html, rel + 1)
        };
    }
    out.push_str(&unescape(&html[i..]));
    out
}

/// Index right after the next unquoted `>` at or after `from` (tolerant of
/// `>` inside quoted attribute values), or end of input if none.
fn skip_tag(html: &[u8], from: usize) -> usize {
    let mut i = from;
    let mut quote: Option<u8> = None;
    while i < html.len() {
        let b = html[i];
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None if b == b'"' || b == b'\'' => quote = Some(b),
            None if b == b'>' => return i + 1,
            None => {}
        }
        i += 1;
    }
    html.len()
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

/// Find an attribute's value within a start tag's raw bytes (after the tag
/// name). Handles both `"` and `'` quoting; requires a whitespace boundary
/// before the attribute name so e.g. `xhref` doesn't match a lookup for
/// `href`.
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

/// Decode the 5 predefined XML/HTML entities and numeric character
/// references (`&#NN;` / `&#xHH;`). Unrecognized or malformed entities are
/// left as a literal `&` rather than erroring -- scraped HTML in the wild
/// is inconsistent here (mirrors `feeds::parser::unescape`).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_table_rows_and_cells() {
        let html = r#"
            <table>
              <tr><th>Country</th><th>Suffix</th></tr>
              <tr><td>Japan</td><td>.T</td></tr>
              <tr><td>Hong Kong</td><td>.HK</td></tr>
            </table>
        "#;
        let table = find_first(html, "table").unwrap();
        let rows = find_all(table.inner, "tr");
        assert_eq!(rows.len(), 3);

        let data_rows: Vec<_> = rows
            .iter()
            .map(|r| find_all(r.inner, "td"))
            .filter(|cells| cells.len() == 2)
            .collect();
        assert_eq!(data_rows.len(), 2);
        assert_eq!(data_rows[0][0].text(), "Japan");
        assert_eq!(data_rows[0][1].text(), ".T");
    }

    #[test]
    fn finds_all_nested_divs() {
        let html = r#"<div id="outer"><div id="inner">x</div></div>"#;
        let divs = find_all(html, "div");
        assert_eq!(divs.len(), 2);
        assert_eq!(divs[0].attr("id").as_deref(), Some("outer"));
        assert_eq!(divs[1].attr("id").as_deref(), Some("inner"));
    }

    #[test]
    fn chained_descendant_lookup() {
        let html = r#"<div><h3><a href="/news/1">Fed holds rates</a></h3></div>"#;
        let item = find_first(html, "div").unwrap();
        let title = find_first(item.inner, "h3")
            .and_then(|h3| find_first(h3.inner, "a"))
            .unwrap();
        assert_eq!(title.text(), "Fed holds rates");
        assert_eq!(title.attr("href").as_deref(), Some("/news/1"));
    }

    #[test]
    fn find_first_with_attr_matches_presence_not_value() {
        let html = r#"<div>text</div><div title="14 hours ago - Reuters">meta</div>"#;
        let el = find_first_with_attr(html, "div", "title").unwrap();
        assert_eq!(el.text(), "meta");
    }

    #[test]
    fn void_elements_have_no_content() {
        let html = r#"<div><img src="/a.png"><p>after</p></div>"#;
        let item = find_first(html, "div").unwrap();
        let img = find_first(item.inner, "img").unwrap();
        assert_eq!(img.attr("src").as_deref(), Some("/a.png"));
        assert_eq!(img.inner, "");
    }

    #[test]
    fn self_closing_img_also_works() {
        let html = r#"<img src="/a.png"/>"#;
        let img = find_first(html, "img").unwrap();
        assert_eq!(img.attr("src").as_deref(), Some("/a.png"));
    }

    #[test]
    fn text_strips_nested_tags_and_decodes_entities() {
        let html = r#"<td>AT&amp;T <b>Inc</b>.</td>"#;
        let cell = find_first(html, "td").unwrap();
        assert_eq!(cell.text(), "AT&T Inc.");
    }

    #[test]
    fn greater_than_inside_quoted_attr_does_not_end_tag_early() {
        let html = r#"<div title="a > b">ok</div>"#;
        let el = find_first(html, "div").unwrap();
        assert_eq!(el.attr("title").as_deref(), Some("a > b"));
        assert_eq!(el.text(), "ok");
    }

    #[test]
    fn unclosed_tag_is_tolerated_not_an_error() {
        let html = r#"<div><p>trailing content, no closing div"#;
        let el = find_first(html, "div").unwrap();
        assert!(el.inner.contains("trailing content"));
    }

    #[test]
    fn attribute_name_boundary_is_respected() {
        let html = r#"<div xhref="wrong" href="right">x</div>"#;
        let el = find_first(html, "div").unwrap();
        assert_eq!(el.attr("href").as_deref(), Some("right"));
    }

    #[test]
    fn no_match_returns_none() {
        assert!(find_first("<div>no table here</div>", "table").is_none());
        assert!(find_all("<div>no rows</div>", "tr").is_empty());
    }
}
