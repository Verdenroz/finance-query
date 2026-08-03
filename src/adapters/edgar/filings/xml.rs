//! Minimal element-tree reader for EDGAR's ownership and 13F XML documents.
//!
//! Deliberately dependency-free for the same reason [`crate::feeds::parser`]
//! is (RUSTSEC-2026-0195 lived in a third-party XML crate's namespace
//! handling): these documents need element nesting and text, nothing else —
//! no DTDs, no entity expansion beyond the five predefined entities, no
//! namespace resolution. Prefixes are simply stripped, which is correct here
//! because EDGAR ownership/13F schemas have no colliding local names.

use crate::error::{FinanceError, Result};
use crate::feeds::parser::{find_byte, trim_ascii, unescape};

/// One element: its local name, its own text, and its child elements.
#[derive(Debug, Default, Clone)]
pub(crate) struct XmlNode {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) children: Vec<XmlNode>,
}

impl XmlNode {
    /// First direct child named `name`.
    pub(crate) fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Every direct child named `name`.
    pub(crate) fn children_named<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a XmlNode> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// Every descendant named `name`, at any depth.
    pub(crate) fn descendants<'a>(&'a self, name: &'a str, out: &mut Vec<&'a XmlNode>) {
        for child in &self.children {
            if child.name == name {
                out.push(child);
            }
            child.descendants(name, out);
        }
    }

    /// Trimmed text at a nested path of element names.
    pub(crate) fn text_at(&self, path: &[&str]) -> Option<String> {
        let mut node = self;
        for step in path {
            node = node.child(step)?;
        }
        let text = node.text.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    /// [`text_at`](Self::text_at) parsed as `f64`. EDGAR writes numbers with
    /// thousands separators and stray currency symbols in places, so those are
    /// stripped before parsing.
    pub(crate) fn number_at(&self, path: &[&str]) -> Option<f64> {
        let raw = self.text_at(path)?;
        let cleaned: String = raw
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();
        cleaned.parse().ok()
    }
}

/// Strip a namespace prefix (`ns1:infoTable` → `infoTable`).
fn local_name(raw: &[u8]) -> String {
    let name = match raw.iter().position(|&b| b == b':') {
        Some(i) => &raw[i + 1..],
        None => raw,
    };
    String::from_utf8_lossy(name).to_string()
}

/// Parse an XML document into its root element.
///
/// Errors only on structurally broken input (unterminated tag, mismatched
/// close, no root) — unknown elements and attributes are ignored, so schema
/// drift degrades to missing fields rather than a hard failure.
pub(crate) fn parse(bytes: &[u8]) -> Result<XmlNode> {
    let err = |context: &str| -> FinanceError {
        FinanceError::ResponseStructureError {
            field: "xml".to_string(),
            context: context.to_string(),
        }
    };

    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root: Option<XmlNode> = None;
    let mut i = 0usize;

    while i < bytes.len() {
        let Some(open) = find_byte(bytes, i, b'<') else {
            break;
        };

        // Text between the previous tag and this one belongs to the open element.
        if open > i
            && let Some(node) = stack.last_mut()
        {
            let raw = trim_ascii(&bytes[i..open]);
            if !raw.is_empty() {
                node.text.push_str(&unescape(raw));
            }
        }

        // Comments, CDATA, doctypes, and processing instructions carry no
        // structure we need; skip to their own terminator.
        if bytes[open..].starts_with(b"<!--") {
            i = find_seq(bytes, open + 4, b"-->").ok_or_else(|| err("unterminated comment"))? + 3;
            continue;
        }
        if bytes[open..].starts_with(b"<![CDATA[") {
            let end = find_seq(bytes, open + 9, b"]]>").ok_or_else(|| err("unterminated CDATA"))?;
            if let Some(node) = stack.last_mut() {
                node.text
                    .push_str(&String::from_utf8_lossy(&bytes[open + 9..end]));
            }
            i = end + 3;
            continue;
        }
        if bytes[open..].starts_with(b"<?") || bytes[open..].starts_with(b"<!") {
            i = find_byte(bytes, open, b'>').ok_or_else(|| err("unterminated declaration"))? + 1;
            continue;
        }

        let close = find_byte(bytes, open, b'>').ok_or_else(|| err("unterminated tag"))?;
        let inner = &bytes[open + 1..close];
        i = close + 1;

        if inner.first() == Some(&b'/') {
            let name = local_name(trim_ascii(&inner[1..]));
            let node = stack
                .pop()
                .ok_or_else(|| err("closing tag without opener"))?;
            if node.name != name {
                return Err(err(&format!(
                    "mismatched closing tag: expected </{}>, found </{name}>",
                    node.name
                )));
            }
            match stack.last_mut() {
                Some(parent) => parent.children.push(node),
                None => root = Some(node),
            }
            continue;
        }

        let self_closing = inner.last() == Some(&b'/');
        let body = if self_closing {
            &inner[..inner.len() - 1]
        } else {
            inner
        };
        let name_end = body
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(body.len());
        let node = XmlNode {
            name: local_name(&body[..name_end]),
            ..Default::default()
        };

        if self_closing {
            match stack.last_mut() {
                Some(parent) => parent.children.push(node),
                None => root = Some(node),
            }
        } else {
            stack.push(node);
        }
    }

    if !stack.is_empty() {
        return Err(err("unclosed elements at end of document"));
    }
    root.ok_or_else(|| err("no root element"))
}

fn find_seq(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nesting_text_and_repeated_children() {
        let xml = br#"<?xml version="1.0"?>
            <root>
              <item><name>a</name><qty>1</qty></item>
              <item><name>b</name><qty>2</qty></item>
            </root>"#;
        let root = parse(xml).unwrap();

        assert_eq!(root.name, "root");
        let items: Vec<_> = root.children_named("item").collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text_at(&["name"]).as_deref(), Some("a"));
        assert_eq!(items[1].number_at(&["qty"]), Some(2.0));
    }

    #[test]
    fn strips_namespace_prefixes() {
        let xml = br#"<ns1:informationTable xmlns:ns1="http://www.sec.gov/edgar">
            <ns1:infoTable><ns1:cusip>037833100</ns1:cusip></ns1:infoTable>
        </ns1:informationTable>"#;
        let root = parse(xml).unwrap();

        assert_eq!(root.name, "informationTable");
        let info = root.child("infoTable").unwrap();
        assert_eq!(info.text_at(&["cusip"]).as_deref(), Some("037833100"));
    }

    #[test]
    fn handles_comments_cdata_and_self_closing_tags() {
        let xml = br#"<root>
            <!-- ignored -->
            <empty/>
            <note><![CDATA[raw <text> & stuff]]></note>
        </root>"#;
        let root = parse(xml).unwrap();

        assert!(root.child("empty").is_some());
        assert_eq!(
            root.text_at(&["note"]).as_deref(),
            Some("raw <text> & stuff")
        );
    }

    #[test]
    fn decodes_predefined_entities_in_text() {
        let root = parse(br#"<root><name>AT&amp;T &lt;Inc&gt;</name></root>"#).unwrap();
        assert_eq!(root.text_at(&["name"]).as_deref(), Some("AT&T <Inc>"));
    }

    #[test]
    fn number_at_tolerates_separators_and_rejects_junk() {
        let root = parse(br#"<r><a>1,234,567</a><b>$12.50</b><c>n/a</c></r>"#).unwrap();
        assert_eq!(root.number_at(&["a"]), Some(1_234_567.0));
        assert_eq!(root.number_at(&["b"]), Some(12.50));
        assert_eq!(root.number_at(&["c"]), None);
    }

    #[test]
    fn descendants_finds_nodes_at_any_depth() {
        let root = parse(br#"<a><b><c>1</c></b><d><e><c>2</c></e></d></a>"#).unwrap();
        let mut found = Vec::new();
        root.descendants("c", &mut found);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn structural_breakage_is_an_error() {
        assert!(parse(b"<a><b></a>").is_err(), "mismatched close");
        assert!(parse(b"<a>").is_err(), "unclosed element");
        assert!(parse(b"<a").is_err(), "unterminated tag");
        assert!(parse(b"text only").is_err(), "no root");
    }
}
