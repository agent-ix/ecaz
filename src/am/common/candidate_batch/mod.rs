use super::quant_codec::QuantCodecKind;
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedLutNoQjl4BitQuery, PreparedQuery, ProdQuantizer};
use std::time::Instant;

mod counters;
mod drivers;

pub(crate) use counters::{
    block_kernel_scoring_snapshots, candidate_batch_scoring_snapshots,
    record_block_scalar_score_for, reset_candidate_batch_scoring_counters, BlockKernelCounterKey,
    CandidateBatchScoringSurface,
};
#[cfg(not(test))]
use counters::{record_block_kernel_score, record_flush_width};
#[cfg(test)]
pub(crate) use counters::{
    record_block_kernel_score, record_flush_width, CANDIDATE_BATCH_COUNTER_TEST_LOCK,
};
use drivers::{score_width_cascade, BatchScoringTiming};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub(crate) enum CandidateMeta<'a> {
    None,
    Gamma(f32),
    GammaAndResidualSigns { gamma: f32, signs: &'a [u8] },
    Binary,
    RaBitQ,
    GroupedPq { group_count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CandidatePayload<'a, Meta = CandidateMeta<'a>> {
    pub(crate) code: &'a [u8],
    pub(crate) meta: Meta,
}

impl<'a, Meta> CandidatePayload<'a, Meta> {
    pub(crate) fn new(code: &'a [u8], meta: Meta) -> Self {
        Self { code, meta }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateBatch<'a, Id, Meta = CandidateMeta<'a>> {
    ids: Vec<Id>,
    payloads: Vec<CandidatePayload<'a, Meta>>,
    capacity: usize,
}

impl<'a, Id, Meta> CandidateBatch<'a, Id, Meta> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            payloads: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub(crate) fn push(
        &mut self,
        id: Id,
        payload: CandidatePayload<'a, Meta>,
    ) -> Result<(), String> {
        if self.ids.len() >= self.capacity {
            return Err(format!(
                "candidate batch capacity {} exceeded",
                self.capacity
            ));
        }
        self.ids.push(id);
        self.payloads.push(payload);
        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.ids.len()
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn ids(&self) -> &[Id] {
        &self.ids
    }

    pub(crate) fn payloads(&self) -> &[CandidatePayload<'a, Meta>] {
        &self.payloads
    }
}

fn record_batch_scoring_timing(
    surface: CandidateBatchScoringSurface,
    quant_kind: QuantCodecKind,
    batch_width: usize,
    timing: &BatchScoringTiming,
) {
    let isa = timing.kernel_isa.unwrap_or(Isa::Scalar);
    record_block_kernel_score(
        BlockKernelCounterKey {
            surface,
            quant_kind,
            isa,
        },
        timing.kernel_candidates,
        timing.kernel_elapsed_nanos,
    );
    record_block_scalar_score_for(
        surface,
        quant_kind,
        timing.scalar_candidates,
        timing.scalar_elapsed_nanos,
    );
    record_flush_width(surface, quant_kind, isa, batch_width);
}

pub(crate) fn score_turboquant_no_qjl_4bit_batch<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    score_turboquant_no_qjl_4bit_batch_for(
        CandidateBatchScoringSurface::Unknown,
        quantizer,
        prepared,
        batch,
        out_scores,
    )
}

/// Task 98: HNSW TiledLut exact-mode batch scoring. The scalar tile walk
/// is currently the only backend (SIMD gated on the Phase A width
/// distribution), so the whole run records as scalar work plus one width
/// sample.
pub(crate) fn score_turboquant_tiled_lut_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    quantizer: &ProdQuantizer,
    lut: &[f32],
    tile_size: usize,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "tiled_lut32 score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    if batch.payloads().is_empty() {
        return Ok(());
    }
    let mut codes: Vec<&[u8]> = Vec::with_capacity(batch.len());
    for (index, payload) in batch.payloads().iter().enumerate() {
        validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
        let mse_packed = quantizer.mse_code_bytes_no_qjl_4bit(payload.code);
        crate::quant::tiled_lut32::validate_code_shape(index, quantizer.original_dim, mse_packed)?;
        codes.push(mse_packed);
    }

    let timing = score_width_cascade(
        &codes,
        out_scores,
        crate::quant::tiled_lut32::BLOCK_WIDTH,
        false,
        |run_codes, run_scores| {
            crate::quant::tiled_lut32::score_tiled_lut_run(
                lut,
                tile_size,
                quantizer.original_dim,
                run_codes,
                run_scores,
            )
        },
        |tail_codes, tail_scores, timing| {
            if tail_codes.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::tiled_lut32::score_tiled_lut_run(
                lut,
                tile_size,
                quantizer.original_dim,
                tail_codes,
                tail_scores,
            );
            timing.record_run(
                isa,
                tail_codes.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    );
    record_batch_scoring_timing(
        surface,
        QuantCodecKind::TurboQuantTiledLut,
        batch.len(),
        &timing,
    );
    Ok(())
}

/// Task 98: HNSW Int8Approx exact-mode batch scoring (integer-exact across
/// backends).
pub(crate) fn score_turboquant_int8_approx_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    quantizer: &ProdQuantizer,
    prepared: &crate::quant::prod::Int8ApproxNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "int8_approx32 score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    if batch.payloads().is_empty() {
        return Ok(());
    }
    let mut codes: Vec<&[u8]> = Vec::with_capacity(batch.len());
    for (index, payload) in batch.payloads().iter().enumerate() {
        validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
        let mse_packed = quantizer.mse_code_bytes_no_qjl_4bit(payload.code);
        crate::quant::int8_approx32::validate_code_shape(
            index,
            quantizer.original_dim,
            mse_packed,
        )?;
        codes.push(mse_packed);
    }

    let timing = score_width_cascade(
        &codes,
        out_scores,
        crate::quant::int8_approx32::BLOCK_WIDTH,
        false,
        |block_codes, block_scores| {
            let block: &[&[u8]; crate::quant::int8_approx32::BLOCK_WIDTH] = block_codes
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::int8_approx32::score_int8_approx_block32(
                prepared,
                quantizer.original_dim,
                block,
                block_scores,
            )
        },
        |tail_codes, tail_scores, timing| {
            if tail_codes.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::int8_approx32::score_int8_approx_partial(
                prepared,
                quantizer.original_dim,
                tail_codes,
                tail_scores,
            );
            timing.record_run(
                isa,
                tail_codes.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    );

    record_batch_scoring_timing(
        surface,
        QuantCodecKind::TurboQuantInt8,
        batch.len(),
        &timing,
    );
    Ok(())
}

pub(crate) fn score_turboquant_no_qjl_4bit_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    let result = score_turboquant_no_qjl_4bit_batch_inner(quantizer, prepared, batch, out_scores);
    if let Ok(timing) = &result {
        record_batch_scoring_timing(surface, QuantCodecKind::TurboQuant, batch.len(), timing);
    }
    result.map(|_| ())
}

pub(crate) fn score_turboquant_qjl_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    let result = score_turboquant_qjl_batch_inner(quantizer, prepared, batch, out_scores);
    if let Ok(timing) = &result {
        record_batch_scoring_timing(surface, QuantCodecKind::TurboQuantQjl, batch.len(), timing);
    }
    result.map(|_| ())
}

pub(crate) fn score_grouped_pq_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    lut: &[f32],
    group_count: usize,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    let result = score_grouped_pq_batch_inner(lut, group_count, batch, out_scores);
    if let Ok(timing) = &result {
        record_batch_scoring_timing(surface, QuantCodecKind::GroupedPq, batch.len(), timing);
    }
    result.map(|_| ())
}

pub(crate) fn score_rabitq_bits1_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    prepared: crate::quant::rabitq32::PreparedBits1<'_>,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    let result = score_rabitq_bits1_batch_inner(prepared, batch, out_scores);
    if let Ok(timing) = &result {
        record_batch_scoring_timing(surface, QuantCodecKind::RaBitQ, batch.len(), timing);
    }
    result.map(|_| ())
}

/// Task 95: Hamming sidecar batch scoring over `u64` words. Distances are
/// integer-exact across every ISA backend, so no tolerance framing applies;
/// counter attribution follows the rabitq32 partial-width convention
/// (`kernel_*` = SIMD-backend flushes, `scalar_*` = scalar-executed).
pub(crate) fn score_hamming_words_batch_for(
    surface: CandidateBatchScoringSurface,
    query_words: &[u64],
    candidates: &[&[u64]],
    out_scores: &mut [f32],
) -> Result<(), String> {
    if candidates.len() != out_scores.len() {
        return Err(format!(
            "hamming32 score output count {} does not match candidate count {}",
            out_scores.len(),
            candidates.len()
        ));
    }
    if query_words.is_empty() {
        return Err("hamming32 query word count must be nonzero".to_owned());
    }
    for (index, candidate) in candidates.iter().enumerate() {
        crate::quant::hamming32::validate_word_shape(index, query_words.len(), candidate)?;
    }
    if candidates.is_empty() {
        return Ok(());
    }

    let mut distances = vec![0u32; candidates.len()];
    let timing = score_width_cascade(
        candidates,
        &mut distances,
        crate::quant::hamming32::BLOCK_WIDTH,
        false,
        |block_candidates, block_distances| {
            let block: &[&[u64]; crate::quant::hamming32::BLOCK_WIDTH] = block_candidates
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::hamming32::score_hamming_block32(query_words, block, block_distances)
        },
        |tail_candidates, tail_distances, timing| {
            if tail_candidates.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::hamming32::score_hamming_partial(
                query_words,
                tail_candidates,
                tail_distances,
            );
            timing.record_run(
                isa,
                tail_candidates.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    );

    for (out, distance) in out_scores.iter_mut().zip(distances.iter()) {
        *out = *distance as f32;
    }

    record_batch_scoring_timing(surface, QuantCodecKind::Binary, candidates.len(), &timing);
    Ok(())
}

fn score_grouped_pq_batch_inner<Id>(
    lut: &[f32],
    group_count: usize,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "candidate batch score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    crate::quant::grouped_pq_block::validate_lut_shape(lut, group_count)?;
    validate_grouped_pq_batch_shapes(group_count, batch)?;

    let codes: Vec<&[u8]> = batch
        .payloads()
        .iter()
        .map(|payload| payload.code)
        .collect();
    Ok(score_width_cascade(
        &codes,
        out_scores,
        crate::quant::grouped_pq_block::BLOCK_WIDTH,
        true,
        |block_codes, block_scores| {
            let codes: [&[u8]; crate::quant::grouped_pq_block::BLOCK_WIDTH] = block_codes
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::grouped_pq_block::score_grouped_pq_block32(
                lut,
                group_count,
                codes,
                block_scores,
            )
        },
        |tail_codes, tail_scores, timing| {
            if tail_codes.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::grouped_pq_block::score_grouped_pq_partial(
                lut,
                group_count,
                tail_codes,
                tail_scores,
            );
            timing.record_run(
                isa,
                tail_codes.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    ))
}

#[allow(dead_code)]
fn score_grouped_pq_tail_scalar(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
    timing: &mut BatchScoringTiming,
) {
    if codes.is_empty() {
        return;
    }
    let scalar_started = Instant::now();
    for (code, out_score) in codes.iter().zip(out_scores.iter_mut()) {
        *out_score =
            crate::quant::grouped_pq_block::score_grouped_pq_scalar(lut, group_count, code);
    }
    timing.scalar_candidates += codes.len();
    timing.scalar_elapsed_nanos = timing
        .scalar_elapsed_nanos
        .saturating_add(u64::try_from(scalar_started.elapsed().as_nanos()).unwrap_or(u64::MAX));
}

#[allow(dead_code)]
fn score_grouped_pq_batch_block32<Id>(
    lut: &[f32],
    group_count: usize,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    let codes: Vec<&[u8]> = batch
        .payloads()
        .iter()
        .map(|payload| payload.code)
        .collect();
    Ok(score_width_cascade(
        &codes,
        out_scores,
        crate::quant::grouped_pq_block::BLOCK_WIDTH,
        true,
        |block_codes, block_scores| {
            let codes: [&[u8]; crate::quant::grouped_pq_block::BLOCK_WIDTH] = block_codes
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::grouped_pq_block::score_grouped_pq_block32(
                lut,
                group_count,
                codes,
                block_scores,
            )
        },
        |tail_codes, tail_scores, timing| {
            score_grouped_pq_tail_scalar(lut, group_count, tail_codes, tail_scores, timing);
        },
    ))
}

fn validate_grouped_pq_batch_shapes<Id>(
    group_count: usize,
    batch: &CandidateBatch<'_, Id>,
) -> Result<(), String> {
    for (candidate_index, payload) in batch.payloads().iter().enumerate() {
        validate_grouped_pq_meta(payload.meta, group_count)?;
        crate::quant::grouped_pq_block::validate_code_shape(
            candidate_index,
            group_count,
            payload.code,
        )?;
    }
    Ok(())
}

fn score_turboquant_no_qjl_4bit_batch_inner<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "candidate batch score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    crate::quant::lut32::validate_lut_shape(&prepared.lut, quantizer.original_dim)?;
    let mut mse_codes: Vec<&[u8]> = Vec::with_capacity(batch.len());
    for (index, payload) in batch.payloads().iter().enumerate() {
        validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
        let mse_code = quantizer.mse_code_bytes_no_qjl_4bit(payload.code);
        crate::quant::lut32::validate_mse_code_shape(index, quantizer.original_dim, mse_code)?;
        mse_codes.push(mse_code);
    }

    Ok(score_turboquant_no_qjl_4bit_codes_lut32(
        quantizer.original_dim,
        &prepared.lut,
        &mse_codes,
        out_scores,
    ))
}

fn score_turboquant_no_qjl_4bit_codes_lut32(
    original_dim: usize,
    lut: &[f32],
    mse_codes: &[&[u8]],
    out_scores: &mut [f32],
) -> BatchScoringTiming {
    score_width_cascade(
        mse_codes,
        out_scores,
        crate::quant::lut32::BLOCK_WIDTH,
        true,
        |block_codes, block_scores| {
            let codes: [&[u8]; crate::quant::lut32::BLOCK_WIDTH] = block_codes
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::lut32::score_lut_no_qjl_4bit_block32(
                lut,
                original_dim,
                codes,
                block_scores,
            )
        },
        |tail_codes, tail_scores, timing| {
            if tail_codes.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::lut32::score_lut_no_qjl_4bit_partial(
                lut,
                original_dim,
                tail_codes,
                tail_scores,
            );
            timing.record_run(
                isa,
                tail_codes.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    )
}

fn score_rabitq_bits1_batch_inner<Id>(
    prepared: crate::quant::rabitq32::PreparedBits1<'_>,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "rabitq32 score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    prepared.validate()?;

    for (index, payload) in batch.payloads().iter().enumerate() {
        validate_rabitq_bits1_meta(payload.meta)?;
        crate::quant::rabitq32::validate_code_shape(index, prepared, payload.code)?;
    }

    Ok(score_rabitq_bits1_batch_blocked(
        prepared, batch, out_scores,
    ))
}

fn score_rabitq_bits1_batch_blocked<Id>(
    prepared: crate::quant::rabitq32::PreparedBits1<'_>,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> BatchScoringTiming {
    let codes: Vec<&[u8]> = batch
        .payloads()
        .iter()
        .map(|payload| payload.code)
        .collect();
    score_width_cascade(
        &codes,
        out_scores,
        crate::quant::rabitq32::BLOCK_WIDTH,
        false,
        |block_codes, block_scores| {
            let codes: [&[u8]; crate::quant::rabitq32::BLOCK_WIDTH] = block_codes
                .try_into()
                .expect("width-cascade block length is exact");
            crate::quant::rabitq32::score_rabitq_bits1_block32(prepared, codes, block_scores)
        },
        |tail_codes, tail_scores, timing| {
            if tail_codes.is_empty() {
                return;
            }
            let started = Instant::now();
            let isa = crate::quant::rabitq32::score_rabitq_bits1_partial(
                prepared,
                tail_codes,
                tail_scores,
            );
            timing.record_run(
                isa,
                tail_codes.len(),
                u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
                false,
            );
        },
    )
}

fn score_turboquant_qjl_batch_inner<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "candidate batch score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }
    crate::quant::qjl32::validate_qjl_shape(quantizer, prepared)?;
    validate_turboquant_qjl_batch_shapes(quantizer.original_dim, batch)?;

    let candidates: Result<Vec<(&[u8], f32)>, String> = batch
        .payloads()
        .iter()
        .map(|payload| {
            validate_turboquant_qjl_meta(payload.meta).map(|gamma| (payload.code, gamma))
        })
        .collect();
    let candidates = candidates?;

    Ok(score_width_cascade(
        &candidates,
        out_scores,
        crate::quant::qjl32::BLOCK_WIDTH,
        true,
        |block_candidates, block_scores| {
            let mut codes = [&[][..]; crate::quant::qjl32::BLOCK_WIDTH];
            let mut gammas = [0.0_f32; crate::quant::qjl32::BLOCK_WIDTH];
            for (lane, (code, gamma)) in block_candidates.iter().enumerate() {
                codes[lane] = *code;
                gammas[lane] = *gamma;
            }
            crate::quant::qjl32::score_turboquant_qjl_block32(
                quantizer,
                prepared,
                codes,
                gammas,
                block_scores,
            )
        },
        |tail_candidates, tail_scores, timing| {
            score_turboquant_qjl_remainder(
                quantizer,
                prepared,
                tail_candidates,
                tail_scores,
                timing,
            );
        },
    ))
}

fn score_turboquant_qjl_remainder(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    candidates: &[(&[u8], f32)],
    out_scores: &mut [f32],
    timing: &mut BatchScoringTiming,
) {
    let mut block_start = 0usize;
    while block_start + crate::quant::qjl32::OCTET_WIDTH <= candidates.len() {
        let block_started = Instant::now();
        let mut codes = [&[][..]; crate::quant::qjl32::OCTET_WIDTH];
        let mut gammas = [0.0_f32; crate::quant::qjl32::OCTET_WIDTH];
        for (lane, (code, gamma)) in candidates
            [block_start..block_start + crate::quant::qjl32::OCTET_WIDTH]
            .iter()
            .enumerate()
        {
            codes[lane] = *code;
            gammas[lane] = *gamma;
        }
        let Some(isa) = crate::quant::qjl32::score_turboquant_qjl_octet8_avx2(
            quantizer,
            prepared,
            codes,
            gammas,
            &mut out_scores[block_start..block_start + crate::quant::qjl32::OCTET_WIDTH],
        ) else {
            break;
        };
        timing.record_run(
            isa,
            crate::quant::qjl32::OCTET_WIDTH,
            u64::try_from(block_started.elapsed().as_nanos()).unwrap_or(u64::MAX),
            true,
        );
        block_start += crate::quant::qjl32::OCTET_WIDTH;
    }

    if block_start >= candidates.len() {
        return;
    }

    let scalar_started = Instant::now();
    for ((code, gamma), out_score) in candidates[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
    {
        *out_score =
            crate::quant::qjl32::score_turboquant_qjl_scalar(quantizer, prepared, code, *gamma);
    }
    timing.scalar_candidates += candidates.len() - block_start;
    timing.scalar_elapsed_nanos = timing
        .scalar_elapsed_nanos
        .saturating_add(u64::try_from(scalar_started.elapsed().as_nanos()).unwrap_or(u64::MAX));
}

fn validate_turboquant_qjl_batch_shapes<Id>(
    original_dim: usize,
    batch: &CandidateBatch<'_, Id>,
) -> Result<(), String> {
    for (candidate_index, payload) in batch.payloads().iter().enumerate() {
        validate_turboquant_qjl_meta(payload.meta)?;
        crate::quant::qjl32::validate_code_shape(candidate_index, original_dim, payload.code)?;
    }
    Ok(())
}

fn validate_rabitq_bits1_meta(meta: CandidateMeta<'_>) -> Result<(), String> {
    match meta {
        CandidateMeta::None | CandidateMeta::RaBitQ => Ok(()),
        CandidateMeta::Gamma(0.0) => Ok(()),
        CandidateMeta::Gamma(_)
        | CandidateMeta::GammaAndResidualSigns { .. }
        | CandidateMeta::Binary
        | CandidateMeta::GroupedPq { .. } => {
            Err("RaBitQ bits=1 batch received incompatible candidate metadata".to_owned())
        }
    }
}

fn validate_grouped_pq_meta(
    meta: CandidateMeta<'_>,
    expected_group_count: usize,
) -> Result<(), String> {
    match meta {
        CandidateMeta::GroupedPq { group_count } if group_count == expected_group_count => Ok(()),
        CandidateMeta::GroupedPq { .. } => {
            Err("grouped-PQ batch received candidate group count mismatch".to_owned())
        }
        CandidateMeta::None
        | CandidateMeta::Gamma(_)
        | CandidateMeta::GammaAndResidualSigns { .. }
        | CandidateMeta::Binary
        | CandidateMeta::RaBitQ => {
            Err("grouped-PQ batch received incompatible candidate metadata".to_owned())
        }
    }
}

fn validate_turboquant_no_qjl_4bit_meta(meta: CandidateMeta<'_>) -> Result<(), String> {
    match meta {
        CandidateMeta::None | CandidateMeta::Gamma(_) => Ok(()),
        CandidateMeta::GammaAndResidualSigns { .. }
        | CandidateMeta::Binary
        | CandidateMeta::RaBitQ
        | CandidateMeta::GroupedPq { .. } => {
            Err("TurboQuant no-QJL 4-bit batch received incompatible candidate metadata".to_owned())
        }
    }
}

fn validate_turboquant_qjl_meta(meta: CandidateMeta<'_>) -> Result<f32, String> {
    match meta {
        CandidateMeta::Gamma(gamma) => Ok(gamma),
        CandidateMeta::GammaAndResidualSigns { gamma, .. } => Ok(gamma),
        CandidateMeta::None
        | CandidateMeta::Binary
        | CandidateMeta::RaBitQ
        | CandidateMeta::GroupedPq { .. } => {
            Err("TurboQuant QJL batch received incompatible candidate metadata".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlockKernelCounterKey, CandidateBatch, CandidateBatchScoringSurface, CandidateMeta,
        CandidatePayload,
    };
    use crate::am::common::quant_codec::QuantCodecKind;
    use crate::quant::grouped_pq::{grouped_pq_score_f32, pack_grouped_pq_nibbles};
    use crate::quant::isa::Isa;

    #[test]
    fn candidate_batch_preserves_ids_and_payloads() {
        let first = [1_u8, 2, 3];
        let second = [4_u8, 5, 6];
        let mut batch = CandidateBatch::with_capacity(2);

        batch
            .push(10_u32, CandidatePayload::new(&first, CandidateMeta::None))
            .unwrap();
        batch
            .push(
                11_u32,
                CandidatePayload::new(&second, CandidateMeta::Gamma(1.25)),
            )
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
        assert_eq!(batch.ids(), &[10, 11]);
        assert_eq!(batch.payloads()[0].code, &first);
        assert_eq!(batch.payloads()[1].meta, CandidateMeta::Gamma(1.25));
    }

    #[test]
    fn candidate_batch_rejects_capacity_overflow() {
        let code = [1_u8];
        let mut batch = CandidateBatch::with_capacity(1);

        batch
            .push(0_u32, CandidatePayload::new(&code, CandidateMeta::None))
            .unwrap();

        assert!(batch
            .push(1_u32, CandidatePayload::new(&code, CandidateMeta::None))
            .is_err());
    }

    #[test]
    fn turboquant_lut_batch_matches_scalar_tail() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 31);
        let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);
        let encoded: Vec<_> = (0..39)
            .map(|seed| quantizer.encode(&random_unit_vector(1536, seed)).mse_packed)
            .collect();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(index, CandidatePayload::new(payload, CandidateMeta::None))
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_turboquant_no_qjl_4bit_batch(&quantizer, &prepared, &batch, &mut batch_scores)
            .unwrap();

        for (payload, score) in encoded.iter().zip(batch_scores.iter()) {
            let scalar = quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared, payload);
            assert_eq!(score.to_bits(), scalar.to_bits());
        }
    }

    #[test]
    fn turboquant_lut_batch_records_surface_counters() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 131);
        let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);
        let encoded: Vec<_> = (0..39)
            .map(|seed| {
                quantizer
                    .encode(&random_unit_vector(1536, seed + 200))
                    .mse_packed
            })
            .collect();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(index, CandidatePayload::new(payload, CandidateMeta::None))
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_turboquant_no_qjl_4bit_batch_for(
            CandidateBatchScoringSurface::Ivf,
            &quantizer,
            &prepared,
            &batch,
            &mut batch_scores,
        )
        .unwrap();

        let snapshots = super::candidate_batch_scoring_snapshots();
        let ivf = snapshots
            .iter()
            .find(|snapshot| snapshot.surface == "ivf")
            .unwrap();
        assert_eq!(ivf.flushes, 1);
        assert_eq!(ivf.candidates, 39);
        assert_eq!(ivf.lut32_flushes, 1);
        assert_eq!(ivf.lut32_candidates, 32);
        let block_snapshots = super::block_kernel_scoring_snapshots();
        let block = block_snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "ivf"
                    && snapshot.quant_kind == "turboquant"
                    && snapshot.isa == "scalar"
            })
            .unwrap();
        assert_eq!(block.surface, "ivf");
        assert_eq!(block.quant_kind, "turboquant");
        assert_eq!(block.isa, "scalar");
        assert_eq!(block.flushes, 2);
        assert_eq!(block.candidates, 39);
        assert_eq!(
            block.elapsed_nanos,
            block.kernel_elapsed_nanos + block.scalar_elapsed_nanos
        );
        assert_eq!(block.kernel_flushes, 1);
        assert_eq!(block.kernel_candidates, 32);
        assert_eq!(block.scalar_flushes, 1);
        assert_eq!(block.scalar_candidates, 7);
        let spire = snapshots
            .iter()
            .find(|snapshot| snapshot.surface == "spire")
            .unwrap();
        assert_eq!(spire.flushes, 0);

        super::reset_candidate_batch_scoring_counters();
        let reset_ivf = super::candidate_batch_scoring_snapshots()
            .into_iter()
            .find(|snapshot| snapshot.surface == "ivf")
            .unwrap();
        assert_eq!(reset_ivf.flushes, 0);
        assert_eq!(reset_ivf.candidates, 0);
        assert!(super::block_kernel_scoring_snapshots().is_empty());
    }

    #[test]
    fn grouped_pq_batch_records_block_and_scalar_tail_counters() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let group_count = 16;
        let lut = grouped_pq_lut(group_count);
        let codes: Vec<Vec<u8>> = (0..39)
            .map(|seed| grouped_pq_code(group_count, seed as u8))
            .collect();
        let mut batch = CandidateBatch::with_capacity(codes.len());
        for (index, code) in codes.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(code, CandidateMeta::GroupedPq { group_count }),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_grouped_pq_batch_for(
            CandidateBatchScoringSurface::Ivf,
            &lut,
            group_count,
            &batch,
            &mut batch_scores,
        )
        .unwrap();

        for (code, score) in codes.iter().zip(batch_scores.iter()) {
            assert_eq!(
                score.to_bits(),
                grouped_pq_score_f32(&lut, group_count, code).to_bits()
            );
        }

        let block_snapshots = super::block_kernel_scoring_snapshots();
        let grouped: Vec<_> = block_snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "ivf" && snapshot.quant_kind == "grouped_pq")
            .collect();
        assert!(!grouped.is_empty());
        let kernel_candidates = grouped
            .iter()
            .map(|snapshot| snapshot.kernel_candidates)
            .sum::<u64>();
        let scalar_candidates = grouped
            .iter()
            .map(|snapshot| snapshot.scalar_candidates)
            .sum::<u64>();
        assert_eq!(kernel_candidates + scalar_candidates, 39);
        assert!(kernel_candidates >= 32);
        assert_eq!(
            grouped
                .iter()
                .map(|snapshot| snapshot.candidates)
                .sum::<u64>(),
            39
        );
        if grouped.iter().any(|snapshot| snapshot.isa != "scalar") {
            assert_eq!(kernel_candidates, 39);
            assert_eq!(scalar_candidates, 0);
        } else {
            assert_eq!(kernel_candidates, 32);
            assert_eq!(scalar_candidates, 7);
        }

        super::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn grouped_pq_batch_shape_error_scores_nothing_and_records_no_counters() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let group_count = 16;
        let lut = grouped_pq_lut(group_count);
        let mut codes: Vec<Vec<u8>> = (0..39)
            .map(|seed| grouped_pq_code(group_count, seed as u8))
            .collect();
        codes[33].pop();
        let mut batch = CandidateBatch::with_capacity(codes.len());
        for (index, code) in codes.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(code, CandidateMeta::GroupedPq { group_count }),
                )
                .unwrap();
        }
        let sentinel = -12_345.25_f32;
        let mut batch_scores = vec![sentinel; batch.len()];

        let err = super::score_grouped_pq_batch_for(
            CandidateBatchScoringSurface::Ivf,
            &lut,
            group_count,
            &batch,
            &mut batch_scores,
        )
        .unwrap_err();

        assert!(err.contains("grouped_pq_block code 33 too short"));
        assert!(batch_scores
            .iter()
            .all(|score| score.to_bits() == sentinel.to_bits()));
        assert!(super::block_kernel_scoring_snapshots().is_empty());
        assert!(super::candidate_batch_scoring_snapshots()
            .iter()
            .all(|snapshot| snapshot.flushes == 0 && snapshot.candidates == 0));

        super::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn block_kernel_counter_api_records_scalar_tail_under_scalar_isa() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        super::record_block_kernel_score(
            BlockKernelCounterKey {
                surface: CandidateBatchScoringSurface::Spire,
                quant_kind: QuantCodecKind::TurboQuant,
                isa: Isa::Sve2,
            },
            32,
            100,
        );
        super::record_block_scalar_score_for(
            CandidateBatchScoringSurface::Spire,
            QuantCodecKind::TurboQuant,
            7,
            20,
        );

        let snapshots = super::block_kernel_scoring_snapshots();
        let sve2 = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "spire"
                    && snapshot.quant_kind == "turboquant"
                    && snapshot.isa == "sve2"
            })
            .unwrap();
        assert_eq!(sve2.flushes, 1);
        assert_eq!(sve2.candidates, 32);
        assert_eq!(sve2.elapsed_nanos, 100);
        assert_eq!(sve2.kernel_flushes, 1);
        assert_eq!(sve2.kernel_candidates, 32);
        assert_eq!(sve2.kernel_elapsed_nanos, 100);
        assert_eq!(sve2.scalar_flushes, 0);
        assert_eq!(sve2.scalar_candidates, 0);

        let scalar = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "spire"
                    && snapshot.quant_kind == "turboquant"
                    && snapshot.isa == "scalar"
            })
            .unwrap();
        assert_eq!(scalar.flushes, 1);
        assert_eq!(scalar.candidates, 7);
        assert_eq!(scalar.elapsed_nanos, 20);
        assert_eq!(scalar.kernel_flushes, 0);
        assert_eq!(scalar.scalar_flushes, 1);
        assert_eq!(scalar.scalar_candidates, 7);
        assert_eq!(scalar.scalar_elapsed_nanos, 20);
    }

    #[test]
    fn block_kernel_counter_api_keeps_turboquant_exact_modes_distinct() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        super::record_block_scalar_score_for(
            CandidateBatchScoringSurface::Hnsw,
            QuantCodecKind::TurboQuantTiledLut,
            11,
            101,
        );
        super::record_flush_width(
            CandidateBatchScoringSurface::Hnsw,
            QuantCodecKind::TurboQuantTiledLut,
            Isa::Scalar,
            11,
        );
        super::record_block_scalar_score_for(
            CandidateBatchScoringSurface::Hnsw,
            QuantCodecKind::TurboQuantInt8,
            13,
            103,
        );
        super::record_flush_width(
            CandidateBatchScoringSurface::Hnsw,
            QuantCodecKind::TurboQuantInt8,
            Isa::Scalar,
            13,
        );

        let block_snapshots = super::block_kernel_scoring_snapshots();
        let tiled = block_snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "hnsw"
                    && snapshot.quant_kind == "turboquant_tiled_lut"
                    && snapshot.isa == "scalar"
            })
            .unwrap();
        assert_eq!(tiled.scalar_candidates, 11);
        let int8 = block_snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "hnsw"
                    && snapshot.quant_kind == "turboquant_int8"
                    && snapshot.isa == "scalar"
            })
            .unwrap();
        assert_eq!(int8.scalar_candidates, 13);

        let hnsw = super::candidate_batch_scoring_snapshots()
            .into_iter()
            .find(|snapshot| snapshot.surface == "hnsw")
            .unwrap();
        assert_eq!(hnsw.candidates, 24);
        assert_eq!(hnsw.lut32_candidates, 0);
        assert_eq!(hnsw.lut32_flushes, 0);

        super::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn rabitq_bits1_batch_records_block_and_tail_counters() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let dimensions = 40;
        let quantizer =
            crate::quant::rabitq::RaBitQQuantizer::cached_seeded_srht_bits(dimensions, 42, 1)
                .unwrap();
        let query = random_unit_vector(dimensions, 331);
        let prepared = quantizer.prepare_estimator(&query);
        let block_prepared = prepared
            .bits1_block_prepared(crate::quant::Quantizer::code_len(quantizer.as_ref()))
            .unwrap();
        let encoded: Vec<_> = (0..39)
            .map(|seed| {
                crate::quant::Quantizer::encode_code(
                    quantizer.as_ref(),
                    &random_unit_vector(dimensions, seed + 400),
                )
                .into_vec()
            })
            .collect();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(index, CandidatePayload::new(payload, CandidateMeta::RaBitQ))
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_rabitq_bits1_batch_for(
            CandidateBatchScoringSurface::Diskann,
            block_prepared,
            &batch,
            &mut batch_scores,
        )
        .unwrap();

        // The first 32 candidates go through the dispatched block kernel and
        // the 7-candidate tail through the partial dispatch; reproduce both
        // calls directly so the expectation matches whichever ISA backend
        // this host selects.
        let block_codes: [&[u8]; 32] = encoded[..32]
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut expected_block_scores = vec![0.0; 32];
        let kernel_isa = crate::quant::rabitq32::score_rabitq_bits1_block32(
            block_prepared,
            block_codes,
            &mut expected_block_scores,
        );
        let tail_codes: Vec<&[u8]> = encoded[32..].iter().map(Vec::as_slice).collect();
        let mut expected_tail_scores = vec![0.0; tail_codes.len()];
        let tail_isa = crate::quant::rabitq32::score_rabitq_bits1_partial(
            block_prepared,
            &tail_codes,
            &mut expected_tail_scores,
        );
        for (score, expected) in batch_scores[..32].iter().zip(expected_block_scores.iter()) {
            assert_eq!(score.to_bits(), expected.to_bits());
        }
        for (score, expected) in batch_scores[32..].iter().zip(expected_tail_scores.iter()) {
            assert_eq!(score.to_bits(), expected.to_bits());
        }

        let block_snapshots = super::block_kernel_scoring_snapshots();
        let kernel_row = block_snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "diskann"
                    && snapshot.quant_kind == "rabitq"
                    && snapshot.isa == kernel_isa.label()
            })
            .unwrap();
        assert!(kernel_row.kernel_flushes >= 1);
        assert!(kernel_row.kernel_candidates >= 32);
        // Width histogram: the wrapper records one width sample per batch
        // (39 candidates here -> the >=32 bucket).
        assert_eq!(kernel_row.width_ge32_flushes, 1);
        assert_eq!(kernel_row.width_lt8_flushes, 0);
        if tail_isa == Isa::Scalar {
            let tail_row = block_snapshots
                .iter()
                .find(|snapshot| {
                    snapshot.surface == "diskann"
                        && snapshot.quant_kind == "rabitq"
                        && snapshot.isa == "scalar"
                })
                .unwrap();
            assert_eq!(tail_row.scalar_candidates, 7);
        }
        let total_candidates: u64 = block_snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "diskann" && snapshot.quant_kind == "rabitq")
            .map(|snapshot| snapshot.candidates)
            .sum();
        assert_eq!(total_candidates, 39);
        super::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn rabitq_bits1_batch_below_width_uses_partial_dispatch() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let dimensions = 40;
        let quantizer =
            crate::quant::rabitq::RaBitQQuantizer::cached_seeded_srht_bits(dimensions, 42, 1)
                .unwrap();
        let query = random_unit_vector(dimensions, 733);
        let prepared = quantizer.prepare_estimator(&query);
        let block_prepared = prepared
            .bits1_block_prepared(crate::quant::Quantizer::code_len(quantizer.as_ref()))
            .unwrap();
        let encoded: Vec<_> = (0..7)
            .map(|seed| {
                crate::quant::Quantizer::encode_code(
                    quantizer.as_ref(),
                    &random_unit_vector(dimensions, seed + 500),
                )
                .into_vec()
            })
            .collect();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(index, CandidatePayload::new(payload, CandidateMeta::RaBitQ))
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_rabitq_bits1_batch_for(
            CandidateBatchScoringSurface::Ivf,
            block_prepared,
            &batch,
            &mut batch_scores,
        )
        .unwrap();

        // Sub-width batches go through the partial dispatch; reproduce that
        // call directly so the expectation matches whichever ISA backend
        // this host selects.
        let codes: Vec<&[u8]> = encoded.iter().map(Vec::as_slice).collect();
        let mut expected_scores = vec![0.0; codes.len()];
        let partial_isa = crate::quant::rabitq32::score_rabitq_bits1_partial(
            block_prepared,
            &codes,
            &mut expected_scores,
        );
        for (score, expected) in batch_scores.iter().zip(expected_scores.iter()) {
            assert_eq!(score.to_bits(), expected.to_bits());
        }
        let block_snapshots = super::block_kernel_scoring_snapshots();
        let row = block_snapshots
            .iter()
            .find(|snapshot| {
                snapshot.surface == "ivf"
                    && snapshot.quant_kind == "rabitq"
                    && snapshot.isa == partial_isa.label()
            })
            .unwrap();
        assert_eq!(row.flushes, 1);
        assert_eq!(row.candidates, 7);
        if partial_isa == Isa::Scalar {
            assert_eq!(row.kernel_flushes, 0);
            assert_eq!(row.scalar_flushes, 1);
            assert_eq!(row.scalar_candidates, 7);
        } else {
            assert_eq!(row.kernel_flushes, 1);
            assert_eq!(row.kernel_candidates, 7);
            assert_eq!(row.scalar_candidates, 0);
        }
        super::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn rabitq_bits1_batch_shape_mismatch_rejects_before_counters() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        let dimensions = 40;
        let quantizer =
            crate::quant::rabitq::RaBitQQuantizer::cached_seeded_srht_bits(dimensions, 42, 1)
                .unwrap();
        let query = random_unit_vector(dimensions, 877);
        let prepared = quantizer.prepare_estimator(&query);
        let block_prepared = prepared
            .bits1_block_prepared(crate::quant::Quantizer::code_len(quantizer.as_ref()))
            .unwrap();
        let encoded = crate::quant::Quantizer::encode_code(
            quantizer.as_ref(),
            &random_unit_vector(dimensions, 901),
        )
        .into_vec();

        let truncated = &encoded[..encoded.len() - 1];
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                0_usize,
                CandidatePayload::new(truncated, CandidateMeta::RaBitQ),
            )
            .unwrap();
        let mut batch_scores = vec![0.0; 1];
        assert!(super::score_rabitq_bits1_batch_for(
            CandidateBatchScoringSurface::Ivf,
            block_prepared,
            &batch,
            &mut batch_scores,
        )
        .is_err());

        let mut meta_batch = CandidateBatch::with_capacity(1);
        meta_batch
            .push(
                0_usize,
                CandidatePayload::new(encoded.as_slice(), CandidateMeta::Binary),
            )
            .unwrap();
        assert!(super::score_rabitq_bits1_batch_for(
            CandidateBatchScoringSurface::Ivf,
            block_prepared,
            &meta_batch,
            &mut batch_scores,
        )
        .is_err());

        let mut count_batch = CandidateBatch::with_capacity(1);
        count_batch
            .push(
                0_usize,
                CandidatePayload::new(encoded.as_slice(), CandidateMeta::RaBitQ),
            )
            .unwrap();
        let mut wrong_len_scores = vec![0.0; 2];
        assert!(super::score_rabitq_bits1_batch_for(
            CandidateBatchScoringSurface::Ivf,
            block_prepared,
            &count_batch,
            &mut wrong_len_scores,
        )
        .is_err());

        assert!(super::block_kernel_scoring_snapshots().is_empty());
    }

    fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        let mut state = seed ^ 0xA076_1D64_78BD_642F;
        let mut values = Vec::with_capacity(dim);
        let mut norm_sq = 0.0_f32;
        for _ in 0..dim {
            state = state
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0xBF58_476D_1CE4_E5B9);
            let raw = ((state >> 32) as u32) as f32 / (u32::MAX as f32);
            let value = raw * 2.0 - 1.0;
            values.push(value);
            norm_sq += value * value;
        }
        let norm = norm_sq.sqrt().max(f32::MIN_POSITIVE);
        for value in &mut values {
            *value /= norm;
        }
        values
    }

    fn grouped_pq_lut(group_count: usize) -> Vec<f32> {
        (0..group_count * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS)
            .map(|index| ((index as i32 % 37) - 18) as f32 * 0.03125 + 0.000_17)
            .collect()
    }

    fn grouped_pq_code(group_count: usize, seed: u8) -> Vec<u8> {
        let indices: Vec<u8> = (0..group_count)
            .map(|group| {
                seed.wrapping_add((group as u8).wrapping_mul(5))
                    .wrapping_add((group as u8) >> 1)
                    & 0x0F
            })
            .collect();
        pack_grouped_pq_nibbles(&indices)
    }
}
