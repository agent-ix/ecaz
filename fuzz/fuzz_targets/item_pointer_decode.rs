#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = ecaz_fuzz::bench_api::ItemPointer::decode(data);
});
