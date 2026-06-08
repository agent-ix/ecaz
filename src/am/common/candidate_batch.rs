use crate::quant::prod::{PreparedLutNoQjl4BitQuery, ProdQuantizer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateBatchScoringSurface {
    Spire,
    Ivf,
    Hnsw,
    Unknown,
}

impl CandidateBatchScoringSurface {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Spire => "spire",
            Self::Ivf => "ivf",
            Self::Hnsw => "hnsw",
            Self::Unknown => "unknown",
        }
    }

    fn counters(self) -> &'static SurfaceCounters {
        match self {
            Self::Spire => &SPIRE_COUNTERS,
            Self::Ivf => &IVF_COUNTERS,
            Self::Hnsw => &HNSW_COUNTERS,
            Self::Unknown => &UNKNOWN_COUNTERS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CandidateBatchScoringSnapshot {
    pub(crate) surface: &'static str,
    pub(crate) flushes: u64,
    pub(crate) candidates: u64,
    pub(crate) elapsed_nanos: u64,
    pub(crate) lut32_flushes: u64,
    pub(crate) lut32_candidates: u64,
}

struct SurfaceCounters {
    flushes: AtomicU64,
    candidates: AtomicU64,
    elapsed_nanos: AtomicU64,
    lut32_flushes: AtomicU64,
    lut32_candidates: AtomicU64,
}

impl SurfaceCounters {
    const fn new() -> Self {
        Self {
            flushes: AtomicU64::new(0),
            candidates: AtomicU64::new(0),
            elapsed_nanos: AtomicU64::new(0),
            lut32_flushes: AtomicU64::new(0),
            lut32_candidates: AtomicU64::new(0),
        }
    }

    fn reset(&self) {
        self.flushes.store(0, Ordering::Relaxed);
        self.candidates.store(0, Ordering::Relaxed);
        self.elapsed_nanos.store(0, Ordering::Relaxed);
        self.lut32_flushes.store(0, Ordering::Relaxed);
        self.lut32_candidates.store(0, Ordering::Relaxed);
    }

    fn record(&self, candidate_count: usize, elapsed_nanos: u64, used_lut32: bool) {
        let candidate_count =
            u64::try_from(candidate_count).expect("candidate count should fit in u64");
        self.flushes.fetch_add(1, Ordering::Relaxed);
        self.candidates
            .fetch_add(candidate_count, Ordering::Relaxed);
        self.elapsed_nanos
            .fetch_add(elapsed_nanos, Ordering::Relaxed);
        if used_lut32 {
            self.lut32_flushes.fetch_add(1, Ordering::Relaxed);
            self.lut32_candidates
                .fetch_add(candidate_count, Ordering::Relaxed);
        }
    }

    fn snapshot(&self, surface: CandidateBatchScoringSurface) -> CandidateBatchScoringSnapshot {
        CandidateBatchScoringSnapshot {
            surface: surface.label(),
            flushes: self.flushes.load(Ordering::Relaxed),
            candidates: self.candidates.load(Ordering::Relaxed),
            elapsed_nanos: self.elapsed_nanos.load(Ordering::Relaxed),
            lut32_flushes: self.lut32_flushes.load(Ordering::Relaxed),
            lut32_candidates: self.lut32_candidates.load(Ordering::Relaxed),
        }
    }
}

static SPIRE_COUNTERS: SurfaceCounters = SurfaceCounters::new();
static IVF_COUNTERS: SurfaceCounters = SurfaceCounters::new();
static HNSW_COUNTERS: SurfaceCounters = SurfaceCounters::new();
static UNKNOWN_COUNTERS: SurfaceCounters = SurfaceCounters::new();

pub(crate) fn reset_candidate_batch_scoring_counters() {
    for counters in [
        &SPIRE_COUNTERS,
        &IVF_COUNTERS,
        &HNSW_COUNTERS,
        &UNKNOWN_COUNTERS,
    ] {
        counters.reset();
    }
}

pub(crate) fn candidate_batch_scoring_snapshots() -> [CandidateBatchScoringSnapshot; 4] {
    [
        SPIRE_COUNTERS.snapshot(CandidateBatchScoringSurface::Spire),
        IVF_COUNTERS.snapshot(CandidateBatchScoringSurface::Ivf),
        HNSW_COUNTERS.snapshot(CandidateBatchScoringSurface::Hnsw),
        UNKNOWN_COUNTERS.snapshot(CandidateBatchScoringSurface::Unknown),
    ]
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

pub(crate) fn score_turboquant_no_qjl_4bit_batch_for<Id>(
    surface: CandidateBatchScoringSurface,
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    let started = Instant::now();
    let used_lut32 = batch.len() >= crate::quant::lut32::BLOCK_WIDTH;
    let result = score_turboquant_no_qjl_4bit_batch_inner(quantizer, prepared, batch, out_scores);
    if result.is_ok() {
        let elapsed_nanos = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        surface
            .counters()
            .record(batch.len(), elapsed_nanos, used_lut32);
    }
    result
}

fn score_turboquant_no_qjl_4bit_batch_inner<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "candidate batch score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }

    for payload in batch.payloads() {
        match payload.meta {
            CandidateMeta::None | CandidateMeta::Gamma(_) => {}
            CandidateMeta::GammaAndResidualSigns { .. }
            | CandidateMeta::Binary
            | CandidateMeta::RaBitQ
            | CandidateMeta::GroupedPq { .. } => {
                return Err(
                    "TurboQuant no-QJL 4-bit batch received incompatible candidate metadata"
                        .to_owned(),
                );
            }
        }
    }

    if batch.len() >= crate::quant::lut32::BLOCK_WIDTH {
        let mse_codes: Vec<&[u8]> = batch
            .payloads()
            .iter()
            .map(|payload| quantizer.mse_code_bytes_no_qjl_4bit(payload.code))
            .collect();
        return crate::quant::lut32::score_lut_no_qjl_4bit_batch(
            &prepared.lut,
            quantizer.original_dim,
            &mse_codes,
            out_scores,
        );
    }

    for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
        *out_score = quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, payload.code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CandidateBatch, CandidateBatchScoringSurface, CandidateMeta, CandidatePayload};

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
        assert_eq!(ivf.lut32_candidates, 39);
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
}
