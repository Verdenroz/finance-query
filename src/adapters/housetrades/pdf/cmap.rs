//! `ToUnicode` CMaps: the code-to-text mapping for embedded subset fonts.
//!
//! A PTR embeds two or three subset fonts whose codes are arbitrary glyph
//! indices, so the CMap is the only way to recover characters. Each font gets
//! its own map, because the same code means different things in different
//! fonts.

use std::collections::HashMap;

use super::object::Lexer;

#[derive(Debug, Default)]
pub(super) struct CMap {
    map: HashMap<u32, String>,
    code_bytes: usize,
}

impl CMap {
    pub(super) fn code_bytes(&self) -> usize {
        if self.code_bytes == 0 {
            2
        } else {
            self.code_bytes
        }
    }

    pub(super) fn lookup(&self, code: u32) -> Option<&str> {
        self.map.get(&code).map(String::as_str)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Decode a PDF string operand into text using this map's code width.
    pub(super) fn decode(&self, bytes: &[u8]) -> String {
        let width = self.code_bytes();
        bytes
            .chunks(width)
            .map(|chunk| {
                let code = chunk.iter().fold(0u32, |acc, b| (acc << 8) | u32::from(*b));
                self.lookup(code).unwrap_or("").to_string()
            })
            .collect()
    }
}

fn utf16be(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_be_bytes(*p))
        .collect();
    String::from_utf16_lossy(&units)
}

fn code_of(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0u32, |acc, b| (acc << 8) | u32::from(*b))
}

pub(super) fn parse(data: &[u8]) -> CMap {
    let mut cmap = CMap::default();
    if let Some(pos) = super::object::find(data, b"begincodespacerange") {
        let mut lex = Lexer::new(data, pos + b"begincodespacerange".len());
        if let Some(obj) = lex.object()
            && let Some(low) = obj.as_str_bytes()
        {
            cmap.code_bytes = low.len();
        }
    }

    let mut from = 0usize;
    while let Some(rel) = super::object::find(&data[from..], b"beginbfchar") {
        let start = from + rel + b"beginbfchar".len();
        let end = super::object::find(&data[start..], b"endbfchar").map(|r| start + r);
        let Some(end) = end else { break };
        let mut lex = Lexer::new(&data[start..end], 0);
        while let Some(src) = lex.object() {
            let Some(dst) = lex.object() else { break };
            if let (Some(s), Some(d)) = (src.as_str_bytes(), dst.as_str_bytes()) {
                cmap.map.insert(code_of(s), utf16be(d));
            }
        }
        from = end;
    }

    let mut from = 0usize;
    while let Some(rel) = super::object::find(&data[from..], b"beginbfrange") {
        let start = from + rel + b"beginbfrange".len();
        let end = super::object::find(&data[start..], b"endbfrange").map(|r| start + r);
        let Some(end) = end else { break };
        let mut lex = Lexer::new(&data[start..end], 0);
        while let Some(lo) = lex.object() {
            let Some(hi) = lex.object() else { break };
            let Some(dst) = lex.object() else { break };
            let (Some(lo), Some(hi)) = (lo.as_str_bytes(), hi.as_str_bytes()) else {
                continue;
            };
            let (lo, hi) = (code_of(lo), code_of(hi));
            match &dst {
                d if d.as_str_bytes().is_some() => {
                    let base = d.as_str_bytes().unwrap_or_default();
                    for code in lo..=hi {
                        cmap.map.insert(code, shift_utf16(base, (code - lo) as u16));
                    }
                }
                d if d.as_array().is_some() => {
                    for (offset, item) in d.as_array().unwrap_or_default().iter().enumerate() {
                        if let Some(text) = item.as_str_bytes() {
                            cmap.map.insert(lo + offset as u32, utf16be(text));
                        }
                    }
                }
                _ => {}
            }
        }
        from = end;
    }
    cmap
}

fn shift_utf16(base: &[u8], delta: u16) -> String {
    let mut units: Vec<u16> = base
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_be_bytes(*p))
        .collect();
    if let Some(last) = units.last_mut() {
        *last = last.wrapping_add(delta);
    }
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bfchar_pairs() {
        let src = b"begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n2 beginbfchar\n<0003> <0041>\n<0004> <0042>\nendbfchar\n";
        let cmap = parse(src);
        assert_eq!(cmap.code_bytes(), 2);
        assert_eq!(cmap.lookup(3), Some("A"));
        assert_eq!(cmap.lookup(4), Some("B"));
    }

    #[test]
    fn parses_bfrange_with_incrementing_base() {
        let cmap = parse(b"1 beginbfrange\n<0010> <0012> <0061>\nendbfrange\n");
        assert_eq!(cmap.lookup(0x10), Some("a"));
        assert_eq!(cmap.lookup(0x11), Some("b"));
        assert_eq!(cmap.lookup(0x12), Some("c"));
    }

    #[test]
    fn parses_bfrange_with_explicit_array() {
        let cmap = parse(b"1 beginbfrange\n<0020> <0022> [<0058> <0059> <005A>]\nendbfrange\n");
        assert_eq!(cmap.lookup(0x20), Some("X"));
        assert_eq!(cmap.lookup(0x21), Some("Y"));
        assert_eq!(cmap.lookup(0x22), Some("Z"));
    }

    #[test]
    fn decodes_two_byte_codes() {
        let cmap = parse(b"1 beginbfrange\n<0001> <0003> <0048>\nendbfrange\n");
        assert_eq!(cmap.decode(&[0x00, 0x01, 0x00, 0x02, 0x00, 0x03]), "HIJ");
    }

    #[test]
    fn single_byte_codespace_is_honoured() {
        let src = b"begincodespacerange\n<00> <FF>\nendcodespacerange\n1 beginbfchar\n<41> <0041>\nendbfchar\n";
        let cmap = parse(src);
        assert_eq!(cmap.code_bytes(), 1);
        assert_eq!(cmap.decode(&[0x41, 0x41]), "AA");
    }

    #[test]
    fn unmapped_codes_contribute_nothing() {
        let cmap = parse(b"1 beginbfchar\n<0001> <0041>\nendbfchar\n");
        assert_eq!(cmap.decode(&[0x00, 0x01, 0x00, 0x99]), "A");
    }
}
