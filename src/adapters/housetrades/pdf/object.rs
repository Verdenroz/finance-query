//! PDF object syntax: a lexer for the subset House PTRs use.
//!
//! Objects are located by scanning for `N G obj` rather than by walking the
//! cross-reference table. A damaged or unusual xref is the most common reason
//! a PDF reader gives up on an otherwise readable file, and the scan does not
//! care.

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Object {
    Null,
    Bool(bool),
    Int(i64),
    Real(f64),
    Str(Vec<u8>),
    Name(String),
    Array(Vec<Object>),
    Dict(Dict),
    Ref(u32, u16),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Dict(Vec<(String, Object)>);

impl Dict {
    pub(super) fn get(&self, key: &str) -> Option<&Object> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub(super) fn entries(&self) -> impl Iterator<Item = (&str, &Object)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl Object {
    pub(super) fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            Self::Real(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub(super) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(n) => Some(*n as f64),
            Self::Real(f) => Some(*f),
            _ => None,
        }
    }

    pub(super) fn as_name(&self) -> Option<&str> {
        match self {
            Self::Name(n) => Some(n),
            _ => None,
        }
    }

    pub(super) fn as_str_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(super) fn as_array(&self) -> Option<&[Object]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    pub(super) fn as_ref_id(&self) -> Option<(u32, u16)> {
        match self {
            Self::Ref(n, g) => Some((*n, *g)),
            _ => None,
        }
    }
}

fn is_ws(b: u8) -> bool {
    matches!(b, b'\0' | b'\t' | b'\n' | 0x0c | b'\r' | b' ')
}

fn is_delim(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

pub(crate) struct Lexer<'a> {
    buf: &'a [u8],
    pub(super) pos: usize,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(buf: &'a [u8], pos: usize) -> Self {
        Self { buf, pos }
    }

    fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    pub(super) fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if is_ws(b) {
                self.pos += 1;
            } else if b == b'%' {
                while self.peek().is_some_and(|c| c != b'\n' && c != b'\r') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    pub(super) fn peek_byte(&self) -> Option<u8> {
        self.peek()
    }

    /// Read a bare keyword, for content-stream operators.
    pub(super) fn operator(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self.peek().is_some_and(|b| !is_ws(b) && !is_delim(b)) {
            self.pos += 1;
        }
        if self.pos == start {
            self.pos += 1;
        }
        &self.buf[start..self.pos]
    }

    fn token(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self.peek().is_some_and(|b| !is_ws(b) && !is_delim(b)) {
            self.pos += 1;
        }
        &self.buf[start..self.pos]
    }

    fn name(&mut self) -> String {
        self.pos += 1;
        let raw = self.token();
        let mut out = String::with_capacity(raw.len());
        let mut i = 0;
        while i < raw.len() {
            if raw[i] == b'#' && i + 2 < raw.len() {
                let hi = (raw[i + 1] as char).to_digit(16);
                let lo = (raw[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push(((hi * 16 + lo) as u8) as char);
                    i += 3;
                    continue;
                }
            }
            out.push(raw[i] as char);
            i += 1;
        }
        out
    }

    fn literal_string(&mut self) -> Vec<u8> {
        self.pos += 1;
        let mut out = Vec::new();
        let mut depth = 1;
        while let Some(b) = self.peek() {
            self.pos += 1;
            match b {
                b'\\' => {
                    let Some(esc) = self.peek() else { break };
                    self.pos += 1;
                    match esc {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'\n' => {}
                        b'\r' => {
                            if self.peek() == Some(b'\n') {
                                self.pos += 1;
                            }
                        }
                        b'0'..=b'7' => {
                            let mut v = u32::from(esc - b'0');
                            for _ in 0..2 {
                                match self.peek() {
                                    Some(d @ b'0'..=b'7') => {
                                        v = v * 8 + u32::from(d - b'0');
                                        self.pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            out.push(v as u8);
                        }
                        other => out.push(other),
                    }
                }
                b'(' => {
                    depth += 1;
                    out.push(b);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    out.push(b);
                }
                _ => out.push(b),
            }
        }
        out
    }

    fn hex_string(&mut self) -> Vec<u8> {
        self.pos += 1;
        let mut digits = Vec::new();
        while let Some(b) = self.peek() {
            self.pos += 1;
            if b == b'>' {
                break;
            }
            if let Some(d) = (b as char).to_digit(16) {
                digits.push(d as u8);
            }
        }
        if digits.len() % 2 == 1 {
            digits.push(0);
        }
        digits
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| p[0] * 16 + p[1])
            .collect()
    }

    pub(super) fn object(&mut self) -> Option<Object> {
        self.skip_ws();
        let b = self.peek()?;
        match b {
            b'/' => Some(Object::Name(self.name())),
            b'(' => Some(Object::Str(self.literal_string())),
            b'[' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    match self.peek() {
                        None => break,
                        Some(b']') => {
                            self.pos += 1;
                            break;
                        }
                        _ => match self.object() {
                            Some(o) => items.push(o),
                            None => break,
                        },
                    }
                }
                Some(Object::Array(items))
            }
            b'<' => {
                if self.buf.get(self.pos + 1) == Some(&b'<') {
                    self.pos += 2;
                    let mut entries = Vec::new();
                    loop {
                        self.skip_ws();
                        match self.peek() {
                            None => break,
                            Some(b'>') => {
                                self.pos += 1;
                                if self.peek() == Some(b'>') {
                                    self.pos += 1;
                                }
                                break;
                            }
                            Some(b'/') => {
                                let key = self.name();
                                let Some(value) = self.object() else { break };
                                entries.push((key, value));
                            }
                            _ => {
                                self.pos += 1;
                            }
                        }
                    }
                    Some(Object::Dict(Dict(entries)))
                } else {
                    Some(Object::Str(self.hex_string()))
                }
            }
            b']' | b'>' | b')' | b'}' => None,
            _ => {
                let save = self.pos;
                let tok = self.token();
                if tok.is_empty() {
                    self.pos += 1;
                    return None;
                }
                match tok {
                    b"true" => return Some(Object::Bool(true)),
                    b"false" => return Some(Object::Bool(false)),
                    b"null" => return Some(Object::Null),
                    _ => {}
                }
                let text = std::str::from_utf8(tok).ok()?;
                if let Ok(n) = text.parse::<i64>() {
                    let after_int = self.pos;
                    self.skip_ws();
                    let gen_text = self.token();
                    if let Ok(generation) =
                        std::str::from_utf8(gen_text).unwrap_or("x").parse::<u16>()
                    {
                        self.skip_ws();
                        if self.peek() == Some(b'R')
                            && self
                                .buf
                                .get(self.pos + 1)
                                .is_none_or(|c| is_ws(*c) || is_delim(*c))
                        {
                            self.pos += 1;
                            return Some(Object::Ref(n as u32, generation));
                        }
                    }
                    self.pos = after_int;
                    return Some(Object::Int(n));
                }
                if let Ok(f) = text.parse::<f64>() {
                    return Some(Object::Real(f));
                }
                self.pos = save + tok.len().max(1);
                None
            }
        }
    }
}

pub(super) struct RawObject {
    pub(super) id: (u32, u16),
    pub(super) dict: Dict,
    pub(super) stream: Option<(usize, usize)>,
}

/// Every `N G obj` in the file, with the byte range of its stream payload.
pub(super) fn scan_objects(buf: &[u8]) -> Vec<RawObject> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(rel) = find(&buf[i..], b" obj") {
        let kw = i + rel;
        i = kw + 4;
        let Some((num, generation)) = header_before(buf, kw) else {
            continue;
        };
        let mut lex = Lexer::new(buf, kw + 4);
        let Some(obj) = lex.object() else { continue };
        let dict = match obj {
            Object::Dict(d) => d,
            _ => continue,
        };
        let stream = stream_range(buf, &mut lex);
        out.push(RawObject {
            id: (num, generation),
            dict,
            stream,
        });
    }
    out
}

fn stream_range(buf: &[u8], lex: &mut Lexer<'_>) -> Option<(usize, usize)> {
    lex.skip_ws();
    if !buf[lex.pos..].starts_with(b"stream") {
        return None;
    }
    let mut start = lex.pos + 6;
    if buf.get(start) == Some(&b'\r') {
        start += 1;
    }
    if buf.get(start) == Some(&b'\n') {
        start += 1;
    }
    let end = find(&buf[start..], b"endstream").map(|r| start + r)?;
    Some((start, end))
}

fn header_before(buf: &[u8], kw: usize) -> Option<(u32, u16)> {
    let lo = kw.saturating_sub(24);
    let head = &buf[lo..kw];
    let text = String::from_utf8_lossy(head);
    let mut parts = text.split_whitespace().rev();
    let generation = parts.next()?.parse::<u16>().ok()?;
    let num = parts.next()?.parse::<u32>().ok()?;
    Some((num, generation))
}

pub(super) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// The last trailer dictionary in the file.
pub(super) fn trailer(buf: &[u8]) -> Option<Dict> {
    let mut at = None;
    let mut from = 0;
    while let Some(rel) = find(&buf[from..], b"trailer") {
        at = Some(from + rel);
        from += rel + 7;
    }
    let mut lex = Lexer::new(buf, at? + 7);
    match lex.object()? {
        Object::Dict(d) => Some(d),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &[u8]) -> Object {
        Lexer::new(src, 0).object().expect("parses")
    }

    #[test]
    fn parses_nested_dictionary() {
        let o = parse(b"<< /Type /Page /Count 3 /Kids [1 0 R 2 0 R] >>");
        let Object::Dict(d) = o else {
            panic!("expected dict")
        };
        assert_eq!(d.get("Type").unwrap().as_name(), Some("Page"));
        assert_eq!(d.get("Count").unwrap().as_i64(), Some(3));
        let kids = d.get("Kids").unwrap().as_array().unwrap();
        assert_eq!(kids[0].as_ref_id(), Some((1, 0)));
        assert_eq!(kids[1].as_ref_id(), Some((2, 0)));
    }

    #[test]
    fn distinguishes_reference_from_two_integers() {
        let arr = parse(b"[1 0 R 4 5]");
        let items = arr.as_array().unwrap();
        assert_eq!(items[0].as_ref_id(), Some((1, 0)));
        assert_eq!(items[1].as_i64(), Some(4));
        assert_eq!(items[2].as_i64(), Some(5));
    }

    #[test]
    fn parses_hex_and_literal_strings() {
        assert_eq!(parse(b"<48656C6C6F>").as_str_bytes(), Some(&b"Hello"[..]));
        assert_eq!(parse(b"(a\\(b\\)c)").as_str_bytes(), Some(&b"a(b)c"[..]));
        assert_eq!(parse(b"(oct\\101)").as_str_bytes(), Some(&b"octA"[..]));
        assert_eq!(parse(b"(nest(ed))").as_str_bytes(), Some(&b"nest(ed)"[..]));
    }

    #[test]
    fn odd_length_hex_string_pads_with_zero() {
        assert_eq!(parse(b"<4A6>").as_str_bytes(), Some(&[0x4a, 0x60][..]));
    }

    #[test]
    fn decodes_name_escapes() {
        assert_eq!(parse(b"/A#20B").as_name(), Some("A B"));
    }

    #[test]
    fn parses_reals_and_negatives() {
        assert_eq!(parse(b"-3.5").as_f64(), Some(-3.5));
        assert_eq!(parse(b"42").as_i64(), Some(42));
    }

    #[test]
    fn scans_objects_and_stream_ranges() {
        let src = b"%PDF-1.4\n1 0 obj\n<< /Length 5 >>\nstream\nHELLO\nendstream\nendobj\n2 0 obj\n<< /Type /Catalog >>\nendobj\n";
        let objs = scan_objects(src);
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].id, (1, 0));
        let (s, e) = objs[0].stream.unwrap();
        assert_eq!(&src[s..e], b"HELLO\n");
        assert_eq!(objs[1].id, (2, 0));
        assert!(objs[1].stream.is_none());
    }

    #[test]
    fn reads_last_trailer() {
        let src = b"trailer\n<< /Size 3 >>\nstartxref\ntrailer\n<< /Size 9 /Encrypt 26 0 R >>\nstartxref\n";
        let t = trailer(src).unwrap();
        assert_eq!(t.get("Size").unwrap().as_i64(), Some(9));
        assert_eq!(t.get("Encrypt").unwrap().as_ref_id(), Some((26, 0)));
    }
}
