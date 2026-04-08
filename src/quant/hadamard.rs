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

/// Apply the orthonormalized FWHT in place.
pub fn orthonormal_fwht_in_place(values: &mut [f32]) {
    fwht_in_place(values);
    let scale = (values.len() as f32).sqrt().recip();
    for value in values {
        *value *= scale;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn fwht_in_place_avx2(values: &mut [f32]) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_loadu_ps, _mm256_storeu_ps, _mm256_sub_ps};

    let mut width = 1;
    while width < values.len() {
        let step = width * 2;
        for chunk in values.chunks_exact_mut(step) {
            let (left, right) = chunk.split_at_mut(width);
            let mut i = 0;
            while i + 8 <= width {
                let a = unsafe { _mm256_loadu_ps(left.as_ptr().add(i)) };
                let b = unsafe { _mm256_loadu_ps(right.as_ptr().add(i)) };
                let sum = _mm256_add_ps(a, b);
                let diff = _mm256_sub_ps(a, b);
                unsafe {
                    _mm256_storeu_ps(left.as_mut_ptr().add(i), sum);
                    _mm256_storeu_ps(right.as_mut_ptr().add(i), diff);
                }
                i += 8;
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
    fn miri_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0];
        fwht_in_place(&mut data);
    }

    #[test]
    fn miri_orthonormal_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0, -1.5, 4.0, 2.0, -0.25];
        orthonormal_fwht_in_place(&mut data);
    }
}
