#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // Derive code_len from first byte to explore more state space
    let code_len = (data[0] as usize) * 4;
    let _ = tqvector::bench_api::TqElementTuple::decode(&data[1..], code_len);
});
