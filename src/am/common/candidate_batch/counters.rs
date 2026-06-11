use super::super::quant_codec::QuantCodecKind;
use crate::quant::isa::Isa;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

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
        QuantCodecKind::TurboQuantTiledLut => 2,
        QuantCodecKind::TurboQuantInt8 => 3,
        QuantCodecKind::RaBitQ => 4,
        QuantCodecKind::GroupedPq => 5,
        QuantCodecKind::Binary => 6,
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
    pub(crate) width_lt8_flushes: u64,
    pub(crate) width_8_15_flushes: u64,
    pub(crate) width_16_31_flushes: u64,
    pub(crate) width_ge32_flushes: u64,
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
    /// Per-flush batch-width histogram (Task 98 acceptance criterion 4):
    /// buckets are 1-7, 8-15, 16-31, >=32 candidates per recorded flush.
    width_buckets: [AtomicU64; 4],
}

fn width_bucket_index(candidate_count: u64) -> usize {
    match candidate_count {
        0..=7 => 0,
        8..=15 => 1,
        16..=31 => 2,
        _ => 3,
    }
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
            width_buckets: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
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
        for bucket in &self.width_buckets {
            bucket.store(0, Ordering::Relaxed);
        }
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

    fn record_width(&self, batch_width: u64) {
        self.width_buckets[width_bucket_index(batch_width)].fetch_add(1, Ordering::Relaxed);
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
            width_lt8_flushes: self.width_buckets[0].load(Ordering::Relaxed),
            width_8_15_flushes: self.width_buckets[1].load(Ordering::Relaxed),
            width_16_31_flushes: self.width_buckets[2].load(Ordering::Relaxed),
            width_ge32_flushes: self.width_buckets[3].load(Ordering::Relaxed),
        }
    }
}

const SURFACE_COUNT: usize = 5;
const QUANT_COUNT: usize = 7;
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

pub(crate) fn record_block_kernel_score(
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

/// Records one batch-width histogram sample for a wrapper-level flush.
pub(crate) fn record_flush_width(
    surface: CandidateBatchScoringSurface,
    quant_kind: QuantCodecKind,
    isa: Isa,
    batch_width: usize,
) {
    if batch_width == 0 {
        return;
    }
    let key = BlockKernelCounterKey {
        surface,
        quant_kind,
        isa,
    };
    block_kernel_counters(key)
        .record_width(u64::try_from(batch_width).expect("batch width should fit in u64"));
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
