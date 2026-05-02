//! Shared internal helpers for adapter modules.

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// Characters that must be percent-encoded inside a URL path segment.
///
/// We use a conservative set: the unreserved characters per RFC 3986
/// (`ALPHA / DIGIT / "-" / "." / "_" / "~"`) are left as-is, everything
/// else — including sub-delims, gen-delims, and `/` — is encoded. Notably
/// this does NOT apply dot-segment resolution, so `".."` survives as
/// literal `..` rather than collapsing the URL path.
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',');

/// Percent-encode a string for safe inclusion as a URL path segment.
///
/// Encodes characters that would otherwise alter URL structure: `?`, `#`,
/// `/`, whitespace, etc. Unlike `url::Url::path_segments_mut().push`, this
/// does NOT apply RFC 3986 dot-segment removal, so a malicious input of
/// `".."` is preserved literally and cannot collapse a path component.
///
/// Use whenever a user-supplied symbol/ticker/CIK is interpolated into a
/// URL path via `format!`.
#[allow(dead_code)] // used by fmp and polygon adapter modules (tasks 5 & 6)
pub(crate) fn encode_path_segment(segment: &str) -> String {
    utf8_percent_encode(segment, PATH_SEGMENT_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_symbol_unchanged() {
        assert_eq!(encode_path_segment("AAPL"), "AAPL");
    }

    #[test]
    fn dot_separated_ticker_unchanged() {
        assert_eq!(encode_path_segment("BRK.B"), "BRK.B");
    }

    #[test]
    fn question_mark_is_encoded() {
        assert_eq!(encode_path_segment("FOO?bar"), "FOO%3Fbar");
    }

    #[test]
    fn hash_is_encoded() {
        assert_eq!(encode_path_segment("FOO#bar"), "FOO%23bar");
    }

    #[test]
    fn slash_is_encoded() {
        assert_eq!(encode_path_segment("a/b"), "a%2Fb");
    }

    #[test]
    fn space_is_encoded() {
        assert_eq!(encode_path_segment("a b"), "a%20b");
    }

    #[test]
    fn dot_dot_is_preserved_literally() {
        // Critical: must NOT collapse to empty (no dot-segment removal).
        // The literal ".." characters are unreserved, so they pass through.
        assert_eq!(encode_path_segment(".."), "..");
    }

    #[test]
    fn dot_dot_slash_is_encoded() {
        // The slash must be encoded so a malicious "../foo" cannot
        // navigate a path component upward in the resulting URL.
        assert_eq!(encode_path_segment("../foo"), "..%2Ffoo");
    }
}
