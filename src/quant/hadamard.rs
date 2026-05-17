//! Fast Walsh-Hadamard Transform primitives.

use crate::quant::simd::{backend, SimdBackend};

/// In-place unnormalized FWHT.
///
/// The input length must be a power of two.
pub fn fwht_in_place(values: &mut [f32]) {
    assert!(
        values.len().is_power_of_two(),
        "fwht input length must be a power of two, got {}",
        values.len()
    );

    match backend() {
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx2Fma => unsafe { fwht_in_place_avx2(values) },
        #[cfg(target_arch = "aarch64")]
        SimdBackend::Neon => unsafe { fwht_in_place_neon(values) },
        SimdBackend::Scalar => fwht_in_place_scalar(values),
    }
}

pub(crate) fn fwht_in_place_scalar(values: &mut [f32]) {
    let mut width = 1;
    while width < values.len() {
        let step = width * 2;
        for chunk in values.chunks_exact_mut(step) {
            let (left, right) = chunk.split_at_mut(width);
            for i in 0..width {
                let a = left[i];
                let b = right[i];
                left[i] = a + b;
                right[i] = a - b;
            }
        }
        width = step;
    }
}

#[cfg(any(test, feature = "bench"))]
pub fn fwht_in_place_scalar_reference(values: &mut [f32]) {
    fwht_in_place_scalar(values);
}

#[cfg(all(any(test, feature = "bench"), target_arch = "x86_64"))]
pub fn fwht_in_place_avx2_for_test(values: &mut [f32]) -> bool {
    if !std::arch::is_x86_feature_detected!("avx2") || !std::arch::is_x86_feature_detected!("fma") {
        return false;
    }
    unsafe { fwht_in_place_avx2(values) };
    true
}

#[cfg(all(any(test, feature = "bench"), target_arch = "aarch64"))]
pub fn fwht_in_place_neon_for_test(values: &mut [f32]) -> bool {
    if !std::arch::is_aarch64_feature_detected!("neon") {
        return false;
    }
    unsafe { fwht_in_place_neon(values) };
    true
}

/// In-place tiled FWHT over equal-sized power-of-two chunks.
pub fn fwht_tiled_in_place(values: &mut [f32], tile_size: usize) {
    assert!(
        tile_size.is_power_of_two(),
        "fwht tile size must be a power of two, got {}",
        tile_size
    );
    assert!(
        values.len() % tile_size == 0,
        "fwht tiled input length {} must be divisible by tile size {}",
        values.len(),
        tile_size
    );
    for chunk in values.chunks_exact_mut(tile_size) {
        fwht_in_place(chunk);
    }
}

/// Apply the orthonormalized FWHT in place.
pub fn orthonormal_fwht_in_place(values: &mut [f32]) {
    fwht_in_place(values);
    let scale = (values.len() as f32).sqrt().recip();
    scale_in_place_scalar(values, scale);
}

fn scale_in_place_scalar(values: &mut [f32], scale: f32) {
    for value in values {
        *value *= scale;
    }
}

/// Apply an orthonormal tiled FWHT in place.
pub fn orthonormal_fwht_tiled_in_place(values: &mut [f32], tile_size: usize) {
    fwht_tiled_in_place(values, tile_size);
    let scale = (tile_size as f32).sqrt().recip();
    for value in values {
        *value *= scale;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn fwht_in_place_avx2(values: &mut [f32]) {
    if values.len() == 1024 {
        unsafe { fwht_in_place_avx2_two_level(values, 512, 256) };
        return;
    }

    if values.len() == 2048 {
        unsafe { fwht_in_place_avx2_two_level(values, 1024, 256) };
        return;
    }

    if values.len() == 4096 {
        unsafe { fwht_in_place_avx2_two_level(values, 2048, 256) };
        return;
    }

    let tile_width = avx2_fwht_tile_width(values.len());
    if tile_width > 0 {
        for chunk in values.chunks_exact_mut(tile_width) {
            unsafe { fwht_in_place_avx2_bootstrap(chunk) };
            unsafe { fwht_in_place_avx2_stages(chunk, 64) };
        }
        unsafe { fwht_in_place_avx2_stages(values, tile_width) };
        return;
    }

    let bootstrap_width = unsafe { fwht_in_place_avx2_bootstrap(values) };
    unsafe { fwht_in_place_avx2_stages(values, bootstrap_width) };
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht_in_place_avx2_two_level(
    values: &mut [f32],
    outer_tile_width: usize,
    inner_tile_width: usize,
) {
    debug_assert!(outer_tile_width.is_power_of_two());
    debug_assert!(inner_tile_width.is_power_of_two());
    debug_assert!(inner_tile_width >= 64);
    debug_assert!(outer_tile_width > inner_tile_width);
    debug_assert_eq!(values.len() % outer_tile_width, 0);
    debug_assert_eq!(outer_tile_width % inner_tile_width, 0);

    for outer_chunk in values.chunks_exact_mut(outer_tile_width) {
        for inner_chunk in outer_chunk.chunks_exact_mut(inner_tile_width) {
            unsafe { fwht_in_place_avx2_bootstrap(inner_chunk) };
            unsafe { fwht_in_place_avx2_stages(inner_chunk, 64) };
        }
        unsafe { fwht_in_place_avx2_stages(outer_chunk, inner_tile_width) };
    }

    unsafe { fwht_in_place_avx2_stages(values, outer_tile_width) };
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn avx2_fwht_tile_width(len: usize) -> usize {
    if len == 2048 {
        256
    } else if len >= 512 {
        512
    } else if len >= 256 {
        256
    } else if len >= 128 {
        128
    } else {
        0
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht_in_place_avx2_bootstrap(values: &mut [f32]) -> usize {
    use std::arch::x86_64::{_mm256_loadu_ps, _mm256_storeu_ps};

    if values.len() < 8 {
        fwht_in_place_scalar(values);
        4
    } else if values.len() < 16 {
        for chunk in values.chunks_exact_mut(8) {
            let block = unsafe { _mm256_loadu_ps(chunk.as_ptr()) };
            let transformed = unsafe { fwht8_avx2_block(block) };
            unsafe { _mm256_storeu_ps(chunk.as_mut_ptr(), transformed) };
        }
        8
    } else if values.len() < 32 {
        for chunk in values.chunks_exact_mut(16) {
            let left = unsafe { _mm256_loadu_ps(chunk.as_ptr()) };
            let right = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(8)) };
            let (sum, diff) = unsafe { fwht16_avx2_block(left, right) };
            unsafe {
                _mm256_storeu_ps(chunk.as_mut_ptr(), sum);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(8), diff);
            }
        }
        16
    } else if values.len() < 64 {
        for chunk in values.chunks_exact_mut(32) {
            let a0 = unsafe { _mm256_loadu_ps(chunk.as_ptr()) };
            let a1 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(8)) };
            let b0 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(16)) };
            let b1 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(24)) };
            let (sum0, sum1, diff0, diff1) = unsafe { fwht32_avx2_block(a0, a1, b0, b1) };
            unsafe {
                _mm256_storeu_ps(chunk.as_mut_ptr(), sum0);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(8), sum1);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(16), diff0);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(24), diff1);
            }
        }
        32
    } else {
        for chunk in values.chunks_exact_mut(64) {
            let a0 = unsafe { _mm256_loadu_ps(chunk.as_ptr()) };
            let a1 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(8)) };
            let a2 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(16)) };
            let a3 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(24)) };
            let b0 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(32)) };
            let b1 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(40)) };
            let b2 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(48)) };
            let b3 = unsafe { _mm256_loadu_ps(chunk.as_ptr().add(56)) };
            let (sum0, sum1, sum2, sum3, diff0, diff1, diff2, diff3) =
                unsafe { fwht64_avx2_block(a0, a1, a2, a3, b0, b1, b2, b3) };
            unsafe {
                _mm256_storeu_ps(chunk.as_mut_ptr(), sum0);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(8), sum1);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(16), sum2);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(24), sum3);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(32), diff0);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(40), diff1);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(48), diff2);
                _mm256_storeu_ps(chunk.as_mut_ptr().add(56), diff3);
            }
        }
        64
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht_in_place_avx2_stages(values: &mut [f32], mut width: usize) {
    while width < values.len() {
        unsafe { fwht_in_place_avx2_stage_width(values, width) };
        width *= 2;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht_in_place_avx2_stage_width(values: &mut [f32], width: usize) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_loadu_ps, _mm256_storeu_ps, _mm256_sub_ps};

    let step = width * 2;
    let len = values.len();
    let ptr = values.as_mut_ptr();
    debug_assert_eq!(len % step, 0);

    if width >= 8 {
        debug_assert_eq!(width % 8, 0);

        let mut offset = 0;
        while offset < len {
            let left = unsafe { ptr.add(offset) };
            let right = unsafe { left.add(width) };
            let mut i = 0;
            while i < width {
                let a = unsafe { _mm256_loadu_ps(left.add(i)) };
                let b = unsafe { _mm256_loadu_ps(right.add(i)) };
                let sum = _mm256_add_ps(a, b);
                let diff = _mm256_sub_ps(a, b);
                unsafe {
                    _mm256_storeu_ps(left.add(i), sum);
                    _mm256_storeu_ps(right.add(i), diff);
                }
                i += 8;
            }

            offset += step;
        }
        return;
    }

    let mut offset = 0;
    while offset < len {
        let left = unsafe { ptr.add(offset) };
        let right = unsafe { left.add(width) };
        let mut i = 0;
        while i < width {
            unsafe {
                let a = *left.add(i);
                let b = *right.add(i);
                *left.add(i) = a + b;
                *right.add(i) = a - b;
            }
            i += 1;
        }

        offset += step;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht8_avx2_block(block: std::arch::x86_64::__m256) -> std::arch::x86_64::__m256 {
    use std::arch::x86_64::{
        _mm256_add_ps, _mm256_blend_ps, _mm256_permutevar8x32_ps, _mm256_setr_epi32, _mm256_sub_ps,
    };

    let swap1 = _mm256_setr_epi32(1, 0, 3, 2, 5, 4, 7, 6);
    let swap2 = _mm256_setr_epi32(2, 3, 0, 1, 6, 7, 4, 5);
    let swap3 = _mm256_setr_epi32(4, 5, 6, 7, 0, 1, 2, 3);

    let mut value = block;

    let paired = _mm256_permutevar8x32_ps(value, swap1);
    let paired_sum = _mm256_add_ps(value, paired);
    let paired_diff = _mm256_permutevar8x32_ps(_mm256_sub_ps(value, paired), swap1);
    value = _mm256_blend_ps(paired_sum, paired_diff, 0b1010_1010);

    let quads = _mm256_permutevar8x32_ps(value, swap2);
    let quad_sum = _mm256_add_ps(value, quads);
    let quad_diff = _mm256_permutevar8x32_ps(_mm256_sub_ps(value, quads), swap2);
    value = _mm256_blend_ps(quad_sum, quad_diff, 0b1100_1100);

    let halves = _mm256_permutevar8x32_ps(value, swap3);
    let half_sum = _mm256_add_ps(value, halves);
    let half_diff = _mm256_permutevar8x32_ps(_mm256_sub_ps(value, halves), swap3);
    _mm256_blend_ps(half_sum, half_diff, 0b1111_0000)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht16_avx2_block(
    left: std::arch::x86_64::__m256,
    right: std::arch::x86_64::__m256,
) -> (std::arch::x86_64::__m256, std::arch::x86_64::__m256) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_sub_ps};

    let left = unsafe { fwht8_avx2_block(left) };
    let right = unsafe { fwht8_avx2_block(right) };
    (_mm256_add_ps(left, right), _mm256_sub_ps(left, right))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht32_avx2_block(
    a0: std::arch::x86_64::__m256,
    a1: std::arch::x86_64::__m256,
    b0: std::arch::x86_64::__m256,
    b1: std::arch::x86_64::__m256,
) -> (
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_sub_ps};

    let (a0, a1) = unsafe { fwht16_avx2_block(a0, a1) };
    let (b0, b1) = unsafe { fwht16_avx2_block(b0, b1) };
    (
        _mm256_add_ps(a0, b0),
        _mm256_add_ps(a1, b1),
        _mm256_sub_ps(a0, b0),
        _mm256_sub_ps(a1, b1),
    )
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fwht64_avx2_block(
    a0: std::arch::x86_64::__m256,
    a1: std::arch::x86_64::__m256,
    a2: std::arch::x86_64::__m256,
    a3: std::arch::x86_64::__m256,
    b0: std::arch::x86_64::__m256,
    b1: std::arch::x86_64::__m256,
    b2: std::arch::x86_64::__m256,
    b3: std::arch::x86_64::__m256,
) -> (
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
    std::arch::x86_64::__m256,
) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_sub_ps};

    let (a0, a1, a2, a3) = unsafe { fwht32_avx2_block(a0, a1, a2, a3) };
    let (b0, b1, b2, b3) = unsafe { fwht32_avx2_block(b0, b1, b2, b3) };
    (
        _mm256_add_ps(a0, b0),
        _mm256_add_ps(a1, b1),
        _mm256_add_ps(a2, b2),
        _mm256_add_ps(a3, b3),
        _mm256_sub_ps(a0, b0),
        _mm256_sub_ps(a1, b1),
        _mm256_sub_ps(a2, b2),
        _mm256_sub_ps(a3, b3),
    )
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn fwht_in_place_neon(values: &mut [f32]) {
    use std::arch::aarch64::{vaddq_f32, vld1q_f32, vst1q_f32, vsubq_f32};

    let mut width = 1;
    while width < values.len() {
        let step = width * 2;
        for chunk in values.chunks_exact_mut(step) {
            let (left, right) = chunk.split_at_mut(width);
            let mut i = 0;
            while i + 4 <= width {
                let a = unsafe { vld1q_f32(left.as_ptr().add(i)) };
                let b = unsafe { vld1q_f32(right.as_ptr().add(i)) };
                let sum = vaddq_f32(a, b);
                let diff = vsubq_f32(a, b);
                unsafe {
                    vst1q_f32(left.as_mut_ptr().add(i), sum);
                    vst1q_f32(right.as_mut_ptr().add(i), diff);
                }
                i += 4;
            }
            while i < width {
                let a = left[i];
                let b = right[i];
                left[i] = a + b;
                right[i] = a - b;
                i += 1;
            }
        }
        width = step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn fwht_preserves_norm_after_normalization() {
        let input = vec![1.0_f32, -2.0, 0.5, 3.25, -1.5, 4.0, 2.0, -0.25];
        let mut rotated = input.clone();
        orthonormal_fwht_in_place(&mut rotated);

        let input_norm = input.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }

    #[test]
    fn fwht_runtime_path_matches_scalar_on_random_inputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..1_000 {
            let size = 1usize << rng.gen_range(1..=9);
            let mut scalar = (0..size)
                .map(|_| rng.gen_range(-10.0_f32..10.0_f32))
                .collect::<Vec<_>>();
            let mut dispatched = scalar.clone();

            fwht_in_place_scalar(&mut scalar);
            fwht_in_place(&mut dispatched);

            for (lhs, rhs) in scalar.iter().zip(dispatched.iter()) {
                let scale = lhs.abs().max(rhs.abs()).max(1.0);
                assert!(
                    ((lhs - rhs) / scale).abs() < 1e-6,
                    "lhs={lhs} rhs={rhs} size={size}"
                );
            }
        }
    }

    #[test]
    fn orthonormal_fwht_runtime_path_matches_scalar_on_random_inputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(43);
        for _ in 0..1_000 {
            let size = 1usize << rng.gen_range(1..=9);
            let mut scalar = (0..size)
                .map(|_| rng.gen_range(-10.0_f32..10.0_f32))
                .collect::<Vec<_>>();
            let mut dispatched = scalar.clone();

            fwht_in_place_scalar(&mut scalar);
            scale_in_place_scalar(&mut scalar, (size as f32).sqrt().recip());
            orthonormal_fwht_in_place(&mut dispatched);

            for (lhs, rhs) in scalar.iter().zip(dispatched.iter()) {
                let scale = lhs.abs().max(rhs.abs()).max(1.0);
                assert!(
                    ((lhs - rhs) / scale).abs() < 1e-6,
                    "lhs={lhs} rhs={rhs} size={size}"
                );
            }
        }
    }

    #[test]
    fn miri_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0];
        fwht_in_place(&mut data);
    }

    #[test]
    fn miri_orthonormal_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0, -1.5, 4.0, 2.0, -0.25];
        orthonormal_fwht_in_place(&mut data);
    }

    #[test]
    fn tiled_fwht_matches_chunkwise_full_fwht() {
        let mut tiled = (0..16).map(|i| i as f32 * 0.25 - 1.0).collect::<Vec<_>>();
        let mut chunkwise = tiled.clone();

        fwht_tiled_in_place(&mut tiled, 8);
        for chunk in chunkwise.chunks_exact_mut(8) {
            fwht_in_place(chunk);
        }

        assert_eq!(tiled, chunkwise);
    }

    #[test]
    fn tiled_orthonormal_fwht_preserves_norm_per_tile() {
        let input = (0..1536)
            .map(|i| (i as f32 * 0.001).sin())
            .collect::<Vec<_>>();
        let mut rotated = input.clone();
        orthonormal_fwht_tiled_in_place(&mut rotated, 512);

        let input_norm = input.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn fwht8_avx2_block_matches_scalar_when_available() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar = vec![1.0f32, -2.0, 0.5, 3.0, -1.5, 4.0, 2.0, -0.25];
        let mut avx = scalar.clone();
        fwht_in_place_scalar(&mut scalar);

        unsafe {
            use std::arch::x86_64::{_mm256_loadu_ps, _mm256_storeu_ps};

            let transformed = fwht8_avx2_block(_mm256_loadu_ps(avx.as_ptr()));
            _mm256_storeu_ps(avx.as_mut_ptr(), transformed);
        }

        assert_eq!(scalar, avx);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn fwht16_avx2_block_matches_scalar_when_available() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar = vec![
            1.0f32, -2.0, 0.5, 3.0, -1.5, 4.0, 2.0, -0.25, 0.75, -3.5, 1.25, 2.5, -4.5, 5.0, -0.75,
            1.5,
        ];
        let mut avx = scalar.clone();
        fwht_in_place_scalar(&mut scalar);

        unsafe {
            use std::arch::x86_64::{_mm256_loadu_ps, _mm256_storeu_ps};

            let left = _mm256_loadu_ps(avx.as_ptr());
            let right = _mm256_loadu_ps(avx.as_ptr().add(8));
            let (sum, diff) = fwht16_avx2_block(left, right);
            _mm256_storeu_ps(avx.as_mut_ptr(), sum);
            _mm256_storeu_ps(avx.as_mut_ptr().add(8), diff);
        }

        assert_eq!(scalar, avx);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn fwht32_avx2_block_matches_scalar_when_available() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar = (0..32)
            .map(|index| (index as f32 * 0.25) - 3.5)
            .collect::<Vec<_>>();
        let mut avx = scalar.clone();
        fwht_in_place_scalar(&mut scalar);

        unsafe {
            use std::arch::x86_64::{_mm256_loadu_ps, _mm256_storeu_ps};

            let a0 = _mm256_loadu_ps(avx.as_ptr());
            let a1 = _mm256_loadu_ps(avx.as_ptr().add(8));
            let b0 = _mm256_loadu_ps(avx.as_ptr().add(16));
            let b1 = _mm256_loadu_ps(avx.as_ptr().add(24));
            let (sum0, sum1, diff0, diff1) = fwht32_avx2_block(a0, a1, b0, b1);
            _mm256_storeu_ps(avx.as_mut_ptr(), sum0);
            _mm256_storeu_ps(avx.as_mut_ptr().add(8), sum1);
            _mm256_storeu_ps(avx.as_mut_ptr().add(16), diff0);
            _mm256_storeu_ps(avx.as_mut_ptr().add(24), diff1);
        }

        assert_eq!(scalar, avx);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn fwht64_avx2_block_matches_scalar_when_available() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar = (0..64)
            .map(|index| (index as f32 * 0.125) - 4.0)
            .collect::<Vec<_>>();
        let mut avx = scalar.clone();
        fwht_in_place_scalar(&mut scalar);

        unsafe {
            use std::arch::x86_64::{_mm256_loadu_ps, _mm256_storeu_ps};

            let a0 = _mm256_loadu_ps(avx.as_ptr());
            let a1 = _mm256_loadu_ps(avx.as_ptr().add(8));
            let a2 = _mm256_loadu_ps(avx.as_ptr().add(16));
            let a3 = _mm256_loadu_ps(avx.as_ptr().add(24));
            let b0 = _mm256_loadu_ps(avx.as_ptr().add(32));
            let b1 = _mm256_loadu_ps(avx.as_ptr().add(40));
            let b2 = _mm256_loadu_ps(avx.as_ptr().add(48));
            let b3 = _mm256_loadu_ps(avx.as_ptr().add(56));
            let (sum0, sum1, sum2, sum3, diff0, diff1, diff2, diff3) =
                fwht64_avx2_block(a0, a1, a2, a3, b0, b1, b2, b3);
            _mm256_storeu_ps(avx.as_mut_ptr(), sum0);
            _mm256_storeu_ps(avx.as_mut_ptr().add(8), sum1);
            _mm256_storeu_ps(avx.as_mut_ptr().add(16), sum2);
            _mm256_storeu_ps(avx.as_mut_ptr().add(24), sum3);
            _mm256_storeu_ps(avx.as_mut_ptr().add(32), diff0);
            _mm256_storeu_ps(avx.as_mut_ptr().add(40), diff1);
            _mm256_storeu_ps(avx.as_mut_ptr().add(48), diff2);
            _mm256_storeu_ps(avx.as_mut_ptr().add(56), diff3);
        }

        assert_eq!(scalar, avx);
    }

    #[test]
    fn fwht_runtime_path_matches_scalar_at_large_sizes() {
        let mut rng = ChaCha8Rng::seed_from_u64(46);
        for size in [1024usize, 2048, 4096] {
            for _ in 0..100 {
                let mut scalar = (0..size)
                    .map(|_| rng.gen_range(-10.0_f32..10.0_f32))
                    .collect::<Vec<_>>();
                let mut dispatched = scalar.clone();

                fwht_in_place_scalar(&mut scalar);
                fwht_in_place(&mut dispatched);

                for (lhs, rhs) in scalar.iter().zip(dispatched.iter()) {
                    let scale = lhs.abs().max(rhs.abs()).max(1.0);
                    assert!(
                        ((lhs - rhs) / scale).abs() < 1e-6,
                        "lhs={lhs} rhs={rhs} size={size}"
                    );
                }
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn fwht_tiled_avx2_exact_sizes_match_scalar_when_available() {
        if !(std::arch::is_x86_feature_detected!("avx2")
            && std::arch::is_x86_feature_detected!("fma"))
        {
            return;
        }

        for size in [128usize, 256, 512, 1024, 2048, 4096] {
            let mut scalar = (0..size)
                .map(|index| ((index as f32 * 0.125) - 8.0).cos())
                .collect::<Vec<_>>();
            let mut avx = scalar.clone();
            fwht_in_place_scalar(&mut scalar);

            unsafe { fwht_in_place_avx2(&mut avx) };

            assert_eq!(scalar, avx, "size={size}");
        }
    }
}
