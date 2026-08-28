#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that the PDF parsers reachable without decryption never panic.
// RC4 key derivation gates whole-file extraction, so a mutated filing rarely
// reaches the object lexer, the ToUnicode CMap decoder, or the width table.
// Those take plain bytes and index into them directly.
fuzz_target!(|data: &[u8]| {
    finance_query::__fuzz_pdf_unencrypted(data);
});
