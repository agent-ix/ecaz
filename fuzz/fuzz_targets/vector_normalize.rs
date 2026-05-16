#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let floats: Vec<f32> = data
        .chunks_exact(4)
        .take(128)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("chunk length")))
        .collect();
    if floats.is_empty() || floats.iter().any(|value| !value.is_finite()) {
        return;
    }
    let bounded = floats
        .into_iter()
        .map(|value| value.clamp(-1.0, 1.0))
        .collect::<Vec<_>>();
    let quantizer = ecaz::bench_api::ProdQuantizer::new(bounded.len(), 4, 42);
    let _ = quantizer.encode(&bounded);
});
