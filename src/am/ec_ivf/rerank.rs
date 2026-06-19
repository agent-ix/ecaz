//! Task 111g: pluggable IVF coarse-rerank representations.
//!
//! The coarse_rerank survivor set is reranked table-side by reading the heap
//! source column (the original full-precision f32 vector) **tid-sorted** and
//! rescoring each candidate through the configured `rerank_format`:
//!
//! - `f32`  — exact negative inner product on the fetched source vector. This
//!   is bit-identical to the pre-111g heap_f32 rerank path.
//! - `f16`  — round both the query and each fetched source vector to IEEE-754
//!   binary16 and back to f32, then take the exact negative inner product. Half
//!   the value precision, no new SIMD.
//! - `rabitq4` — encode each fetched source vector with the shared 4-bit RaBitQ
//!   codec and score through the existing `candidate_batch` RaBitQ scorers, then
//!   negate the inner-product estimate.
//!
//! All three keep the on-disk layout untouched: the rerank payload is always the
//! heap source vector. The representation only changes how a fetched vector is
//! *scored*, so the read order (tid-sorted) and IO shape match the f32 path.

use super::options::RerankFormat;
use super::quantizer::{IvfPreparedQuery, IvfQuantizer};
use crate::am::ec_hnsw::source;
use crate::am::ec_ivf::options::StorageFormat;

/// Resolved rerank scorer for a single scan. Owns whatever per-query state the
/// chosen representation needs so the per-candidate loop stays allocation-light.
pub(super) enum RerankScorer {
    /// Exact f32 negative inner product (the pre-111g heap_f32 behaviour).
    F32,
    /// f16 round-trip negative inner product. Holds the query pre-rounded to
    /// f16 so each candidate only rounds its own source vector.
    F16 { query_f16: Vec<f32> },
    /// 4-bit RaBitQ estimate via the shared candidate_batch scorers.
    RaBitQ4 {
        quantizer: IvfQuantizer,
        prepared_query: IvfPreparedQuery,
        payload_len: usize,
    },
}

impl RerankScorer {
    /// Build the scorer for `rerank_format`. `f16`/`rabitq4` are only valid for
    /// the `coarse_rerank` storage format (table-side rerank); the caller has
    /// already validated that pairing at index creation, but this re-checks so a
    /// stray format on another storage profile is a hard error rather than a
    /// silent wrong answer.
    pub(super) fn resolve(
        rerank_format: RerankFormat,
        storage_format: StorageFormat,
        dimensions: usize,
        query: &[f32],
    ) -> Result<Self, String> {
        match rerank_format {
            // Auto never reaches scan time (build_options_from_reloptions
            // resolves it), but treat it as the exact path defensively.
            RerankFormat::Auto | RerankFormat::F32 => Ok(Self::F32),
            RerankFormat::F16 => {
                let query_f16 = query.iter().copied().map(f16_round_trip).collect();
                Ok(Self::F16 { query_f16 })
            }
            RerankFormat::RaBitQ4 => {
                if storage_format != StorageFormat::CoarseRerank {
                    return Err(format!(
                        "ec_ivf rerank_format = 'rabitq4' requires storage_format = 'coarse_rerank', got {}",
                        storage_format.reloption_name()
                    ));
                }
                let quantizer = IvfQuantizer::resolve_with_pq_group_size_and_bits(
                    StorageFormat::RaBitQ,
                    dimensions,
                    None,
                    Some(4),
                )?;
                let prepared_query = quantizer.prepare_ip_query(query)?;
                let payload_len = quantizer.payload_len();
                Ok(Self::RaBitQ4 {
                    quantizer,
                    prepared_query,
                    payload_len,
                })
            }
            RerankFormat::RaBitQ2 | RerankFormat::RaBitQ8 | RerankFormat::TurboQuant => Err(format!(
                "ec_ivf rerank_format = '{}' is not implemented",
                rerank_format.reloption_name()
            )),
        }
    }

    /// Score a single fetched source vector against `query`, returning the
    /// candidate score (negative inner product; lower is better). Used by the
    /// f32/f16 paths which score per-candidate during the tid-sorted fetch loop.
    pub(super) fn score_source(&self, query: &[f32], source: &[f32]) -> f32 {
        match self {
            Self::F32 => source::negative_inner_product_index_internal(query, source),
            Self::F16 { query_f16 } => f16_negative_inner_product(query_f16, source),
            // RaBitQ4 scores in a batch; this entry point is not used for it.
            Self::RaBitQ4 { .. } => {
                pgrx::error!("ec_ivf rabitq4 rerank must score through score_sources_batch")
            }
        }
    }

    /// Whether this representation scores per-candidate (`score_source`) or in a
    /// batch over all fetched source vectors (`score_sources_batch`).
    pub(super) fn is_batched(&self) -> bool {
        matches!(self, Self::RaBitQ4 { .. })
    }

    /// The compact sidecar payload width this representation persists, for
    /// `rerank_placement = 'index'`. `None` for f32 (no sidecar — keeps the
    /// heap source). f16 stores `dimensions * 2` bytes; rabitq4 stores its
    /// codec payload length.
    pub(super) fn sidecar_payload_len(&self, dimensions: usize) -> Option<usize> {
        match self {
            Self::F32 => None,
            Self::F16 { .. } => Some(dimensions * 2),
            Self::RaBitQ4 { payload_len, .. } => Some(*payload_len),
        }
    }

    /// Encode an f32 source vector into this representation's compact sidecar
    /// payload (the persisted `0x2A` payload). `None` for f32.
    pub(super) fn encode_sidecar_payload(&self, source: &[f32]) -> Result<Option<Vec<u8>>, String> {
        match self {
            Self::F32 => Ok(None),
            Self::F16 { .. } => Ok(Some(pack_f16_payload(source))),
            Self::RaBitQ4 {
                quantizer,
                payload_len,
                ..
            } => {
                let (_dimensions, _gamma, code) = quantizer.encode_source(source)?;
                if code.len() != *payload_len {
                    return Err(format!(
                        "ec_ivf rabitq4 sidecar code length {} does not match payload length {payload_len}",
                        code.len()
                    ));
                }
                Ok(Some(code))
            }
        }
    }

    /// Score a single candidate from its persisted compact sidecar payload
    /// (`rerank_placement = 'index'`). Used by the f16 path which scores
    /// per-candidate (against the f16-prepared query held in `self`); rabitq4
    /// scores its payloads in a batch via `score_sidecar_payloads_batch`.
    pub(super) fn score_sidecar_payload(&self, payload: &[u8]) -> f32 {
        match self {
            Self::F32 => {
                pgrx::error!("ec_ivf f32 rerank has no sidecar payload to score")
            }
            Self::F16 { query_f16 } => {
                let source = unpack_f16_payload(payload);
                f16_negative_inner_product(query_f16, &source)
            }
            Self::RaBitQ4 { .. } => {
                pgrx::error!(
                    "ec_ivf rabitq4 sidecar rerank must score through score_sidecar_payloads_batch"
                )
            }
        }
    }

    /// Batch-score persisted rabitq4 sidecar payloads directly (no re-encode):
    /// `payloads` is a flat `count * payload_len` slab in survivor order.
    pub(super) fn score_sidecar_payloads_batch(
        &self,
        payloads: &[u8],
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        let Self::RaBitQ4 {
            quantizer,
            prepared_query,
            payload_len,
        } = self
        else {
            return Err(
                "ec_ivf score_sidecar_payloads_batch called for a non-batched rerank format".into(),
            );
        };
        if out_scores.is_empty() {
            return Ok(());
        }
        if payloads.len() != out_scores.len() * payload_len {
            return Err(format!(
                "ec_ivf rabitq4 sidecar payload slab {} does not match {} entries * {payload_len}",
                payloads.len(),
                out_scores.len()
            ));
        }
        let mut estimates: Vec<f32> = Vec::with_capacity(out_scores.len());
        let scored = quantizer.score_ip_bits1_batch_from_payloads(
            prepared_query,
            payloads,
            *payload_len,
            &mut estimates,
        )?;
        if !scored || estimates.len() != out_scores.len() {
            return Err(format!(
                "ec_ivf rabitq4 sidecar batch scorer produced {} scores for {} payloads",
                estimates.len(),
                out_scores.len()
            ));
        }
        for (out, estimate) in out_scores.iter_mut().zip(estimates.iter()) {
            *out = -estimate;
        }
        Ok(())
    }

    /// Batch-score every fetched source vector. `out_scores.len()` must equal
    /// `sources.len()`. Only the batched representations (rabitq4) implement
    /// this; the f32/f16 paths score per-candidate via `score_source`.
    pub(super) fn score_sources_batch(
        &self,
        sources: &[&[f32]],
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        let Self::RaBitQ4 {
            quantizer,
            prepared_query,
            payload_len,
        } = self
        else {
            return Err("ec_ivf score_sources_batch called for a non-batched rerank format".into());
        };
        if sources.len() != out_scores.len() {
            return Err(format!(
                "ec_ivf rabitq4 rerank score count {} does not match source count {}",
                out_scores.len(),
                sources.len()
            ));
        }
        if sources.is_empty() {
            return Ok(());
        }

        // Encode each fetched source vector into the rabitq4 payload slab, then
        // score the whole slab through the shared candidate_batch scorers.
        let mut payloads = Vec::with_capacity(sources.len() * payload_len);
        for source in sources {
            let (_dimensions, _gamma, code) = quantizer.encode_source(source)?;
            if code.len() != *payload_len {
                return Err(format!(
                    "ec_ivf rabitq4 rerank code length {} does not match payload length {payload_len}",
                    code.len()
                ));
            }
            payloads.extend_from_slice(&code);
        }

        let mut estimates: Vec<f32> = Vec::with_capacity(sources.len());
        let scored = quantizer.score_ip_bits1_batch_from_payloads(
            prepared_query,
            &payloads,
            *payload_len,
            &mut estimates,
        )?;
        if !scored || estimates.len() != sources.len() {
            return Err(format!(
                "ec_ivf rabitq4 rerank batch scorer produced {} scores for {} sources",
                estimates.len(),
                sources.len()
            ));
        }
        // The shared scorers return an inner-product estimate (higher = closer);
        // the candidate score convention is negative inner product.
        for (out, estimate) in out_scores.iter_mut().zip(estimates.iter()) {
            *out = -estimate;
        }
        Ok(())
    }
}

/// Query-independent encoder for the persisted compact rerank sidecar payload
/// (`rerank_placement = 'index'`). Resolved at build/insert time (no query),
/// it produces the `0x2A` payload that the scan-time `RerankScorer` later reads
/// and scores. Mirrors the scoring scorer's compact reps: f16 and rabitq4.
pub(super) enum RerankSidecarEncoder {
    F16,
    RaBitQ4 {
        quantizer: IvfQuantizer,
        payload_len: usize,
    },
}

impl RerankSidecarEncoder {
    /// Resolve the sidecar encoder for an index-placement compact `rerank_format`.
    /// Returns `Ok(None)` for formats that keep the heap source (f32) — those
    /// never persist a sidecar. Errors for unimplemented formats.
    pub(super) fn resolve(
        rerank_format: RerankFormat,
        dimensions: usize,
    ) -> Result<Option<Self>, String> {
        match rerank_format {
            RerankFormat::Auto | RerankFormat::F32 => Ok(None),
            RerankFormat::F16 => Ok(Some(Self::F16)),
            RerankFormat::RaBitQ4 => {
                let quantizer = IvfQuantizer::resolve_with_pq_group_size_and_bits(
                    StorageFormat::RaBitQ,
                    dimensions,
                    None,
                    Some(4),
                )?;
                let payload_len = quantizer.payload_len();
                Ok(Some(Self::RaBitQ4 {
                    quantizer,
                    payload_len,
                }))
            }
            RerankFormat::RaBitQ2 | RerankFormat::RaBitQ8 | RerankFormat::TurboQuant => Err(format!(
                "ec_ivf rerank_format = '{}' is not implemented for index placement",
                rerank_format.reloption_name()
            )),
        }
    }

    /// The compact payload width this encoder persists (f16: `dimensions * 2`).
    pub(super) fn payload_len(&self, dimensions: usize) -> usize {
        match self {
            Self::F16 => dimensions * 2,
            Self::RaBitQ4 { payload_len, .. } => *payload_len,
        }
    }

    /// Encode an f32 source vector into the persisted compact sidecar payload.
    pub(super) fn encode(&self, source: &[f32]) -> Result<Vec<u8>, String> {
        match self {
            Self::F16 => Ok(pack_f16_payload(source)),
            Self::RaBitQ4 {
                quantizer,
                payload_len,
            } => {
                let (_dimensions, _gamma, code) = quantizer.encode_source(source)?;
                if code.len() != *payload_len {
                    return Err(format!(
                        "ec_ivf rabitq4 sidecar code length {} does not match payload length {payload_len}",
                        code.len()
                    ));
                }
                Ok(code)
            }
        }
    }
}

/// Round a single f32 to IEEE-754 binary16 (round-to-nearest-even) and back to
/// f32. No `half` crate dependency and no SIMD — this is a scalar reference
/// round-trip used only on the bounded rerank frontier.
pub(super) fn f16_round_trip(value: f32) -> f32 {
    f16_bits_to_f32(f32_to_f16_bits(value))
}

/// Pack an f32 source vector into the compact f16 sidecar payload: two
/// little-endian bytes per dimension (IEEE-754 binary16, round-to-nearest).
/// This is the persisted `0x2A` payload for `rerank_format = 'f16'` with
/// `rerank_placement = 'index'` — half the bytes of the f32 heap source.
pub(super) fn pack_f16_payload(source: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(source.len() * 2);
    for value in source {
        out.extend_from_slice(&f32_to_f16_bits(*value).to_le_bytes());
    }
    out
}

/// Decode a compact f16 sidecar payload (2 little-endian bytes per dimension)
/// back to f32 values. The inverse of `pack_f16_payload`; the returned vector
/// is bit-identical to round-tripping each stored source component through f16.
pub(super) fn unpack_f16_payload(payload: &[u8]) -> Vec<f32> {
    payload
        .chunks_exact(2)
        .map(|chunk| f16_bits_to_f32(u16::from_le_bytes([chunk[0], chunk[1]])))
        .collect()
}

fn f16_negative_inner_product(query_f16: &[f32], source: &[f32]) -> f32 {
    if query_f16.len() != source.len() {
        pgrx::error!(
            "ec_ivf f16 rerank source vector dimension mismatch: query dim {}, source dim {}",
            query_f16.len(),
            source.len()
        );
    }
    let sum: f32 = query_f16
        .iter()
        .zip(source.iter())
        .map(|(q, s)| q * f16_round_trip(*s))
        .sum();
    -sum
}

/// Convert an f32 to IEEE-754 binary16, returning the 16 raw bits. Implements
/// round-to-nearest-even with subnormal and inf/NaN handling.
fn f32_to_f16_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;

    if exponent == 0xff {
        // Inf / NaN. Preserve a non-zero mantissa as a quiet NaN.
        let mant16 = if mantissa != 0 {
            // Keep top mantissa bits; ensure non-zero so NaN stays NaN.
            ((mantissa >> 13) as u16) | 0x0200
        } else {
            0
        };
        return sign | 0x7c00 | mant16;
    }

    // Unbiased exponent for f16 (bias 15) from f32 (bias 127).
    let unbiased = exponent - 127 + 15;

    if unbiased >= 0x1f {
        // Overflow to infinity.
        return sign | 0x7c00;
    }

    if unbiased <= 0 {
        // Subnormal or zero in f16. The implicit leading 1 must be added back.
        if unbiased < -10 {
            // Too small to represent even as a subnormal — flushes to zero.
            return sign;
        }
        let mantissa_with_implicit = mantissa | 0x0080_0000;
        let shift = (14 - unbiased) as u32;
        let mut mant16 = (mantissa_with_implicit >> shift) as u16;
        // Round to nearest even.
        let round_bit = (mantissa_with_implicit >> (shift - 1)) & 1;
        let sticky = (mantissa_with_implicit & ((1 << (shift - 1)) - 1)) != 0;
        if round_bit == 1 && (sticky || (mant16 & 1) == 1) {
            mant16 += 1;
        }
        return sign | mant16;
    }

    // Normal f16 number.
    let mut mant16 = (mantissa >> 13) as u16;
    let exp16 = (unbiased as u16) << 10;
    let round_bit = (mantissa >> 12) & 1;
    let sticky = (mantissa & 0x0fff) != 0;
    let mut result = sign | exp16 | mant16;
    if round_bit == 1 && (sticky || (mant16 & 1) == 1) {
        // Carry can ripple into the exponent; adding to the combined value
        // handles the mantissa-overflow-into-exponent case for free.
        result += 1;
    }
    let _ = &mut mant16;
    result
}

/// Convert IEEE-754 binary16 raw bits back to f32.
fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = ((bits >> 10) & 0x1f) as u32;
    let mantissa = (bits & 0x03ff) as u32;

    if exponent == 0 {
        if mantissa == 0 {
            // Signed zero.
            return f32::from_bits(sign);
        }
        // Subnormal f16 → normalized f32.
        let mut mant = mantissa;
        let mut exp: i32 = -1;
        // Normalize: shift mantissa left until the implicit bit appears.
        while (mant & 0x0400) == 0 {
            mant <<= 1;
            exp -= 1;
        }
        mant &= 0x03ff;
        let f32_exp = ((exp + 127 + 1) as u32) << 23;
        let f32_mant = mant << 13;
        return f32::from_bits(sign | f32_exp | f32_mant);
    }

    if exponent == 0x1f {
        // Inf / NaN.
        let f32_mant = mantissa << 13;
        return f32::from_bits(sign | 0x7f80_0000 | f32_mant);
    }

    // Normal number: rebias exponent (15 → 127) and widen mantissa.
    let f32_exp = (exponent + 127 - 15) << 23;
    let f32_mant = mantissa << 13;
    f32::from_bits(sign | f32_exp | f32_mant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f16_round_trip_is_exact_for_small_integers() {
        for value in [0.0_f32, 1.0, -1.0, 2.0, -2.0, 256.0, -256.0, 0.5, 0.25] {
            assert_eq!(
                f16_round_trip(value),
                value,
                "f16 round-trip should be exact for {value}"
            );
        }
    }

    #[test]
    fn f16_round_trip_preserves_sign_of_zero() {
        assert_eq!(f16_round_trip(0.0_f32).to_bits(), 0.0_f32.to_bits());
        assert_eq!(f16_round_trip(-0.0_f32).to_bits(), (-0.0_f32).to_bits());
    }

    #[test]
    fn f16_round_trip_handles_inf_and_nan() {
        assert!(f16_round_trip(f32::INFINITY).is_infinite());
        assert!(f16_round_trip(f32::INFINITY) > 0.0);
        assert!(f16_round_trip(f32::NEG_INFINITY).is_infinite());
        assert!(f16_round_trip(f32::NEG_INFINITY) < 0.0);
        assert!(f16_round_trip(f32::NAN).is_nan());
    }

    #[test]
    fn f16_round_trip_overflows_large_values_to_infinity() {
        // f16 max finite is 65504; anything larger overflows to +inf.
        assert!(f16_round_trip(1.0e30_f32).is_infinite());
    }

    #[test]
    fn f16_round_trip_is_close_for_typical_unit_components() {
        // Normalized embedding components are small; the f16 round-trip should
        // stay within the binary16 relative precision (~2^-11).
        for value in [0.1_f32, -0.137, 0.0123, -0.5009, 0.999] {
            let round = f16_round_trip(value);
            let rel = ((round - value) / value).abs();
            assert!(
                rel < 1.0e-3,
                "f16 round-trip of {value} = {round} exceeded binary16 precision (rel {rel})"
            );
        }
    }

    #[test]
    fn f16_negative_inner_product_matches_round_tripped_reference() {
        let query = [0.2_f32, -0.5, 0.75, 0.1, -0.9];
        let source = [0.4_f32, 0.25, -0.6, 0.05, 0.33];
        let query_f16: Vec<f32> = query.iter().copied().map(f16_round_trip).collect();
        let actual = f16_negative_inner_product(&query_f16, &source);
        let expected: f32 = -query
            .iter()
            .zip(source.iter())
            .map(|(q, s)| f16_round_trip(*q) * f16_round_trip(*s))
            .sum::<f32>();
        assert_eq!(actual.to_bits(), expected.to_bits());
    }

    #[test]
    fn f16_pack_unpack_roundtrips_through_binary16() {
        let source = [0.2_f32, -0.5, 0.75, 0.1, -0.9, 0.0123];
        let packed = pack_f16_payload(&source);
        assert_eq!(packed.len(), source.len() * 2);
        let unpacked = unpack_f16_payload(&packed);
        // Unpacking is exactly the f16 round-trip of each source component.
        for (orig, got) in source.iter().zip(unpacked.iter()) {
            assert_eq!(got.to_bits(), f16_round_trip(*orig).to_bits());
        }
    }

    #[test]
    fn f16_sidecar_payload_scores_match_table_f16_path() {
        // The index-side f16 path (score_sidecar_payload over a packed payload)
        // must produce the same score as the table-side f16 path
        // (score_source over the raw f32 source), because both round-trip the
        // source through binary16 before the inner product.
        let query = [0.2_f32, -0.5, 0.75, 0.1, -0.9];
        let source = [0.4_f32, 0.25, -0.6, 0.05, 0.33];
        let scorer = RerankScorer::F16 {
            query_f16: query.iter().copied().map(f16_round_trip).collect(),
        };
        let table_score = scorer.score_source(&query, &source);
        let payload = pack_f16_payload(&source);
        let index_score = scorer.score_sidecar_payload(&payload);
        assert_eq!(index_score.to_bits(), table_score.to_bits());
    }

    #[test]
    fn f16_sidecar_encoder_matches_pack_helper() {
        let encoder = RerankSidecarEncoder::F16;
        let source = [0.1_f32, -0.2, 0.3, -0.4];
        assert_eq!(encoder.payload_len(source.len()), source.len() * 2);
        assert_eq!(encoder.encode(&source).unwrap(), pack_f16_payload(&source));
    }

    #[test]
    fn f32_scorer_matches_source_reference() {
        let query = [0.2_f32, -0.5, 0.75];
        let source = [0.4_f32, 0.25, -0.6];
        let scorer = RerankScorer::F32;
        assert_eq!(
            scorer.score_source(&query, &source).to_bits(),
            source::negative_inner_product_index_internal(&query, &source).to_bits()
        );
    }
}
