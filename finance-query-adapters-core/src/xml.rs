//! Byte-level helpers for the hand-rolled XML and feed extractors.

pub fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

pub fn find_byte(haystack: &[u8], from: usize, byte: u8) -> Option<usize> {
    if from > haystack.len() {
        return None;
    }
    haystack[from..]
        .iter()
        .position(|&b| b == byte)
        .map(|p| p + from)
}

pub fn trim_ascii(bytes: &[u8]) -> &[u8] {
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
pub fn unescape(raw: &[u8]) -> String {
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
