use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

/// NEON backend. Per 16 packed bytes (32 dims): nibble-split, codebook
/// dequant via `vqtbl1q_s8`, de-interleaved `vld2q_s8` query loads (even
/// dims pair with low nibbles, odd with high), `vmull_s8` i16 products and
/// `vpadalq_s16` widening accumulation into i32 lanes. Every product fits
/// i16 (|i8×i8| <= 16129) and the i32 accumulation is order-independent, so
/// results are bit-identical to the scalar reference.
///
/// Task 141: on `dotprod` hardware (all Apple M-series, Graviton 2+) the
/// same 32-dim step runs through `vdotq_s32`, replacing each
/// `vmull`+`vpadal` pair with one instruction. SDOT accumulates the exact
/// i8×i8 products into i32 lanes, so the order-independent integer sum —
/// and therefore bit-identity with the scalar reference — is unchanged.
#[cfg(target_arch = "aarch64")]
pub(super) fn score_block32_neon(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    if std::arch::is_aarch64_feature_detected!("dotprod") {
        debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
        for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
            // SAFETY: dotprod (which implies NEON) detected above; the batch
            // wrapper validated code lengths (>= ceil(original_dim / 2)
            // packed bytes) and the prepared query carries exactly
            // original_dim rotated values.
            *out = unsafe { score_candidate_neon_dotprod(prepared, original_dim, code) };
        }
        return Isa::Neon;
    }
    if std::arch::is_aarch64_feature_detected!("neon") {
        debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
        for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
            // SAFETY: NEON detected above; the batch wrapper validated code
            // lengths (>= ceil(original_dim / 2) packed bytes) and the
            // prepared query carries exactly original_dim rotated values.
            *out = unsafe { score_candidate_neon(prepared, original_dim, code) };
        }
        return Isa::Neon;
    }
    scalar::score_block32_scalar(prepared, original_dim, codes, out_scores)
}

#[cfg(not(target_arch = "aarch64"))]
pub(super) fn score_block32_neon(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(prepared, original_dim, codes, out_scores)
}

#[cfg(target_arch = "aarch64")]
pub(super) fn score_partial_neon(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    if std::arch::is_aarch64_feature_detected!("dotprod") {
        debug_assert_eq!(out_scores.len(), codes.len());
        for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
            // SAFETY: as in score_block32_neon.
            *out = unsafe { score_candidate_neon_dotprod(prepared, original_dim, code) };
        }
        return Isa::Neon;
    }
    if std::arch::is_aarch64_feature_detected!("neon") {
        debug_assert_eq!(out_scores.len(), codes.len());
        for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
            // SAFETY: as in score_block32_neon.
            *out = unsafe { score_candidate_neon(prepared, original_dim, code) };
        }
        return Isa::Neon;
    }
    scalar::score_partial_scalar(prepared, original_dim, codes, out_scores)
}

#[cfg(not(target_arch = "aarch64"))]
pub(super) fn score_partial_neon(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_partial_scalar(prepared, original_dim, codes, out_scores)
}

/// sdot accumulator using inline asm. Equivalent to the still-unstable
/// `vdotq_s32(acc, a, b)` intrinsic (`stdarch_neon_dotprod`, rust#117224);
/// same shape as the `bfdot_asm` precedent in `quant::rabitq`.
///
/// Result: `acc[i] += sum(a[4i..4i+4][j] as i32 * b[4i..4i+4][j] as i32)`
/// for i in 0..4 — exact widened i8×i8 products accumulated in i32.
///
/// # Safety
///
/// Caller must confirm `dotprod` availability.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon,dotprod")]
unsafe fn sdot_asm(
    acc: std::arch::aarch64::int32x4_t,
    a: std::arch::aarch64::int8x16_t,
    b: std::arch::aarch64::int8x16_t,
) -> std::arch::aarch64::int32x4_t {
    let mut out = acc;
    core::arch::asm!(
        "sdot {acc:v}.4s, {a:v}.16b, {b:v}.16b",
        acc = inout(vreg) out,
        a = in(vreg) a,
        b = in(vreg) b,
        options(pure, nomem, nostack),
    );
    out
}

/// Task 141 dotprod variant of [`score_candidate_neon`]: identical
/// nibble-split + `vqtbl1q_s8` dequant + de-interleaved `vld2q_s8` loads,
/// but each `vmull_s8`/`vmull_high_s8` + `vpadalq_s16` pair collapses into
/// one `vdotq_s32`. SDOT sums exact i8×i8 products into i32 lanes, so the
/// result stays bit-identical to the scalar reference.
///
/// # Safety
///
/// Caller must confirm `dotprod` availability (implies NEON),
/// `mse_packed.len() >= ceil(original_dim / 2)`, and
/// `prepared.rotated.len() == original_dim`.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon,dotprod")]
unsafe fn score_candidate_neon_dotprod(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    use std::arch::aarch64::{
        vaddvq_s32, vandq_u8, vdupq_n_s32, vdupq_n_u8, vld1q_s8, vld1q_u8, vld2q_s8, vqtbl1q_s8,
        vshrq_n_u8,
    };

    if prepared.score_scale == 0.0 {
        return 0.0;
    }

    let codebook = vld1q_s8(prepared.codebook.as_ptr());
    let nibble_mask = vdupq_n_u8(0x0F);
    let mut acc = vdupq_n_s32(0);

    let full_chunks = original_dim / 32;
    let mut dim_index = 0usize;
    for chunk in 0..full_chunks {
        let byte_base = chunk * 16;
        // SAFETY: byte_base + 16 <= ceil(original_dim / 2) within full
        // 32-dim chunks; dim_index + 32 <= original_dim for rotated.
        let packed = vld1q_u8(mse_packed.as_ptr().add(byte_base));
        let low_indices = vandq_u8(packed, nibble_mask);
        let high_indices = vshrq_n_u8::<4>(packed);
        // Low nibbles dequantize even dims, high nibbles odd dims.
        let dequant_even = vqtbl1q_s8(codebook, low_indices);
        let dequant_odd = vqtbl1q_s8(codebook, high_indices);
        let rotated = vld2q_s8(prepared.rotated.as_ptr().add(dim_index));
        acc = sdot_asm(acc, dequant_even, rotated.0);
        acc = sdot_asm(acc, dequant_odd, rotated.1);
        dim_index += 32;
    }

    let mut sum = vaddvq_s32(acc);

    // Scalar tail for the trailing < 32 dims, identical to the reference.
    while dim_index < original_dim {
        let packed = mse_packed[dim_index / 2];
        let nibble = if dim_index % 2 == 0 {
            packed & 0x0F
        } else {
            packed >> 4
        } as usize;
        sum += prepared.codebook[nibble] as i32 * prepared.rotated[dim_index] as i32;
        dim_index += 1;
    }

    sum as f32 * prepared.score_scale
}

/// Dispatch on dotprod hardware routes every batch call through the SDOT
/// kernel, so the legacy `vmull`/`vpadal` path would silently lose coverage
/// on M-series/Graviton test hosts; this pins both against the scalar
/// reference directly.
#[cfg(all(test, target_arch = "aarch64"))]
mod neon_path_tests {
    use super::super::scalar;
    use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

    #[test]
    fn legacy_and_dotprod_candidate_paths_match_scalar_bits() {
        for dimensions in [7usize, 64, 100, 191, 1536] {
            let prepared = Int8ApproxNoQjl4BitQuery {
                codebook: [
                    -128, 127, -90, 64, -33, 17, -5, 1, 0, -2, 8, -21, 47, -76, 101, -120,
                ],
                rotated: (0..dimensions)
                    .map(|dim| (((dim * 37 + 11) % 255) as i16 - 128) as i8)
                    .collect(),
                score_scale: 0.25,
            };
            let code: Vec<u8> = (0..dimensions.div_ceil(2))
                .map(|byte| {
                    let low = (3 * byte + 5) % 16;
                    let high = (7 * byte + 1) % 16;
                    (low | (high << 4)) as u8
                })
                .collect();
            let reference = scalar::score_candidate(&prepared, dimensions, &code);

            if std::arch::is_aarch64_feature_detected!("neon") {
                // SAFETY: NEON detected; code has ceil(dimensions / 2) bytes
                // and prepared.rotated has exactly `dimensions` values.
                let legacy = unsafe { super::score_candidate_neon(&prepared, dimensions, &code) };
                assert_eq!(legacy.to_bits(), reference.to_bits(), "dims={dimensions}");
            }
            if std::arch::is_aarch64_feature_detected!("dotprod") {
                // SAFETY: dotprod detected; same length guarantees as above.
                let dotprod =
                    unsafe { super::score_candidate_neon_dotprod(&prepared, dimensions, &code) };
                assert_eq!(dotprod.to_bits(), reference.to_bits(), "dims={dimensions}");
            }
        }
    }
}

/// # Safety
///
/// Caller must confirm NEON availability, `mse_packed.len() >=
/// ceil(original_dim / 2)`, and `prepared.rotated.len() == original_dim`.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_candidate_neon(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    use std::arch::aarch64::{
        vaddvq_s32, vandq_u8, vdupq_n_s32, vdupq_n_u8, vget_low_s8, vld1q_s8, vld1q_u8, vld2q_s8,
        vmull_high_s8, vmull_s8, vpadalq_s16, vqtbl1q_s8, vshrq_n_u8,
    };

    if prepared.score_scale == 0.0 {
        return 0.0;
    }

    let codebook = vld1q_s8(prepared.codebook.as_ptr());
    let nibble_mask = vdupq_n_u8(0x0F);
    let mut acc = vdupq_n_s32(0);

    let full_chunks = original_dim / 32;
    let mut dim_index = 0usize;
    for chunk in 0..full_chunks {
        let byte_base = chunk * 16;
        // SAFETY: byte_base + 16 <= ceil(original_dim / 2) within full
        // 32-dim chunks; dim_index + 32 <= original_dim for rotated.
        let packed = vld1q_u8(mse_packed.as_ptr().add(byte_base));
        let low_indices = vandq_u8(packed, nibble_mask);
        let high_indices = vshrq_n_u8::<4>(packed);
        // Low nibbles dequantize even dims, high nibbles odd dims.
        let dequant_even = vqtbl1q_s8(codebook, low_indices);
        let dequant_odd = vqtbl1q_s8(codebook, high_indices);
        let rotated = vld2q_s8(prepared.rotated.as_ptr().add(dim_index));
        acc = vpadalq_s16(
            acc,
            vmull_s8(vget_low_s8(dequant_even), vget_low_s8(rotated.0)),
        );
        acc = vpadalq_s16(acc, vmull_high_s8(dequant_even, rotated.0));
        acc = vpadalq_s16(
            acc,
            vmull_s8(vget_low_s8(dequant_odd), vget_low_s8(rotated.1)),
        );
        acc = vpadalq_s16(acc, vmull_high_s8(dequant_odd, rotated.1));
        dim_index += 32;
    }

    let mut sum = vaddvq_s32(acc);

    // Scalar tail for the trailing < 32 dims, identical to the reference.
    while dim_index < original_dim {
        let packed = mse_packed[dim_index / 2];
        let nibble = if dim_index % 2 == 0 {
            packed & 0x0F
        } else {
            packed >> 4
        } as usize;
        sum += prepared.codebook[nibble] as i32 * prepared.rotated[dim_index] as i32;
        dim_index += 1;
    }

    sum as f32 * prepared.score_scale
}
