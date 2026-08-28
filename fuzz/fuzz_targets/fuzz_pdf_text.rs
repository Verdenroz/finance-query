#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that House PTR text extraction never panics on arbitrary bytes.
// The parser walks untrusted PDF structure by hand: object and stream byte
// ranges, a content-stream operand stack, ToUnicode CMap ranges, and glyph
// width tables, all of which index into attacker-shaped input.
fuzz_target!(|data: &[u8]| {
    let _ = finance_query::__fuzz_pdf_extract_lines(data.to_vec());
});
