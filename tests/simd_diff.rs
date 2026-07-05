//! SIMD/scalar differential tests for quantizer hot paths.
//!
//! These tests intentionally call the dispatched production entry points and
//! bench-only scalar references in the same process. That keeps the comparison
//! independent of `ECAZ_SIMD` and catches host-reachable SIMD divergence.

#![cfg(feature = "bench")]

use ecaz::bench_api::{
    fwht_in_place, fwht_in_place_scalar_reference, pack_mse_indices, ProdQuantizer,
};
use proptest::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

fn random_bounded_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn code_bytes(encoded: &ecaz::bench_api::EncodedTq) -> Vec<u8> {
    let mut bytes = encoded.mse_packed.clone();
    bytes.extend_from_slice(&encoded.qjl_packed);
    bytes
}

fn assert_close(label: &str, dispatched: f32, scalar: f32, rel_tol: f32) {
    let abs_tol = rel_tol.max(rel_tol * scalar.abs().max(dispatched.abs()).max(1.0));
    assert!(
        (dispatched - scalar).abs() <= abs_tol,
        "{label}: dispatched={dispatched:.9}, scalar={scalar:.9}, abs_diff={:.9}, tol={abs_tol:.9}",
        (dispatched - scalar).abs()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn dispatched_score_ip_from_parts_matches_scalar_reference(
        dim in prop::sample::select(&[8usize, 16, 32, 64, 128, 384][..]),
        bits in prop_oneof![Just(3u8), Just(4u8), 2u8..=8],
        query_seed in 0u64..5000,
        candidate_seed in 5000u64..10000,
    ) {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let query = random_unit_vector(dim, query_seed);
        let candidate = quantizer.encode(&random_unit_vector(dim, candidate_seed));
        let prepared = quantizer.prepare_ip_query(&query);
        let codes = code_bytes(&candidate);

        let dispatched = quantizer.score_ip_from_parts(&prepared, candidate.gamma, &codes);
        let scalar = quantizer.score_ip_from_parts_scalar_reference(&prepared, candidate.gamma, &codes);
        assert_close("score_ip_from_parts", dispatched, scalar, 1.0e-5);
    }

    #[test]
    fn dispatched_code_to_code_score_matches_scalar_reference(
        dim in prop::sample::select(&[8usize, 16, 32, 64, 128, 384][..]),
        bits in prop_oneof![Just(3u8), Just(4u8), 2u8..=8],
        left_seed in 10000u64..15000,
        right_seed in 15000u64..20000,
    ) {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let left = code_bytes(&quantizer.encode(&random_unit_vector(dim, left_seed)));
        let right = code_bytes(&quantizer.encode(&random_unit_vector(dim, right_seed)));

        let dispatched = quantizer.score_ip_codes_lite(&left, &right);
        let scalar = quantizer.score_ip_codes_lite_scalar_reference(&left, &right);
        assert_close("score_ip_codes_lite", dispatched, scalar, 1.0e-5);
    }

    #[test]
    fn dispatched_fwht_matches_scalar_reference(
        len in prop::sample::select(&[4usize, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096][..]),
        seed in 20000u64..25000,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut dispatched = (0..len)
            .map(|_| rng.gen_range(-1.0f32..1.0f32))
            .collect::<Vec<_>>();
        let mut scalar = dispatched.clone();

        fwht_in_place(&mut dispatched);
        fwht_in_place_scalar_reference(&mut scalar);

        for (index, (actual, expected)) in dispatched.iter().zip(scalar.iter()).enumerate() {
            assert_close(&format!("fwht lane {index}"), *actual, *expected, 1.0e-5);
        }
    }

    #[test]
    fn packed_mse_indices_roundtrip_across_widths(
        dim in 1usize..513,
        bits in prop_oneof![Just(3u8), Just(4u8), 2u8..=8],
        seed in 25000u64..30000,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let max_index = 1u16 << bits;
        let indices = (0..dim)
            .map(|_| rng.gen_range(0..max_index))
            .collect::<Vec<_>>();
        let packed = pack_mse_indices(&indices, bits);
        let unpacked = ecaz::bench_api::unpack_mse_indices(&packed, indices.len(), bits);
        prop_assert_eq!(unpacked, indices);
    }

    #[test]
    fn am_source_inner_product_simd_matches_scalar_reference(
        dim in prop::sample::select(&[1usize, 3, 4, 7, 8, 15, 16, 31, 32, 33, 64, 127, 128, 384, 1536][..]),
        left_seed in 30000u64..35000,
        right_seed in 35000u64..40000,
    ) {
        let left = random_bounded_vector(dim, left_seed);
        let right = random_bounded_vector(dim, right_seed);

        let hnsw_scalar = ecaz::bench_api::hnsw_source_inner_product_scalar_reference(&left, &right);
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if let Some(hnsw_avx2) = ecaz::bench_api::hnsw_source_inner_product_avx2_fma_for_test(&left, &right) {
            assert_close("hnsw forced avx2 source inner product", hnsw_avx2, hnsw_scalar, 1.0e-4);
        }
        #[cfg(target_arch = "aarch64")]
        if let Some(hnsw_neon) = ecaz::bench_api::hnsw_source_inner_product_neon_for_test(&left, &right) {
            assert_close("hnsw forced neon source inner product", hnsw_neon, hnsw_scalar, 1.0e-4);
        }

        let diskann_scalar = ecaz::bench_api::diskann_source_inner_product_scalar_reference(&left, &right);
        #[cfg(target_arch = "x86_64")]
        if let Some(diskann_avx2) = ecaz::bench_api::diskann_source_inner_product_avx2_fma_for_test(&left, &right) {
            assert_close("diskann forced avx2 source inner product", diskann_avx2, diskann_scalar, 1.0e-4);
        }
        #[cfg(target_arch = "aarch64")]
        if let Some(diskann_neon) = ecaz::bench_api::diskann_source_inner_product_neon_for_test(&left, &right) {
            assert_close("diskann forced neon source inner product", diskann_neon, diskann_scalar, 1.0e-4);
        }
    }
}

#[test]
fn production_1536_4bit_score_path_matches_scalar_reference() {
    let quantizer = ProdQuantizer::new(1536, 4, 42);
    // Seeds are tied to the documented 1536/4 production baseline.
    let query = random_unit_vector(1536, 0x1536);
    let candidate = quantizer.encode(&random_unit_vector(1536, 0x4B17));
    let prepared = quantizer.prepare_ip_query(&query);
    let codes = code_bytes(&candidate);

    let dispatched = quantizer.score_ip_from_parts(&prepared, candidate.gamma, &codes);
    let scalar = quantizer.score_ip_from_parts_scalar_reference(&prepared, candidate.gamma, &codes);
    assert_close(
        "production 1536/4 score_ip_from_parts",
        dispatched,
        scalar,
        1.0e-5,
    );
}

#[cfg(target_arch = "x86_64")]
#[test]
fn forced_avx2_fma_score_paths_match_scalar_reference_when_available() {
    let quantizer = ProdQuantizer::new(384, 4, 42);
    let query = random_unit_vector(384, 0xA42);
    let candidate = quantizer.encode(&random_unit_vector(384, 0xB43));
    let prepared = quantizer.prepare_ip_query(&query);
    let codes = code_bytes(&candidate);

    if let Some(avx2) =
        quantizer.score_ip_from_parts_avx2_fma_for_test(&prepared, candidate.gamma, &codes)
    {
        let scalar =
            quantizer.score_ip_from_parts_scalar_reference(&prepared, candidate.gamma, &codes);
        assert_close("forced avx2 score_ip_from_parts", avx2, scalar, 1.0e-5);
    }

    let other = code_bytes(&quantizer.encode(&random_unit_vector(384, 0xC44)));
    if let Some(avx2) = quantizer.score_ip_codes_lite_avx2_fma_for_test(&codes, &other) {
        let scalar = quantizer.score_ip_codes_lite_scalar_reference(&codes, &other);
        assert_close("forced avx2 score_ip_codes_lite", avx2, scalar, 1.0e-5);
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn forced_neon_score_paths_match_scalar_reference_when_available() {
    let quantizer = ProdQuantizer::new(384, 4, 42);
    let query = random_unit_vector(384, 0xA42);
    let candidate = quantizer.encode(&random_unit_vector(384, 0xB43));
    let prepared = quantizer.prepare_ip_query(&query);
    let codes = code_bytes(&candidate);

    if let Some(neon) =
        quantizer.score_ip_from_parts_neon_for_test(&prepared, candidate.gamma, &codes)
    {
        let scalar =
            quantizer.score_ip_from_parts_scalar_reference(&prepared, candidate.gamma, &codes);
        assert_close("forced neon score_ip_from_parts", neon, scalar, 1.0e-5);
    }

    let other = code_bytes(&quantizer.encode(&random_unit_vector(384, 0xC44)));
    if let Some(neon) = quantizer.score_ip_codes_lite_neon_for_test(&codes, &other) {
        let scalar = quantizer.score_ip_codes_lite_scalar_reference(&codes, &other);
        assert_close("forced neon score_ip_codes_lite", neon, scalar, 1.0e-5);
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn forced_avx2_fwht_matches_scalar_reference_when_available() {
    let mut rng = ChaCha8Rng::seed_from_u64(0xF00D);
    let mut avx2 = (0..4096)
        .map(|_| rng.gen_range(-1.0f32..1.0f32))
        .collect::<Vec<_>>();
    let mut scalar = avx2.clone();

    if ecaz::bench_api::fwht_in_place_avx2_for_test(&mut avx2) {
        fwht_in_place_scalar_reference(&mut scalar);
        for (index, (actual, expected)) in avx2.iter().zip(scalar.iter()).enumerate() {
            assert_close(
                &format!("forced avx2 fwht lane {index}"),
                *actual,
                *expected,
                1.0e-5,
            );
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn forced_neon_fwht_matches_scalar_reference_when_available() {
    let mut rng = ChaCha8Rng::seed_from_u64(0xF00D);
    let mut neon = (0..4096)
        .map(|_| rng.gen_range(-1.0f32..1.0f32))
        .collect::<Vec<_>>();
    let mut scalar = neon.clone();

    if ecaz::bench_api::fwht_in_place_neon_for_test(&mut neon) {
        fwht_in_place_scalar_reference(&mut scalar);
        for (index, (actual, expected)) in neon.iter().zip(scalar.iter()).enumerate() {
            assert_close(
                &format!("forced neon fwht lane {index}"),
                *actual,
                *expected,
                1.0e-5,
            );
        }
    }
}

#[test]
fn three_bit_unpack_fixture_stays_bit_exact() {
    let indices = [0u16, 7, 3, 5, 1, 6, 2, 4, 4, 2, 6, 1, 5, 3, 7, 0];
    let packed = pack_mse_indices(&indices, 3);
    let unpacked = ecaz::bench_api::unpack_mse_indices(&packed, indices.len(), 3);
    assert_eq!(unpacked, indices);
}
