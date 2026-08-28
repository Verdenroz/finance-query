//! Minimal, dependency-free text extraction for House PTR disclosure PDFs.
//!
//! Every PTR filed through fd.house.gov's e-filing system is a PDF 1.4 with a
//! classic cross-reference table, RC4-128 encryption (`/V 2 /R 3`) under an
//! empty user password, and two or three embedded CIDFontType2 subsets that
//! each carry a `ToUnicode` CMap. Most are a single page; a filing with enough
//! transactions runs to several. Owning that narrow slice outright removes a
//! 31-crate dependency (`pdf-extract` and its `lopdf` font, cipher, and
//! parser-combinator stack, one of which was unmaintained under
//! RUSTSEC-2026-0192), the same trade already made for XML in `feeds::parser`.
//!
//! Anything outside that slice is reported rather than silently returning no
//! text, because zero rows is indistinguishable from a member disclosing no
//! trades. Older hand-signed PTRs are scanned images with no text layer at all;
//! those surface as [`PdfError::NoTextLayer`], and OCR is out of scope.

mod cmap;
mod crypto;
mod document;
mod font;
mod matrix;
mod object;
mod text;

use document::Document;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PdfError {
    NotAPdf,
    MissingEncryptDict,
    /// Encryption this extractor does not implement. House PTRs are `/V 2 /R 3`.
    UnsupportedEncryption {
        v: i64,
        r: i64,
    },
    /// Parsed cleanly but carries no text, which means a scanned filing.
    NoTextLayer,
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAPdf => f.write_str("not a PDF"),
            Self::MissingEncryptDict => {
                f.write_str("encryption dictionary is missing or malformed")
            }
            Self::UnsupportedEncryption { v, r } => {
                write!(
                    f,
                    "unsupported encryption (V {v}, R {r}); expected V 2, R 3"
                )
            }
            Self::NoTextLayer => f.write_str("no text layer, likely a scanned filing"),
        }
    }
}

pub(super) type Result<T> = std::result::Result<T, PdfError>;

/// Drive the parsers that read plain bytes, for `fuzz/fuzz_targets`.
///
/// Decryption gates [`extract_lines`], so mutating an encrypted filing mostly
/// dies at key derivation. The lexer, the CMap decoder, and the width table
/// take unencrypted input and are reachable directly.
#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_unencrypted(bytes: &[u8]) {
    let _ = object::scan_objects(bytes);
    let _ = object::trailer(bytes);
    let _ = cmap::parse(bytes);
    let mut lex = object::Lexer::new(bytes, 0);
    for _ in 0..256 {
        if lex.object().is_none() && lex.pos >= bytes.len() {
            break;
        }
    }
}

/// Text lines of a PTR, in reading order.
pub(crate) fn extract_lines(bytes: Vec<u8>) -> Result<Vec<String>> {
    let doc = Document::load(bytes)?;
    let lines = text::extract_lines(&doc);
    if lines.is_empty() {
        return Err(PdfError::NoTextLayer);
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanned_filing_is_distinguishable_from_an_empty_one() {
        let src =
            b"%PDF-1.4\n1 0 obj\n<< /Type /XObject /Subtype /Image >>\nendobj\ntrailer\n<< >>\n";
        assert_eq!(
            extract_lines(src.to_vec()).unwrap_err(),
            PdfError::NoTextLayer
        );
    }

    #[test]
    fn error_messages_name_the_cause() {
        assert_eq!(
            PdfError::UnsupportedEncryption { v: 4, r: 4 }.to_string(),
            "unsupported encryption (V 4, R 4); expected V 2, R 3"
        );
        assert_eq!(
            PdfError::NoTextLayer.to_string(),
            "no text layer, likely a scanned filing"
        );
    }
}
