//! Property tests for quantizer invariants.

use proptest::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use tqvector::bench_api::{
    inverse_srht, pack_mse_indices, pack_qjl_signs, pad_input, payload_len, sign_vector, srht,
    transform_dim, unpack_mse_indices, unpack_qjl_signs, ProdQuantizer,
};

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

// P1: SRHT preserves L2 norm (isometry).
proptest! {
    #[test]
    fn srht_preserves_norm(dim in 2..512usize, seed in 0..1000u64) {
        let td = transform_dim(dim);
        let input = random_unit_vector(dim, seed + 1000);
        let padded = pad_input(&input, td);
        let signs = sign_vector(td, seed);
        let rotated = srht(&padded, &signs);

        let input_norm: f32 = padded.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm: f32 = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1e-10)).abs();
        prop_assert!(rel_err < 1e-4, "relative error = {rel_err}");
    }
}

// P2: SRHT is self-inverse: inverse_srht(srht(x)) ≈ x.
proptest! {
    #[test]
    fn srht_roundtrip(dim in 2..512usize, seed in 0..1000u64) {
        let td = transform_dim(dim);
        let input = random_unit_vector(dim, seed + 2000);
        let padded = pad_input(&input, td);
        let signs = sign_vector(td, seed);
        let rotated = srht(&padded, &signs);
        let recovered = inverse_srht(&rotated, &signs);

        for (a, b) in padded.iter().zip(recovered.iter()) {
            prop_assert!((a - b).abs() < 1e-4, "mismatch: {a} vs {b}");
        }
    }
}

// P3: MSE pack/unpack roundtrip for arbitrary dimensions and bit widths.
proptest! {
    #[test]
    fn mse_pack_unpack_roundtrip(
        dim in 1..2048usize,
        bits_per_index in 1..7u8,
        seed in 0..1000u64,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let max_val = 1u16 << bits_per_index;
        let indices: Vec<u16> = (0..dim).map(|_| rng.gen_range(0..max_val)).collect();
        let packed = pack_mse_indices(&indices, bits_per_index);
        let unpacked = unpack_mse_indices(&packed, dim, bits_per_index);
        prop_assert_eq!(unpacked, indices);
    }
}

// P4: QJL pack/unpack roundtrip.
proptest! {
    #[test]
    fn qjl_pack_unpack_roundtrip(dim in 1..4096usize, seed in 0..1000u64) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let signs: Vec<bool> = (0..dim).map(|_| rng.gen::<bool>()).collect();
        let packed = pack_qjl_signs(&signs);
        let unpacked = unpack_qjl_signs(&packed, dim);
        prop_assert_eq!(unpacked, signs);
    }
}

// P5: Encode determinism — same input always produces same output.
proptest! {
    #[test]
    fn encode_determinism(dim in prop::sample::select(&[32, 64, 128, 256][..]), seed in 0..100u64) {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let vector = random_unit_vector(dim, seed);

        let a = quantizer.pack_payload(&quantizer.encode(&vector));
        let b = quantizer.pack_payload(&quantizer.encode(&vector));
        prop_assert_eq!(a, b);
    }
}

// P6: score_ip_codes_lite is symmetric.
proptest! {
    #[test]
    fn score_ip_codes_lite_symmetric(dim in prop::sample::select(&[32, 64, 256][..]), seed in 0..100u64) {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);

        let enc_a = quantizer.encode(&random_unit_vector(dim, seed));
        let enc_b = quantizer.encode(&random_unit_vector(dim, seed + 1000));

        let mut code_a = enc_a.mse_packed.clone();
        code_a.extend_from_slice(&enc_a.qjl_packed);
        let mut code_b = enc_b.mse_packed.clone();
        code_b.extend_from_slice(&enc_b.qjl_packed);

        let ab = quantizer.score_ip_codes_lite(&code_a, &code_b);
        let ba = quantizer.score_ip_codes_lite(&code_b, &code_a);
        prop_assert_eq!(ab, ba);
    }
}

// P7: payload_len matches actual packed payload length.
proptest! {
    #[test]
    fn payload_len_matches_actual(
        dim in prop::sample::select(&[32, 64, 128, 256, 1536][..]),
        bits in prop::sample::select(&[2u8, 3, 4, 6, 8][..]),
    ) {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let vector = random_unit_vector(dim, 99);
        let actual = quantizer.pack_payload(&quantizer.encode(&vector));
        prop_assert_eq!(actual.len(), payload_len(dim, bits));
    }
}

// P8: score_ip_encoded == score_ip_from_parts for same data.
proptest! {
    #[test]
    fn score_consistency(dim in prop::sample::select(&[32, 64, 256][..]), seed in 0..100u64) {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let query = random_unit_vector(dim, seed);
        let candidate = quantizer.encode(&random_unit_vector(dim, seed + 500));
        let prepared = quantizer.prepare_ip_query(&query);

        let payload = quantizer.pack_payload(&candidate);
        let mut code_bytes = candidate.mse_packed.clone();
        code_bytes.extend_from_slice(&candidate.qjl_packed);

        let payload_score = quantizer.score_ip_encoded(&prepared, &payload);
        let parts_score = quantizer.score_ip_from_parts(&prepared, candidate.gamma, &code_bytes);

        prop_assert!(
            (payload_score - parts_score).abs() < 1e-6,
            "payload={payload_score}, parts={parts_score}"
        );
    }
}
