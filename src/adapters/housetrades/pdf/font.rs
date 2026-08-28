//! Font resources: the `ToUnicode` CMap plus the glyph advances from `/W`.
//!
//! Advances come from the CIDFont's own `/W` array, so the embedded TrueType
//! program never has to be parsed. Without them a run of glyphs positioned
//! individually cannot be told apart from adjacent table columns.

use std::collections::HashMap;

use super::cmap::{self, CMap};
use super::document::Document;
use super::object::{Dict, Object};

const DEFAULT_WIDTH: f64 = 1000.0;

pub(super) struct Font {
    cmap: CMap,
    widths: HashMap<u32, f64>,
    default_width: f64,
}

impl Font {
    pub(super) fn decode(&self, bytes: &[u8]) -> String {
        self.cmap.decode(bytes)
    }

    /// Width of `bytes` at `size`, in text-space units.
    pub(super) fn advance(&self, bytes: &[u8], size: f64) -> f64 {
        let width = self.cmap.code_bytes();
        bytes
            .chunks(width)
            .map(|chunk| {
                let cid = chunk.iter().fold(0u32, |acc, b| (acc << 8) | u32::from(*b));
                self.widths.get(&cid).copied().unwrap_or(self.default_width)
            })
            .sum::<f64>()
            / 1000.0
            * size
    }
}

/// Build a font from a `/Font` resource entry, or `None` without a `ToUnicode`.
pub(super) fn load(doc: &Document, entry: &Object) -> Option<Font> {
    let dict = doc.dict_of(entry)?;
    let to_unicode = dict.get("ToUnicode").and_then(Object::as_ref_id)?;
    let cmap = cmap::parse(&doc.stream(to_unicode)?);
    if cmap.is_empty() {
        return None;
    }

    let descendant = dict
        .get("DescendantFonts")
        .and_then(Object::as_array)
        .and_then(|a| a.first())
        .and_then(|o| doc.dict_of(o));

    let (widths, default_width) = match descendant {
        Some(d) => (parse_widths(d), default_width(d)),
        None => (HashMap::new(), DEFAULT_WIDTH),
    };
    Some(Font {
        cmap,
        widths,
        default_width,
    })
}

fn default_width(dict: &Dict) -> f64 {
    dict.get("DW")
        .and_then(Object::as_f64)
        .unwrap_or(DEFAULT_WIDTH)
}

/// `/W [ cid [w ...] cfirst clast w ... ]`, both forms interleaved.
fn parse_widths(dict: &Dict) -> HashMap<u32, f64> {
    let mut out = HashMap::new();
    let Some(items) = dict.get("W").and_then(Object::as_array) else {
        return out;
    };
    let mut i = 0;
    while i < items.len() {
        let Some(first) = items[i].as_f64() else {
            i += 1;
            continue;
        };
        match items.get(i + 1) {
            Some(Object::Array(list)) => {
                for (offset, w) in list.iter().enumerate() {
                    if let Some(w) = w.as_f64() {
                        out.insert(first as u32 + offset as u32, w);
                    }
                }
                i += 2;
            }
            Some(second) => {
                let (Some(last), Some(w)) =
                    (second.as_f64(), items.get(i + 2).and_then(Object::as_f64))
                else {
                    i += 2;
                    continue;
                };
                for cid in (first as u32)..=(last as u32) {
                    out.insert(cid, w);
                }
                i += 3;
            }
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::housetrades::pdf::object::Lexer;

    fn dict(src: &[u8]) -> Dict {
        Lexer::new(src, 0)
            .object()
            .and_then(|o| match o {
                Object::Dict(d) => Some(d),
                _ => None,
            })
            .expect("dict")
    }

    #[test]
    fn parses_indexed_width_lists() {
        let w = parse_widths(&dict(b"<< /W [0 [1000 500] 36 [670 653]] >>"));
        assert_eq!(w.get(&0), Some(&1000.0));
        assert_eq!(w.get(&1), Some(&500.0));
        assert_eq!(w.get(&36), Some(&670.0));
        assert_eq!(w.get(&37), Some(&653.0));
    }

    #[test]
    fn parses_first_last_ranges() {
        let w = parse_widths(&dict(b"<< /W [10 12 250] >>"));
        assert_eq!(w.get(&10), Some(&250.0));
        assert_eq!(w.get(&11), Some(&250.0));
        assert_eq!(w.get(&12), Some(&250.0));
        assert_eq!(w.get(&13), None);
    }

    #[test]
    fn parses_both_forms_interleaved() {
        let w = parse_widths(&dict(b"<< /W [0 [100] 5 7 300 20 [400 500]] >>"));
        assert_eq!(w.get(&0), Some(&100.0));
        assert_eq!(w.get(&6), Some(&300.0));
        assert_eq!(w.get(&21), Some(&500.0));
    }

    #[test]
    fn missing_w_array_yields_no_widths() {
        assert!(parse_widths(&dict(b"<< /Type /Font >>")).is_empty());
    }

    #[test]
    fn default_width_falls_back_to_1000() {
        assert_eq!(default_width(&dict(b"<< >>")), 1000.0);
        assert_eq!(default_width(&dict(b"<< /DW 512 >>")), 512.0);
    }
}
