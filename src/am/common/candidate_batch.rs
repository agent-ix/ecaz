use super::quant_codec::QuantCodecKind;
use crate::quant::grouped_pq::grouped_pq_score_f32;
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedLutNoQjl4BitQuery, ProdQuantizer};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
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
    Diskann,
    Hnsw,
    Unknown,
}

impl CandidateBatchScoringSurface {
    const TASK87_ALL: [Self; 4] = [Self::Spire, Self::Ivf, Self::Hnsw, Self::Unknown];
    const BLOCK_KERNEL_ALL: [Self; 5] = [
        Self::Spire,
        Self::Ivf,
        Self::Diskann,
        Self::Hnsw,
        Self::Unknown,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Spire => "spire",
            Self::Ivf => "ivf",
            Self::Diskann => "diskann",
            Self::Hnsw => "hnsw",
            Self::Unknown => "unknown",
        }
    }

    fn task87_all() -> [Self; 4] {
        Self::TASK87_ALL
    }

    fn block_kernel_all() -> [Self; 5] {
        Self::BLOCK_KERNEL_ALL
    }

    fn index(self) -> usize {
        match self {
            Self::Spire => 0,
            Self::Ivf => 1,
            Self::Diskann => 2,
            Self::Hnsw => 3,
            Self::Unknown => 4,
        }
    }
}

fn quant_index(quant_kind: QuantCodecKind) -> usize {
    match quant_kind {
        QuantCodecKind::TurboQuant => 0,
        QuantCodecKind::TurboQuantQjl => 1,
        QuantCodecKind::RaBitQ => 2,
        QuantCodecKind::GroupedPq => 3,
        QuantCodecKind::Binary => 4,
    }
}

fn isa_index(isa: Isa) -> usize {
    match isa {
        Isa::Scalar => 0,
        Isa::Neon => 1,
        Isa::Sve => 2,
        Isa::Sve2 => 3,
        Isa::Avx2 => 4,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockKernelCounterKey {
    pub(crate) surface: CandidateBatchScoringSurface,
    pub(crate) quant_kind: QuantCodecKind,
    pub(crate) isa: Isa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockKernelScoringSnapshot {
    pub(crate) surface: &'static str,
    pub(crate) quant_kind: &'static str,
    pub(crate) isa: &'static str,
    pub(crate) flushes: u64,
    pub(crate) candidates: u64,
    pub(crate) elapsed_nanos: u64,
    pub(crate) kernel_flushes: u64,
    pub(crate) kernel_candidates: u64,
    pub(crate) kernel_elapsed_nanos: u64,
    pub(crate) scalar_flushes: u64,
    pub(crate) scalar_candidates: u64,
    pub(crate) scalar_elapsed_nanos: u64,
}

struct BlockKernelCounters {
    flushes: AtomicU64,
    candidates: AtomicU64,
    elapsed_nanos: AtomicU64,
    kernel_flushes: AtomicU64,
    kernel_candidates: AtomicU64,
    kernel_elapsed_nanos: AtomicU64,
    scalar_flushes: AtomicU64,
    scalar_candidates: AtomicU64,
    scalar_elapsed_nanos: AtomicU64,
}

impl BlockKernelCounters {
    fn new() -> Self {
        Self {
            flushes: AtomicU64::new(0),
            candidates: AtomicU64::new(0),
            elapsed_nanos: AtomicU64::new(0),
            kernel_flushes: AtomicU64::new(0),
            kernel_candidates: AtomicU64::new(0),
            kernel_elapsed_nanos: AtomicU64::new(0),
            scalar_flushes: AtomicU64::new(0),
            scalar_candidates: AtomicU64::new(0),
            scalar_elapsed_nanos: AtomicU64::new(0),
        }
    }

    fn reset(&self) {
        self.flushes.store(0, Ordering::Relaxed);
        self.candidates.store(0, Ordering::Relaxed);
        self.elapsed_nanos.store(0, Ordering::Relaxed);
        self.kernel_flushes.store(0, Ordering::Relaxed);
        self.kernel_candidates.store(0, Ordering::Relaxed);
        self.kernel_elapsed_nanos.store(0, Ordering::Relaxed);
        self.scalar_flushes.store(0, Ordering::Relaxed);
        self.scalar_candidates.store(0, Ordering::Relaxed);
        self.scalar_elapsed_nanos.store(0, Ordering::Relaxed);
    }

    fn record_kernel(&self, candidate_count: u64, elapsed_nanos: u64) {
        self.flushes.fetch_add(1, Ordering::Relaxed);
        self.candidates
            .fetch_add(candidate_count, Ordering::Relaxed);
        self.elapsed_nanos
            .fetch_add(elapsed_nanos, Ordering::Relaxed);
        self.kernel_flushes.fetch_add(1, Ordering::Relaxed);
        self.kernel_candidates
            .fetch_add(candidate_count, Ordering::Relaxed);
        self.kernel_elapsed_nanos
            .fetch_add(elapsed_nanos, Ordering::Relaxed);
    }

    fn record_scalar(&self, candidate_count: u64, elapsed_nanos: u64) {
        self.flushes.fetch_add(1, Ordering::Relaxed);
        self.candidates
            .fetch_add(candidate_count, Ordering::Relaxed);
        self.elapsed_nanos
            .fetch_add(elapsed_nanos, Ordering::Relaxed);
        self.scalar_flushes.fetch_add(1, Ordering::Relaxed);
        self.scalar_candidates
            .fetch_add(candidate_count, Ordering::Relaxed);
        self.scalar_elapsed_nanos
            .fetch_add(elapsed_nanos, Ordering::Relaxed);
    }

    fn snapshot(&self, key: BlockKernelCounterKey) -> BlockKernelScoringSnapshot {
        BlockKernelScoringSnapshot {
            surface: key.surface.label(),
            quant_kind: key.quant_kind.label(),
            isa: key.isa.label(),
            flushes: self.flushes.load(Ordering::Relaxed),
            candidates: self.candidates.load(Ordering::Relaxed),
            elapsed_nanos: self.elapsed_nanos.load(Ordering::Relaxed),
            kernel_flushes: self.kernel_flushes.load(Ordering::Relaxed),
            kernel_candidates: self.kernel_candidates.load(Ordering::Relaxed),
            kernel_elapsed_nanos: self.kernel_elapsed_nanos.load(Ordering::Relaxed),
            scalar_flushes: self.scalar_flushes.load(Ordering::Relaxed),
            scalar_candidates: self.scalar_candidates.load(Ordering::Relaxed),
            scalar_elapsed_nanos: self.scalar_elapsed_nanos.load(Ordering::Relaxed),
        }
    }
}

const SURFACE_COUNT: usize = 5;
const QUANT_COUNT: usize = 5;
const ISA_COUNT: usize = 5;

static BLOCK_KERNEL_COUNTERS: OnceLock<Vec<BlockKernelCounters>> = OnceLock::new();

#[cfg(test)]
pub(crate) static CANDIDATE_BATCH_COUNTER_TEST_LOCK: std::sync::Mutex<()> =
    std::sync::Mutex::new(());

fn block_kernel_counters_storage() -> &'static [BlockKernelCounters] {
    BLOCK_KERNEL_COUNTERS
        .get_or_init(|| {
            (0..SURFACE_COUNT * QUANT_COUNT * ISA_COUNT)
                .map(|_| BlockKernelCounters::new())
                .collect()
        })
        .as_slice()
}

fn block_kernel_counter_index(key: BlockKernelCounterKey) -> usize {
    (key.surface.index() * QUANT_COUNT * ISA_COUNT)
        + (quant_index(key.quant_kind) * ISA_COUNT)
        + isa_index(key.isa)
}

fn block_kernel_counters(key: BlockKernelCounterKey) -> &'static BlockKernelCounters {
    &block_kernel_counters_storage()[block_kernel_counter_index(key)]
}

pub(crate) fn reset_candidate_batch_scoring_counters() {
    for counters in block_kernel_counters_storage() {
        counters.reset();
    }
}

pub(crate) fn candidate_batch_scoring_snapshots() -> [CandidateBatchScoringSnapshot; 4] {
    CandidateBatchScoringSurface::task87_all().map(candidate_batch_surface_snapshot)
}

pub(crate) fn block_kernel_scoring_snapshots() -> Vec<BlockKernelScoringSnapshot> {
    let mut snapshots = Vec::new();
    for surface in CandidateBatchScoringSurface::block_kernel_all() {
        for quant_kind in QuantCodecKind::ALL {
            for isa in Isa::ALL {
                let key = BlockKernelCounterKey {
                    surface,
                    quant_kind,
                    isa,
                };
                let snapshot = block_kernel_counters(key).snapshot(key);
                if snapshot.flushes > 0 {
                    snapshots.push(snapshot);
                }
            }
        }
    }
    snapshots
}

fn candidate_batch_surface_snapshot(
    surface: CandidateBatchScoringSurface,
) -> CandidateBatchScoringSnapshot {
    let mut flushes = 0;
    let mut candidates = 0;
    let mut elapsed_nanos = 0;
    let mut lut32_flushes = 0;
    let mut lut32_candidates = 0;
    for quant_kind in QuantCodecKind::ALL {
        for isa in Isa::ALL {
            let snapshot = block_kernel_counters(BlockKernelCounterKey {
                surface,
                quant_kind,
                isa,
            })
            .snapshot(BlockKernelCounterKey {
                surface,
                quant_kind,
                isa,
            });
            let compatibility_flushes = if quant_kind == QuantCodecKind::TurboQuant {
                snapshot.kernel_flushes.max(snapshot.scalar_flushes)
            } else {
                snapshot.flushes
            };
            flushes += compatibility_flushes;
            candidates += snapshot.candidates;
            elapsed_nanos += snapshot.elapsed_nanos;
            if quant_kind == QuantCodecKind::TurboQuant {
                lut32_flushes += snapshot.kernel_flushes;
                lut32_candidates += snapshot.kernel_candidates;
            }
        }
    }
    CandidateBatchScoringSnapshot {
        surface: surface.label(),
        flushes,
        candidates,
        elapsed_nanos,
        lut32_flushes,
        lut32_candidates,
    }
}

fn record_block_kernel_score(
    key: BlockKernelCounterKey,
    candidate_count: usize,
    elapsed_nanos: u64,
) {
    if candidate_count == 0 {
        return;
    }
    let candidate_count =
        u64::try_from(candidate_count).expect("candidate count should fit in u64");
    block_kernel_counters(key).record_kernel(candidate_count, elapsed_nanos);
}

pub(crate) fn record_block_scalar_score_for(
    surface: CandidateBatchScoringSurface,
    quant_kind: QuantCodecKind,
    candidate_count: usize,
    elapsed_nanos: u64,
) {
    if candidate_count == 0 {
        return;
    }
    let candidate_count =
        u64::try_from(candidate_count).expect("candidate count should fit in u64");
    let key = BlockKernelCounterKey {
        surface,
        quant_kind,
        isa: Isa::Scalar,
    };
    block_kernel_counters(key).record_scalar(candidate_count, elapsed_nanos);
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
    let result = score_turboquant_no_qjl_4bit_batch_inner(quantizer, prepared, batch, out_scores);
    if let Ok(timing) = &result {
        let key = BlockKernelCounterKey {
            surface,
            quant_kind: QuantCodecKind::TurboQuant,
            isa: timing.kernel_isa.unwrap_or(Isa::Scalar),
        };
        record_block_kernel_score(key, timing.kernel_candidates, timing.kernel_elapsed_nanos);
        record_block_scalar_score_for(
            surface,
            QuantCodecKind::TurboQuant,
            timing.scalar_candidates,
            timing.scalar_elapsed_nanos,
        );
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
        let key = BlockKernelCounterKey {
            surface,
            quant_kind: QuantCodecKind::GroupedPq,
            isa: timing.kernel_isa.unwrap_or(Isa::Scalar),
        };
        record_block_kernel_score(key, timing.kernel_candidates, timing.kernel_elapsed_nanos);
        record_block_scalar_score_for(
            surface,
            QuantCodecKind::GroupedPq,
            timing.scalar_candidates,
            timing.scalar_elapsed_nanos,
        );
    }
    result.map(|_| ())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BatchScoringTiming {
    kernel_isa: Option<Isa>,
    kernel_candidates: usize,
    kernel_elapsed_nanos: u64,
    scalar_candidates: usize,
    scalar_elapsed_nanos: u64,
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

    if batch.len() >= crate::quant::grouped_pq_block::BLOCK_WIDTH {
        return score_grouped_pq_batch_block32(lut, group_count, batch, out_scores);
    }

    let scalar_started = Instant::now();
    for (candidate_index, (payload, out_score)) in batch
        .payloads()
        .iter()
        .zip(out_scores.iter_mut())
        .enumerate()
    {
        validate_grouped_pq_meta(payload.meta, group_count)?;
        crate::quant::grouped_pq_block::validate_code_shape(
            candidate_index,
            group_count,
            payload.code,
        )?;
        *out_score = grouped_pq_score_f32(lut, group_count, payload.code);
    }
    Ok(BatchScoringTiming {
        scalar_candidates: batch.len(),
        scalar_elapsed_nanos: u64::try_from(scalar_started.elapsed().as_nanos())
            .unwrap_or(u64::MAX),
        ..BatchScoringTiming::default()
    })
}

fn score_grouped_pq_batch_block32<Id>(
    lut: &[f32],
    group_count: usize,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    let mut block_start = 0usize;
    let mut timing = BatchScoringTiming::default();
    while block_start + crate::quant::grouped_pq_block::BLOCK_WIDTH <= batch.len() {
        let block_started = Instant::now();
        let payloads = &batch.payloads()
            [block_start..block_start + crate::quant::grouped_pq_block::BLOCK_WIDTH];
        let mut codes = [&[][..]; crate::quant::grouped_pq_block::BLOCK_WIDTH];
        for (lane, payload) in payloads.iter().enumerate() {
            validate_grouped_pq_meta(payload.meta, group_count)?;
            crate::quant::grouped_pq_block::validate_code_shape(
                block_start + lane,
                group_count,
                payload.code,
            )?;
            codes[lane] = payload.code;
        }
        let isa = crate::quant::grouped_pq_block::score_grouped_pq_block32(
            lut,
            group_count,
            codes,
            &mut out_scores[block_start..block_start + crate::quant::grouped_pq_block::BLOCK_WIDTH],
        );
        timing.kernel_isa = Some(isa);
        timing.kernel_candidates += crate::quant::grouped_pq_block::BLOCK_WIDTH;
        timing.kernel_elapsed_nanos = timing
            .kernel_elapsed_nanos
            .saturating_add(u64::try_from(block_started.elapsed().as_nanos()).unwrap_or(u64::MAX));
        block_start += crate::quant::grouped_pq_block::BLOCK_WIDTH;
    }

    let scalar_started = Instant::now();
    for (candidate_index, (payload, out_score)) in batch.payloads()[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
        .enumerate()
    {
        validate_grouped_pq_meta(payload.meta, group_count)?;
        crate::quant::grouped_pq_block::validate_code_shape(
            block_start + candidate_index,
            group_count,
            payload.code,
        )?;
        timing.scalar_candidates += 1;
        *out_score =
            crate::quant::grouped_pq_block::score_grouped_pq_scalar(lut, group_count, payload.code);
    }
    if timing.scalar_candidates > 0 {
        timing.scalar_elapsed_nanos =
            u64::try_from(scalar_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    }

    Ok(timing)
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

    if batch.len() >= crate::quant::lut32::BLOCK_WIDTH {
        return score_turboquant_no_qjl_4bit_batch_lut32(quantizer, prepared, batch, out_scores);
    }

    let scalar_started = Instant::now();
    for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
        validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
        *out_score = quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, payload.code);
    }
    Ok(BatchScoringTiming {
        scalar_candidates: batch.len(),
        scalar_elapsed_nanos: u64::try_from(scalar_started.elapsed().as_nanos())
            .unwrap_or(u64::MAX),
        ..BatchScoringTiming::default()
    })
}

fn score_turboquant_no_qjl_4bit_batch_lut32<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<BatchScoringTiming, String> {
    crate::quant::lut32::validate_lut_shape(&prepared.lut, quantizer.original_dim)?;

    let mut block_start = 0usize;
    let mut timing = BatchScoringTiming::default();
    while block_start + crate::quant::lut32::BLOCK_WIDTH <= batch.len() {
        let block_started = Instant::now();
        let payloads =
            &batch.payloads()[block_start..block_start + crate::quant::lut32::BLOCK_WIDTH];
        let mut mse_codes = [&[][..]; crate::quant::lut32::BLOCK_WIDTH];
        for (lane, payload) in payloads.iter().enumerate() {
            validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
            let mse_code = quantizer.mse_code_bytes_no_qjl_4bit(payload.code);
            crate::quant::lut32::validate_mse_code_shape(
                block_start + lane,
                quantizer.original_dim,
                mse_code,
            )?;
            mse_codes[lane] = mse_code;
        }
        let isa = crate::quant::lut32::score_lut_no_qjl_4bit_block32(
            &prepared.lut,
            quantizer.original_dim,
            mse_codes,
            &mut out_scores[block_start..block_start + crate::quant::lut32::BLOCK_WIDTH],
        );
        timing.kernel_isa = Some(isa);
        timing.kernel_candidates += crate::quant::lut32::BLOCK_WIDTH;
        timing.kernel_elapsed_nanos = timing
            .kernel_elapsed_nanos
            .saturating_add(u64::try_from(block_started.elapsed().as_nanos()).unwrap_or(u64::MAX));
        block_start += crate::quant::lut32::BLOCK_WIDTH;
    }

    let scalar_started = Instant::now();
    for (candidate_index, (payload, out_score)) in batch.payloads()[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
        .enumerate()
    {
        validate_turboquant_no_qjl_4bit_meta(payload.meta)?;
        let mse_code = quantizer.mse_code_bytes_no_qjl_4bit(payload.code);
        crate::quant::lut32::validate_mse_code_shape(
            block_start + candidate_index,
            quantizer.original_dim,
            mse_code,
        )?;
        timing.scalar_candidates += 1;
        *out_score = crate::quant::lut32::score_lut_no_qjl_4bit_scalar(
            &prepared.lut,
            quantizer.original_dim,
            mse_code,
        );
    }
    if timing.scalar_candidates > 0 {
        timing.scalar_elapsed_nanos =
            u64::try_from(scalar_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    }

    Ok(timing)
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
        assert_eq!(
            grouped
                .iter()
                .map(|snapshot| snapshot.kernel_candidates)
                .sum::<u64>(),
            32
        );
        assert_eq!(
            grouped
                .iter()
                .map(|snapshot| snapshot.scalar_candidates)
                .sum::<u64>(),
            7
        );
        assert_eq!(
            grouped
                .iter()
                .map(|snapshot| snapshot.candidates)
                .sum::<u64>(),
            39
        );
        assert!(grouped
            .iter()
            .any(|snapshot| snapshot.isa == "scalar" && snapshot.scalar_candidates == 7));

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
    fn turboquant_qjl_counter_kind_has_distinct_direct_rows_without_lut32_compat() {
        let _guard = super::CANDIDATE_BATCH_COUNTER_TEST_LOCK.lock().unwrap();
        super::reset_candidate_batch_scoring_counters();
        super::record_block_kernel_score(
            BlockKernelCounterKey {
                surface: CandidateBatchScoringSurface::Ivf,
                quant_kind: QuantCodecKind::TurboQuantQjl,
                isa: Isa::Avx2,
            },
            32,
            100,
        );
        super::record_block_scalar_score_for(
            CandidateBatchScoringSurface::Ivf,
            QuantCodecKind::TurboQuantQjl,
            3,
            20,
        );

        let block_snapshots = super::block_kernel_scoring_snapshots();
        let qjl: Vec<_> = block_snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "ivf" && snapshot.quant_kind == "turboquant_qjl")
            .collect();
        assert_eq!(qjl.len(), 2);
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.isa == "avx2" && snapshot.kernel_candidates == 32));
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.isa == "scalar" && snapshot.scalar_candidates == 3));

        let task87_ivf = super::candidate_batch_scoring_snapshots()
            .into_iter()
            .find(|snapshot| snapshot.surface == "ivf")
            .unwrap();
        assert_eq!(task87_ivf.lut32_flushes, 0);
        assert_eq!(task87_ivf.lut32_candidates, 0);

        super::reset_candidate_batch_scoring_counters();
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
