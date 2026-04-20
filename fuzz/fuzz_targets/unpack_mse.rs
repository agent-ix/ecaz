#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    // First byte: dimension (1-255), second byte: bits_per_index (1-7)
    let dim = data[0] as usize;
    let bits_per_index = (data[1] % 7) + 1;
    if dim == 0 {
        return;
    }
    let packed = &data[2..];
    let expected_len = (dim * bits_per_index as usize).div_ceil(8);
    if packed.len() != expected_len {
        return;
    }
    let _ = ecaz::bench_api::unpack_mse_indices(packed, dim, bits_per_index);
});
