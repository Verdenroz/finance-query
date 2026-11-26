#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that decoding arbitrary pricing frames never panics. Malformed
// base64 or protobuf must return Err.
fuzz_target!(|data: &[u8]| {
    finance_query::__fuzz_pricing(data);
});
