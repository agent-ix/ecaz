use std::mem::{offset_of, size_of};
use std::ptr::{self, NonNull};

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};

use crate::am::common::callback::pg_am_callback;

use super::{
    EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
    EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS, EC_IVF_DEFAULT_NLISTS,
    EC_IVF_DEFAULT_NPROBE, EC_IVF_DEFAULT_POSTING_SLACK_PERCENT, EC_IVF_DEFAULT_PQ_GROUP_SIZE,
    EC_IVF_DEFAULT_QUANT_BITS, EC_IVF_DEFAULT_RERANK_GROUP_WIDTH, EC_IVF_DEFAULT_RERANK_WIDTH,
    EC_IVF_DEFAULT_SEED, EC_IVF_DEFAULT_STAGE2_FINAL_RERANK_WIDTH,
    EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS, EC_IVF_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
    EC_IVF_MAX_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS, EC_IVF_MAX_NLISTS, EC_IVF_MAX_NPROBE,
    EC_IVF_MAX_POSTING_SLACK_PERCENT, EC_IVF_MAX_PQ_GROUP_SIZE, EC_IVF_MAX_QUANT_BITS,
    EC_IVF_MAX_RERANK_GROUP_WIDTH, EC_IVF_MAX_RERANK_WIDTH, EC_IVF_MAX_SEED,
    EC_IVF_MAX_STAGE2_FINAL_RERANK_WIDTH, EC_IVF_MAX_TRAINING_SAMPLE_ROWS, EC_IVF_MIN_NLISTS,
    EC_IVF_MIN_NPROBE, EC_IVF_MIN_POSTING_SLACK_PERCENT, EC_IVF_MIN_PQ_GROUP_SIZE,
    EC_IVF_MIN_QUANT_BITS, EC_IVF_MIN_RERANK_GROUP_WIDTH, EC_IVF_MIN_RERANK_WIDTH, EC_IVF_MIN_SEED,
    EC_IVF_MIN_STAGE2_FINAL_RERANK_WIDTH, EC_IVF_MIN_TRAINING_SAMPLE_ROWS,
};

const EC_IVF_SESSION_NPROBE_UNSET: i32 = -1;
const EC_IVF_SESSION_RERANK_WIDTH_UNSET: i32 = -1;
pub(super) const EC_IVF_DEFAULT_RABITQ_RERANK_CLIP: i32 = 2;
const EC_IVF_MIN_RABITQ_RERANK_CLIP: i32 = 1;
const EC_IVF_MAX_RABITQ_RERANK_CLIP: i32 = 8;

static EC_IVF_NPROBE_GUC: GucSetting<i32> = GucSetting::<i32>::new(EC_IVF_SESSION_NPROBE_UNSET);
static EC_IVF_RERANK_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_IVF_SESSION_RERANK_WIDTH_UNSET);
static EC_IVF_STAGE2_FINAL_RERANK_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_IVF_SESSION_RERANK_WIDTH_UNSET);
static EC_IVF_ADAPTIVE_NPROBE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static EC_IVF_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS);
static EC_IVF_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS);
// ADR-077 §4 (Task 105, 2026-06-12): default flipped to on. The Task 99
// three-lane profile measured batch-on winning IVF turboquant by
// -66/-69% (local) and -44% (G4) p50 with byte-equal recall, and winning
// pq_fastscan despite the suffix-max pruning trade (-5 to -10% on every
// lane). Off remains a diagnostic switch.
static EC_IVF_SCRATCH_SOA_BATCH_DECODE_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
static EC_IVF_DENSE_POSTING_COALESCING_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
static EC_IVF_DENSE_POSTING_TYPED_VIEWS_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
// Task 112: drive the heap-f32 exact rerank through the lazy frontier driver
// (process the approximate frontier best-first and stop exact-scoring once the
// remaining candidates are provably unable to enter the result). Enabled by
// default: under the sound `NoBound` contract that holds until Task 113 lands a
// calibrated lower bound, the lazy driver never stops early, so it is
// byte-identical to the fixed-width path. Disable to force the legacy
// fixed-width rerank for a deterministic A/B.
static EC_IVF_LAZY_HEAP_RERANK_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
// Task 113: thread the running top-k cutoff (`min_ip_to_keep`) into posting
// scoring so candidates whose sound Cauchy-Schwarz upper bound proves they
// cannot enter the frontier are pruned before full scoring/retention. Enabled
// by default and recall-safe by construction (the cutoff is a deterministic
// upper bound, see `quant::rabitq::try_estimate_ip_scalar`). Disable to force
// the unpruned scan for a deterministic A/B; the pruned and unpruned scans must
// return byte-identical results (only the work counts differ).
static EC_IVF_POSTING_BOUND_PRUNE_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
// Task 136: session selector for the TurboQuant no-QJL 4-bit approximate
// scorer. `lut` is the i16-LUT block kernel (Task 125); `int8_approx`
// routes the same prepared query through the factored rank-1 in-register
// kernel (`quant::int8_approx32`, Task 98 + Task 141 SDOT) that keeps the
// 16-entry codebook in one register and streams an i8-quantized rotated
// query. Query-side only — on-disk codes are decoded identically. Default
// flipped to `int8_approx` per the Task 143 promotion matrix (100k/1m:
// −33/−30% latency vs lut at recall within noise across nprobe 8–64).
static EC_IVF_TURBOQUANT_SCORER_GUC: GucSetting<TurboQuantScorerGuc> =
    GucSetting::<TurboQuantScorerGuc>::new(TurboQuantScorerGuc::Int8Approx);

/// Session selector for the ec_ivf TurboQuant no-QJL 4-bit approximate-scan
/// scorer (Task 136). Mirrors the `ec_hnsw.turboquant_exact_score_mode`
/// enum-GUC pattern so the A/B is on/off in one binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PostgresGucEnum)]
pub(super) enum TurboQuantScorerGuc {
    #[name = c"lut"]
    Lut,
    #[name = c"int8_approx"]
    Int8Approx,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcIvfReloptions {
    vl_len_: i32,
    nlists: i32,
    nprobe: i32,
    rerank_width: i32,
    rerank_group_width: i32,
    stage2_final_rerank_width: i32,
    training_sample_rows: i32,
    seed: i32,
    pq_group_size: i32,
    posting_slack_percent: i32,
    quant_bits: i32,
    coarse_bits: i32,
    dense_posting_blocks: i32,
    dense_posting_typed_layout: i32,
    rabitq_residual: i32,
    rabitq_rerank_least_squares: i32,
    rerank_exact_dequant: i32,
    rabitq_rerank_clip: i32,
    storage_format_offset: i32,
    quantizer_offset: i32,
    rerank_offset: i32,
    coarse_format_offset: i32,
    rerank_placement_offset: i32,
    rerank_format_offset: i32,
    turboquant_profile_offset: i32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageFormat {
    Auto = 0,
    TurboQuant = 1,
    PqFastScan = 2,
    RaBitQ = 3,
    CoarseRerank = 4,
}

impl StorageFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "turboquant" => Ok(Self::TurboQuant),
            "pq_fastscan" => Ok(Self::PqFastScan),
            "rabitq" => Ok(Self::RaBitQ),
            "coarse_rerank" => Ok(Self::CoarseRerank),
            other => Err(format!(
                "invalid ec_ivf storage_format reloption: expected 'auto', 'turboquant', 'pq_fastscan', 'rabitq', or 'coarse_rerank', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::TurboQuant => "turboquant",
            Self::PqFastScan => "pq_fastscan",
            Self::RaBitQ => "rabitq",
            Self::CoarseRerank => "coarse_rerank",
        }
    }

    pub(super) fn validate_v1_supported(self) -> Result<(), String> {
        match self {
            Self::Auto
            | Self::TurboQuant
            | Self::PqFastScan
            | Self::RaBitQ
            | Self::CoarseRerank => Ok(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurboQuantProfile {
    Standard = 0,
    TqPlus = 1,
}

impl TurboQuantProfile {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "standard" => Ok(Self::Standard),
            "tqplus" => Ok(Self::TqPlus),
            other => Err(format!(
                "invalid ec_ivf turboquant_profile reloption: expected 'standard' or 'tqplus', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::TqPlus => "tqplus",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerankMode {
    Auto = 0,
    Off = 1,
    HeapF32 = 2,
    SourceColumn = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoarseFormat {
    Auto = 0,
    RaBitQ = 1,
}

impl CoarseFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "rabitq" => Ok(Self::RaBitQ),
            other => Err(format!(
                "invalid ec_ivf coarse_format reloption: expected 'auto' or 'rabitq', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::RaBitQ => "rabitq",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerankPlacement {
    Auto = 0,
    Source = 1,
    Table = 2,
    Index = 3,
    SourceDiagnostic = 4,
}

impl RerankPlacement {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "source" | "heap" => Ok(Self::Source),
            "table" => Ok(Self::Table),
            "index" => Ok(Self::Index),
            "source_diagnostic" | "heap_diagnostic" => Ok(Self::SourceDiagnostic),
            other => Err(format!(
                "invalid ec_ivf rerank_placement reloption: expected 'auto', 'source', 'heap', 'table', 'index', or 'source_diagnostic', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Source => "source",
            Self::Table => "table",
            Self::Index => "index",
            Self::SourceDiagnostic => "source_diagnostic",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerankFormat {
    Auto = 0,
    F32 = 1,
    RaBitQ2 = 2,
    RaBitQ4 = 3,
    RaBitQ8 = 4,
    TurboQuant = 5,
    F16 = 6,
}

impl RerankFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "f32" | "heap_f32" => Ok(Self::F32),
            "f16" | "heap_f16" => Ok(Self::F16),
            "rabitq2" | "rabitq_2" => Ok(Self::RaBitQ2),
            "rabitq4" | "rabitq_4" => Ok(Self::RaBitQ4),
            "rabitq8" | "rabitq_8" => Ok(Self::RaBitQ8),
            "turboquant" => Ok(Self::TurboQuant),
            other => Err(format!(
                "invalid ec_ivf rerank_format reloption: expected 'auto', 'f32', 'f16', 'heap_f32', 'rabitq2', 'rabitq4', 'rabitq8', or 'turboquant', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::F32 => "f32",
            Self::F16 => "f16",
            Self::RaBitQ2 => "rabitq2",
            Self::RaBitQ4 => "rabitq4",
            Self::RaBitQ8 => "rabitq8",
            Self::TurboQuant => "turboquant",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaBitQRerankScoreMode {
    Estimator,
    LeastSquares,
    ExactDequant,
}

impl RaBitQRerankScoreMode {
    pub(super) fn from_reloption_flags(
        least_squares: i32,
        exact_dequant: i32,
    ) -> Result<Self, String> {
        if !matches!(least_squares, 0 | 1) {
            return Err(format!(
                "ec_ivf rabitq_rerank_least_squares must be 0 or 1, got {least_squares}"
            ));
        }
        if !matches!(exact_dequant, 0 | 1) {
            return Err(format!(
                "ec_ivf rerank_exact_dequant must be 0 or 1, got {exact_dequant}"
            ));
        }
        match (least_squares, exact_dequant) {
            (0, 0) => Ok(Self::Estimator),
            (1, 0) => Ok(Self::LeastSquares),
            (0, 1) => Ok(Self::ExactDequant),
            (1, 1) => Err(
                "ec_ivf rabitq_rerank_least_squares and rerank_exact_dequant are mutually exclusive"
                    .to_owned(),
            ),
            _ => unreachable!("validated boolean reloption flags"),
        }
    }

    pub(super) fn from_metadata_byte(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Estimator),
            1 => Ok(Self::LeastSquares),
            2 => Ok(Self::ExactDequant),
            other => Err(format!(
                "invalid ec_ivf rerank score mode stored in metadata: {other}"
            )),
        }
    }

    pub(super) fn metadata_byte(self) -> u8 {
        match self {
            Self::Estimator => 0,
            Self::LeastSquares => 1,
            Self::ExactDequant => 2,
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Estimator => "estimator",
            Self::LeastSquares => "least_squares",
            Self::ExactDequant => "exact_dequant",
        }
    }
}

impl RerankMode {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "off" => Ok(Self::Off),
            "heap_f32" => Ok(Self::HeapF32),
            "source_column" => Ok(Self::SourceColumn),
            other => Err(format!(
                "invalid ec_ivf rerank reloption: expected 'auto', 'off', 'heap_f32', or 'source_column', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Off => "off",
            Self::HeapF32 => "heap_f32",
            Self::SourceColumn => "source_column",
        }
    }

    pub(super) fn v1_effective(self) -> Self {
        match self {
            Self::Auto => Self::Off,
            other => other,
        }
    }

    pub(super) fn validate_v1_supported(self) -> Result<(), String> {
        match self {
            Self::Auto | Self::Off | Self::HeapF32 => Ok(()),
            Self::SourceColumn => Err(format!(
                "ec_ivf rerank mode {} is not supported yet; use rerank = 'off', rerank = 'auto', or rerank = 'heap_f32'",
                self.reloption_name()
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct EcIvfOptions {
    pub(super) nlists: i32,
    pub(super) nprobe: i32,
    pub(super) rerank_width: i32,
    /// Task 124: build-time compact rerank sidecar group width. A value of 0
    /// preserves the historical behavior where groups flush at `rerank_width`.
    pub(super) rerank_group_width: i32,
    pub(super) stage2_final_rerank_width: i32,
    pub(super) training_sample_rows: i32,
    pub(super) seed: i32,
    pub(super) pq_group_size: i32,
    pub(super) posting_slack_percent: i32,
    pub(super) quant_bits: i32,
    pub(super) coarse_bits: i32,
    pub(super) dense_posting_blocks: bool,
    pub(super) dense_posting_typed_layout: bool,
    /// Task 115: RaBitQ residual encoding gate. Only meaningful for
    /// `storage_format = 'rabitq'`; ignored (forced false) otherwise.
    pub(super) rabitq_residual: bool,
    /// Task 111h follow-up: index/source-diagnostic RaBitQ rerank scoring
    /// profile. Default keeps the paper estimator; least_squares exposes the
    /// lower-variance dequantized projection already present in the harness.
    pub(super) rabitq_rerank_score: RaBitQRerankScoreMode,
    /// Task 111h follow-up: integer scalar clip radius used for persisted
    /// RaBitQ rerank payloads. Default 2 preserves the existing profile.
    pub(super) rabitq_rerank_clip: i32,
    pub(super) storage_format: StorageFormat,
    pub(super) turboquant_profile: TurboQuantProfile,
    pub(super) rerank: RerankMode,
    pub(super) coarse_format: CoarseFormat,
    pub(super) rerank_placement: RerankPlacement,
    pub(super) rerank_format: RerankFormat,
}

impl EcIvfOptions {
    const DEFAULT: Self = Self {
        nlists: EC_IVF_DEFAULT_NLISTS,
        nprobe: EC_IVF_DEFAULT_NPROBE,
        rerank_width: EC_IVF_DEFAULT_RERANK_WIDTH,
        rerank_group_width: EC_IVF_DEFAULT_RERANK_GROUP_WIDTH,
        stage2_final_rerank_width: EC_IVF_DEFAULT_STAGE2_FINAL_RERANK_WIDTH,
        training_sample_rows: EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS,
        seed: EC_IVF_DEFAULT_SEED,
        pq_group_size: EC_IVF_DEFAULT_PQ_GROUP_SIZE,
        posting_slack_percent: EC_IVF_DEFAULT_POSTING_SLACK_PERCENT,
        quant_bits: EC_IVF_DEFAULT_QUANT_BITS,
        coarse_bits: 0,
        // Task 143 promotion: the no-reloptions default (storage_format Auto
        // resolves to TurboQuant) builds dense posting blocks.
        dense_posting_blocks: true,
        dense_posting_typed_layout: false,
        rabitq_residual: false,
        rabitq_rerank_score: RaBitQRerankScoreMode::Estimator,
        rabitq_rerank_clip: EC_IVF_DEFAULT_RABITQ_RERANK_CLIP,
        storage_format: StorageFormat::Auto,
        turboquant_profile: TurboQuantProfile::Standard,
        rerank: RerankMode::Auto,
        coarse_format: CoarseFormat::Auto,
        rerank_placement: RerankPlacement::Auto,
        rerank_format: RerankFormat::Auto,
    };

    /// Validated, in-range quantizer bits-per-dim used for RaBitQ
    /// encoding/decoding. Always returns a value in {1, 2, 4, 8}; if
    /// the index stores 0 (legacy metadata page with no bits field),
    /// falls back to `EC_IVF_DEFAULT_QUANT_BITS = 4`.
    pub(super) fn effective_quant_bits(self) -> u8 {
        let raw = if self.quant_bits == 0 {
            EC_IVF_DEFAULT_QUANT_BITS
        } else {
            self.quant_bits
        };
        match raw {
            1 | 2 | 4 | 8 => raw as u8,
            _ => EC_IVF_DEFAULT_QUANT_BITS as u8,
        }
    }

    pub(super) fn requested_pq_group_size(self) -> Option<usize> {
        if self.pq_group_size > 0 {
            Some(self.pq_group_size as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NprobeResolution {
    pub(super) relation_nprobe: u32,
    pub(super) session_nprobe: Option<u32>,
    pub(super) effective_nprobe: u32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RerankWidthResolution {
    pub(super) relation_rerank_width: i32,
    pub(super) session_rerank_width: Option<i32>,
    pub(super) effective_rerank_width: i32,
    pub(super) source: &'static str,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_ivf.nprobe",
        c"Session override for ec_ivf posting-list probe count.",
        c"Overrides ec_ivf index nprobe reloption when set to 1 or higher; -1 uses the relation value.",
        &EC_IVF_NPROBE_GUC,
        EC_IVF_SESSION_NPROBE_UNSET,
        EC_IVF_MAX_NPROBE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_ivf.rerank_width",
        c"Session override for ec_ivf heap_f32 rerank frontier width.",
        c"Overrides ec_ivf index rerank_width reloption when set to 0 or higher; -1 uses the relation value.",
        &EC_IVF_RERANK_WIDTH_GUC,
        EC_IVF_SESSION_RERANK_WIDTH_UNSET,
        EC_IVF_MAX_RERANK_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_ivf.stage2_final_rerank_width",
        c"Session override for ec_ivf TQ stage-2 final exact f32 rerank width.",
        c"Overrides ec_ivf index stage2_final_rerank_width reloption when set to 0 or higher; 0 disables the final exact stage and -1 uses the relation value.",
        &EC_IVF_STAGE2_FINAL_RERANK_WIDTH_GUC,
        EC_IVF_SESSION_RERANK_WIDTH_UNSET,
        EC_IVF_MAX_STAGE2_FINAL_RERANK_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.adaptive_nprobe",
        c"Enable deterministic adaptive ec_ivf nprobe reduction.",
        c"Diagnostic Task 51 mode; when enabled, scans may reduce nprobe by half when the centroid frontier has the configured score gap.",
        &EC_IVF_ADAPTIVE_NPROBE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_ivf.adaptive_nprobe_score_gap_micros",
        c"Score-gap threshold for ec_ivf adaptive nprobe.",
        c"Inner-product score gap, multiplied by 1,000,000, required between the retained adaptive frontier and the next centroid before adaptive nprobe reduces probe breadth.",
        &EC_IVF_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC,
        0,
        EC_IVF_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_ivf.adaptive_nprobe_score_margin_ratio_bps",
        c"Score margin-ratio threshold for ec_ivf adaptive nprobe.",
        c"Basis-point ratio of the boundary score gap to the top-to-boundary score margin. Values greater than zero switch adaptive nprobe to the ratio signal.",
        &EC_IVF_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS_GUC,
        0,
        EC_IVF_MAX_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.scratch_soa_batch_decode",
        c"Enable ec_ivf posting scratch SoA batch decode (block-kernel scoring).",
        c"Batches decoded posting tuple fields into scan-local structure-of-arrays buffers so scoring routes through the block kernels. Enabled by default per ADR-077 §4 (Task 99 three-lane profile); disable only as a diagnostic A/B switch.",
        &EC_IVF_SCRATCH_SOA_BATCH_DECODE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.dense_posting_coalescing",
        c"Enable ec_ivf dense posting cross-block coalescing.",
        c"Diagnostic Task 111a switch; when enabled, dense posting blocks are coalesced across consecutive blocks before batch scoring. Disable only to compare against the original one-page dense scan behavior.",
        &EC_IVF_DENSE_POSTING_COALESCING_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.dense_posting_typed_views",
        c"Enable ec_ivf dense posting aligned typed views.",
        c"Diagnostic Task 111a switch; when enabled, little-endian aligned dense numeric arrays can be read as native typed slices. Disable to compare against the byte-decoding fallback.",
        &EC_IVF_DENSE_POSTING_TYPED_VIEWS_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.lazy_heap_rerank",
        c"Enable ec_ivf lazy heap-f32 exact rerank.",
        c"Task 112 switch; when enabled, the heap-f32 rerank stage processes the approximate frontier best-first and stops exact-scoring once the remaining candidates are provably unable to enter the result. Under the sound no-bound default (until Task 113 supplies a calibrated lower bound) the stop never fires early, so this is byte-identical to the fixed-width path. Disable to force the legacy fixed-width rerank for a deterministic A/B.",
        &EC_IVF_LAZY_HEAP_RERANK_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_ivf.posting_bound_prune",
        c"Enable ec_ivf posting-scan bound pruning.",
        c"Task 113 switch; when enabled, the running top-k cutoff is threaded into posting scoring so candidates whose sound Cauchy-Schwarz upper bound proves they cannot enter the frontier are pruned before full scoring/retention. Recall-safe by construction. Disable to force the unpruned scan for a deterministic A/B; pruned and unpruned scans return byte-identical results, only the work counts differ.",
        &EC_IVF_POSTING_BOUND_PRUNE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_enum_guc(
        c"ec_ivf.turboquant_scorer",
        c"Session selector for the ec_ivf TurboQuant no-QJL 4-bit approximate scorer.",
        c"Task 136/141/143 scorer selector. Values: int8_approx (default per the Task 143 promotion: factored rank-1 in-register SDOT kernel with an i8-quantized rotated query), lut (i16-LUT block kernel, the pre-143 default). Query-side only; on-disk codes are unchanged.",
        &EC_IVF_TURBOQUANT_SCORER_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_nprobe() -> i32 {
    EC_IVF_NPROBE_GUC.get()
}

pub(super) fn current_session_rerank_width() -> i32 {
    EC_IVF_RERANK_WIDTH_GUC.get()
}

pub(super) fn current_session_stage2_final_rerank_width() -> i32 {
    EC_IVF_STAGE2_FINAL_RERANK_WIDTH_GUC.get()
}

pub(super) fn current_session_adaptive_nprobe() -> bool {
    if cfg!(test) {
        false
    } else {
        EC_IVF_ADAPTIVE_NPROBE_GUC.get()
    }
}

pub(super) fn current_session_adaptive_nprobe_score_gap_micros() -> i32 {
    if cfg!(test) {
        EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS
    } else {
        EC_IVF_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC.get()
    }
}

pub(super) fn current_session_adaptive_nprobe_score_margin_ratio_bps() -> i32 {
    if cfg!(test) {
        EC_IVF_DEFAULT_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS
    } else {
        EC_IVF_ADAPTIVE_NPROBE_SCORE_MARGIN_RATIO_BPS_GUC.get()
    }
}

pub(super) fn current_session_scratch_soa_batch_decode() -> bool {
    if cfg!(test) {
        false
    } else {
        EC_IVF_SCRATCH_SOA_BATCH_DECODE_GUC.get()
    }
}

pub(super) fn current_session_dense_posting_coalescing() -> bool {
    if cfg!(test) {
        true
    } else {
        EC_IVF_DENSE_POSTING_COALESCING_GUC.get()
    }
}

pub(super) fn current_session_lazy_heap_rerank() -> bool {
    EC_IVF_LAZY_HEAP_RERANK_GUC.get()
}

pub(super) fn current_session_posting_bound_prune() -> bool {
    if cfg!(test) {
        true
    } else {
        EC_IVF_POSTING_BOUND_PRUNE_GUC.get()
    }
}

pub(super) fn current_session_dense_posting_typed_views() -> bool {
    if cfg!(test) {
        true
    } else {
        EC_IVF_DENSE_POSTING_TYPED_VIEWS_GUC.get()
    }
}

pub(super) fn current_session_turboquant_scorer() -> TurboQuantScorerGuc {
    if cfg!(test) {
        // Unit tests pin the LUT scorer so the legacy kernel keeps
        // deterministic coverage (the int8 path is covered by the explicit
        // `prepare_ip_query_with_turboquant_scorer` tests and the
        // int8_approx32 parity suite). The production default is
        // Int8Approx per the Task 143 promotion.
        TurboQuantScorerGuc::Lut
    } else {
        EC_IVF_TURBOQUANT_SCORER_GUC.get()
    }
}

pub(super) fn resolve_scan_nprobe(nlists: u32, relation_nprobe: u32) -> NprobeResolution {
    let session_nprobe = match current_session_nprobe() {
        value if value > 0 => Some(value as u32),
        _ => None,
    };
    if nlists == 0 {
        return NprobeResolution {
            relation_nprobe,
            session_nprobe,
            effective_nprobe: 0,
            source: "none",
        };
    }

    let (requested, source) = match session_nprobe {
        Some(value) => (value, "session"),
        None if relation_nprobe == 0 => (auto_nprobe(nlists), "auto"),
        None => (relation_nprobe, "relation"),
    };

    NprobeResolution {
        relation_nprobe,
        session_nprobe,
        effective_nprobe: requested.clamp(1, nlists),
        source,
    }
}

pub(super) fn resolve_scan_rerank_width(relation_rerank_width: i32) -> RerankWidthResolution {
    let session_rerank_width = match current_session_rerank_width() {
        value if value >= 0 => Some(value),
        _ => None,
    };
    let (effective_rerank_width, source) = match session_rerank_width {
        Some(value) => (value.clamp(0, EC_IVF_MAX_RERANK_WIDTH), "session"),
        None => (relation_rerank_width, "relation"),
    };

    RerankWidthResolution {
        relation_rerank_width,
        session_rerank_width,
        effective_rerank_width,
        source,
    }
}

pub(super) fn resolve_scan_stage2_final_rerank_width(
    relation_stage2_final_rerank_width: i32,
) -> RerankWidthResolution {
    let session_rerank_width = match current_session_stage2_final_rerank_width() {
        value if value >= 0 => Some(value),
        _ => None,
    };
    let (effective_rerank_width, source) = match session_rerank_width {
        Some(value) => (
            value.clamp(
                EC_IVF_MIN_STAGE2_FINAL_RERANK_WIDTH,
                EC_IVF_MAX_STAGE2_FINAL_RERANK_WIDTH,
            ),
            "session",
        ),
        None => (
            relation_stage2_final_rerank_width.clamp(
                EC_IVF_MIN_STAGE2_FINAL_RERANK_WIDTH,
                EC_IVF_MAX_STAGE2_FINAL_RERANK_WIDTH,
            ),
            "relation",
        ),
    };

    RerankWidthResolution {
        relation_rerank_width: relation_stage2_final_rerank_width,
        session_rerank_width,
        effective_rerank_width,
        source,
    }
}

fn auto_nprobe(nlists: u32) -> u32 {
    if nlists == 0 {
        return 0;
    }
    (nlists as f64).sqrt().ceil() as u32
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    pg_am_callback!({
        let mut relopts = pg_sys::local_relopts::default();

        pg_sys::init_local_reloptions(&mut relopts, size_of::<EcIvfReloptions>());
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"nlists".as_ptr(),
            c"Number of IVF centroid posting lists; 0 chooses an automatic value.".as_ptr(),
            EC_IVF_DEFAULT_NLISTS,
            EC_IVF_MIN_NLISTS,
            EC_IVF_MAX_NLISTS,
            offset_of!(EcIvfReloptions, nlists) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"nprobe".as_ptr(),
            c"Number of IVF posting lists to probe during scan; 0 chooses an automatic value."
                .as_ptr(),
            EC_IVF_DEFAULT_NPROBE,
            EC_IVF_MIN_NPROBE,
            EC_IVF_MAX_NPROBE,
            offset_of!(EcIvfReloptions, nprobe) as i32,
        );
        pg_sys::add_local_int_reloption(
                &mut relopts,
                c"rerank_width".as_ptr(),
                c"Number of approximate candidates to return after heap-rerank when rerank = 'heap_f32'; 0 reranks and returns the full probed frontier."
                    .as_ptr(),
                EC_IVF_DEFAULT_RERANK_WIDTH,
                EC_IVF_MIN_RERANK_WIDTH,
                EC_IVF_MAX_RERANK_WIDTH,
                offset_of!(EcIvfReloptions, rerank_width) as i32,
            );
        pg_sys::add_local_int_reloption(
                &mut relopts,
                c"rerank_group_width".as_ptr(),
                c"Task 124: build-time compact rerank sidecar group width; 0 uses rerank_width. Smaller values improve index-side TQ payload locality without changing scan frontier width."
                    .as_ptr(),
                EC_IVF_DEFAULT_RERANK_GROUP_WIDTH,
                EC_IVF_MIN_RERANK_GROUP_WIDTH,
                EC_IVF_MAX_RERANK_GROUP_WIDTH,
                offset_of!(EcIvfReloptions, rerank_group_width) as i32,
            );
        pg_sys::add_local_int_reloption(
                &mut relopts,
                c"stage2_final_rerank_width".as_ptr(),
                c"Task 124: exact/source f32 rerank width after index-side TurboQuant stage-2; 0 disables the second exact stage."
                    .as_ptr(),
                EC_IVF_DEFAULT_STAGE2_FINAL_RERANK_WIDTH,
                EC_IVF_MIN_STAGE2_FINAL_RERANK_WIDTH,
                EC_IVF_MAX_STAGE2_FINAL_RERANK_WIDTH,
                offset_of!(EcIvfReloptions, stage2_final_rerank_width) as i32,
            );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"training_sample_rows".as_ptr(),
            c"Maximum rows sampled for centroid training; 0 chooses an automatic value.".as_ptr(),
            EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS,
            EC_IVF_MIN_TRAINING_SAMPLE_ROWS,
            EC_IVF_MAX_TRAINING_SAMPLE_ROWS,
            offset_of!(EcIvfReloptions, training_sample_rows) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"seed".as_ptr(),
            c"Deterministic seed for IVF centroid training.".as_ptr(),
            EC_IVF_DEFAULT_SEED,
            EC_IVF_MIN_SEED,
            EC_IVF_MAX_SEED,
            offset_of!(EcIvfReloptions, seed) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"pq_group_size".as_ptr(),
            c"Grouped-PQ subvector size for storage_format = 'pq_fastscan'; 0 chooses the default."
                .as_ptr(),
            EC_IVF_DEFAULT_PQ_GROUP_SIZE,
            EC_IVF_MIN_PQ_GROUP_SIZE,
            EC_IVF_MAX_PQ_GROUP_SIZE,
            offset_of!(EcIvfReloptions, pq_group_size) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"posting_slack_percent".as_ptr(),
            c"Build-time extra empty posting pages reserved per IVF list for churn reuse.".as_ptr(),
            EC_IVF_DEFAULT_POSTING_SLACK_PERCENT,
            EC_IVF_MIN_POSTING_SLACK_PERCENT,
            EC_IVF_MAX_POSTING_SLACK_PERCENT,
            offset_of!(EcIvfReloptions, posting_slack_percent) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"quant_bits".as_ptr(),
            c"RaBitQ per-dimension code width: 1, 2, 4 (default), or 8 bits. 1-bit gives ~4x kernel throughput vs 4-bit; pair with rerank='heap_f32' for high-recall queries."
                .as_ptr(),
            EC_IVF_DEFAULT_QUANT_BITS,
            EC_IVF_MIN_QUANT_BITS,
            EC_IVF_MAX_QUANT_BITS,
            offset_of!(EcIvfReloptions, quant_bits) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"coarse_bits".as_ptr(),
            c"Task 111e coarse_rerank coarse-stage RaBitQ bit width; 0 chooses the preset default, currently 1 bit."
                .as_ptr(),
            0,
            0,
            EC_IVF_MAX_QUANT_BITS,
            offset_of!(EcIvfReloptions, coarse_bits) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"dense_posting_blocks".as_ptr(),
            c"Task 111 build-time dense IVF posting block layout: -1 auto (dense for the TurboQuant lane per the Task 143 promotion; row for RaBitQ pending its own promotion), 0 disables, 1 enables for frozen build postings."
                .as_ptr(),
            -1,
            -1,
            1,
            offset_of!(EcIvfReloptions, dense_posting_blocks) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"dense_posting_typed_layout".as_ptr(),
            c"Experimental Task 111a aligned dense posting layout for native little-endian typed views: 0 disables, 1 enables."
                .as_ptr(),
            0,
            0,
            1,
            offset_of!(EcIvfReloptions, dense_posting_typed_layout) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"rabitq_residual".as_ptr(),
            c"Task 115 RaBitQ residual encoding: 0 disables (plain RaBitQ, default), 1 encodes posting payloads as the residual against the assigned IVF centroid. Only valid with storage_format = 'rabitq'."
                .as_ptr(),
            0,
            0,
            1,
            offset_of!(EcIvfReloptions, rabitq_residual) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"rabitq_rerank_least_squares".as_ptr(),
            c"Task 111h RaBitQ rerank scorer: 0 uses the default asymmetric estimator, 1 uses the lower-variance least-squares dequantized projection."
                .as_ptr(),
            0,
            0,
            1,
            offset_of!(EcIvfReloptions, rabitq_rerank_least_squares) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"rerank_exact_dequant".as_ptr(),
            c"Task 111h compact rerank scorer: 0 uses the format default, 1 scores the persisted compact payload as a dequantized vector diagnostic."
                .as_ptr(),
            0,
            0,
            1,
            offset_of!(EcIvfReloptions, rerank_exact_dequant) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"rabitq_rerank_clip".as_ptr(),
            c"Task 111h RaBitQ rerank scalar quantization clip radius for compact rerank payloads; default 2 preserves the existing profile."
                .as_ptr(),
            EC_IVF_DEFAULT_RABITQ_RERANK_CLIP,
            EC_IVF_MIN_RABITQ_RERANK_CLIP,
            EC_IVF_MAX_RABITQ_RERANK_CLIP,
            offset_of!(EcIvfReloptions, rabitq_rerank_clip) as i32,
        );
        pg_sys::add_local_string_reloption(
                &mut relopts,
                c"storage_format".as_ptr(),
                c"IVF posting-list quantizer profile: 'turboquant', 'pq_fastscan', 'rabitq', 'coarse_rerank', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcIvfReloptions, storage_format_offset) as i32,
            );
        pg_sys::add_local_string_reloption(
                &mut relopts,
                c"quantizer".as_ptr(),
                c"Alias for storage_format: IVF posting-list quantizer profile 'turboquant', 'pq_fastscan', 'rabitq', 'coarse_rerank', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcIvfReloptions, quantizer_offset) as i32,
            );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            c"rerank".as_ptr(),
            c"IVF rerank mode: 'off', 'heap_f32', 'source_column', or 'auto'.".as_ptr(),
            ptr::null(),
            None,
            None,
            offset_of!(EcIvfReloptions, rerank_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            c"coarse_format".as_ptr(),
            c"Task 111e coarse_rerank coarse-stage format: 'rabitq' or 'auto'.".as_ptr(),
            ptr::null(),
            None,
            None,
            offset_of!(EcIvfReloptions, coarse_format_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            c"rerank_placement".as_ptr(),
            c"Task 111h coarse_rerank rerank payload placement: 'source', 'index', 'table' (reserved), 'source_diagnostic', or 'auto'."
                .as_ptr(),
            ptr::null(),
            None,
            None,
            offset_of!(EcIvfReloptions, rerank_placement_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            c"rerank_format".as_ptr(),
            c"Task 111e coarse_rerank rerank representation: 'f32', 'f16', 'rabitq4', 'rabitq8', 'turboquant', or 'auto'."
                .as_ptr(),
            ptr::null(),
            None,
            None,
            offset_of!(EcIvfReloptions, rerank_format_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            c"turboquant_profile".as_ptr(),
            c"Task 148 TurboQuant calibration profile: 'standard' or 'tqplus'. Default 'standard' preserves existing codes."
                .as_ptr(),
            ptr::null(),
            None,
            None,
            offset_of!(EcIvfReloptions, turboquant_profile_offset) as i32,
        );
        pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
    })
}

struct EcIvfReloptionsView {
    rd_options: crate::am::common::reloptions::ReloptionsBlob,
}

impl EcIvfReloptionsView {
    fn from_relation(index_relation: NonNull<pg_sys::RelationData>) -> Option<Self> {
        let rd_options = crate::storage::relation::relation_options_handle(index_relation);
        let rd_options = NonNull::new(rd_options)?;
        Some(Self {
            rd_options: crate::am::common::reloptions::ReloptionsBlob::new(rd_options),
        })
    }

    fn reloptions(&self) -> &EcIvfReloptions {
        crate::storage::relation::relation_options_layout_ref(self.rd_options.handle())
    }

    fn read_string_reloption(&self, offset: i32, name: &str) -> Option<String> {
        self.rd_options
            .read_string_reloption(offset, "ec_ivf", name)
    }

    fn to_options(&self) -> EcIvfOptions {
        let reloptions = self.reloptions();
        let storage_format_reloption =
            self.read_string_reloption(reloptions.storage_format_offset, "storage_format");
        let quantizer_reloption =
            self.read_string_reloption(reloptions.quantizer_offset, "quantizer");
        let rerank_reloption = self.read_string_reloption(reloptions.rerank_offset, "rerank");
        let coarse_format_reloption =
            self.read_string_reloption(reloptions.coarse_format_offset, "coarse_format");
        let rerank_placement_reloption =
            self.read_string_reloption(reloptions.rerank_placement_offset, "rerank_placement");
        let rerank_format_reloption =
            self.read_string_reloption(reloptions.rerank_format_offset, "rerank_format");
        let turboquant_profile_reloption =
            self.read_string_reloption(reloptions.turboquant_profile_offset, "turboquant_profile");

        build_options_from_reloptions(
            reloptions,
            storage_format_reloption,
            quantizer_reloption,
            rerank_reloption,
            coarse_format_reloption,
            rerank_placement_reloption,
            rerank_format_reloption,
            turboquant_profile_reloption,
        )
    }
}

fn build_options_from_reloptions(
    reloptions: &EcIvfReloptions,
    storage_format_reloption: Option<String>,
    quantizer_reloption: Option<String>,
    rerank_reloption: Option<String>,
    coarse_format_reloption: Option<String>,
    rerank_placement_reloption: Option<String>,
    rerank_format_reloption: Option<String>,
    turboquant_profile_reloption: Option<String>,
) -> EcIvfOptions {
    if let (Some(storage_format), Some(quantizer)) =
        (&storage_format_reloption, &quantizer_reloption)
    {
        if storage_format != quantizer {
            pgrx::error!(
                "ec_ivf storage_format and quantizer reloptions conflict: storage_format = '{}', quantizer = '{}'",
                storage_format,
                quantizer
            );
        }
    }
    let storage_format = storage_format_reloption
        .or(quantizer_reloption)
        .map(|value| StorageFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}")))
        .unwrap_or(StorageFormat::Auto);
    let mut rerank = match rerank_reloption {
        Some(value) => RerankMode::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}")),
        None => RerankMode::Auto,
    };
    let mut coarse_format = match coarse_format_reloption {
        Some(value) => {
            CoarseFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        }
        None => CoarseFormat::Auto,
    };
    let mut coarse_bits = reloptions.coarse_bits;
    let mut rerank_placement = match rerank_placement_reloption {
        Some(value) => {
            RerankPlacement::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        }
        None => RerankPlacement::Auto,
    };
    let mut rerank_format = match rerank_format_reloption {
        Some(value) => {
            RerankFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        }
        None => RerankFormat::Auto,
    };
    let turboquant_profile = turboquant_profile_reloption
        .map(|value| {
            TurboQuantProfile::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        })
        .unwrap_or(TurboQuantProfile::Standard);
    let rabitq_rerank_score = RaBitQRerankScoreMode::from_reloption_flags(
        reloptions.rabitq_rerank_least_squares,
        reloptions.rerank_exact_dequant,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    if !(EC_IVF_MIN_RABITQ_RERANK_CLIP..=EC_IVF_MAX_RABITQ_RERANK_CLIP)
        .contains(&reloptions.rabitq_rerank_clip)
    {
        pgrx::error!(
            "ec_ivf rabitq_rerank_clip must be between {} and {}, got {}",
            EC_IVF_MIN_RABITQ_RERANK_CLIP,
            EC_IVF_MAX_RABITQ_RERANK_CLIP,
            reloptions.rabitq_rerank_clip
        );
    }
    if storage_format == StorageFormat::CoarseRerank {
        match coarse_format {
            CoarseFormat::Auto => coarse_format = CoarseFormat::RaBitQ,
            CoarseFormat::RaBitQ => {}
        }
        match coarse_bits {
            0 => coarse_bits = 1,
            1 => {}
            2 | 4 | 8 => pgrx::error!(
                "ec_ivf storage_format = 'coarse_rerank' currently requires coarse_bits = 1; wider coarse stages belong in the Task 111e alternative-coarse-stage sweep"
            ),
            _ => pgrx::error!(
                "ec_ivf coarse_bits must be 0, 1, 2, 4, or 8 for storage_format = 'coarse_rerank'"
            ),
        }
        match rerank_format {
            RerankFormat::Auto => rerank_format = RerankFormat::F32,
            // Task 111h: f32 source plus persisted f16/RaBitQ-4/RaBitQ-8/
            // TurboQuant compact reranks are implemented through the common
            // rerank payload codec. RaBitQ-2 remains outside the required
            // 111h decision matrix.
            RerankFormat::F32
            | RerankFormat::F16
            | RerankFormat::RaBitQ4
            | RerankFormat::RaBitQ8
            | RerankFormat::TurboQuant => {}
            RerankFormat::RaBitQ2 => {
                pgrx::error!(
                    "ec_ivf storage_format = 'coarse_rerank' supports rerank_format = 'f32', 'f16', 'rabitq4', 'rabitq8', or 'turboquant'; '{}' is not implemented",
                    rerank_format.reloption_name()
                )
            }
        }
        match rerank_placement {
            RerankPlacement::Auto => {
                rerank_placement = match rerank_format {
                    RerankFormat::F32 => RerankPlacement::Source,
                    RerankFormat::F16
                    | RerankFormat::RaBitQ4
                    | RerankFormat::RaBitQ8
                    | RerankFormat::TurboQuant => RerankPlacement::Index,
                    RerankFormat::Auto
                    | RerankFormat::RaBitQ2 => unreachable!(
                        "coarse_rerank rerank_format should be resolved or rejected"
                    ),
                }
            }
            RerankPlacement::Source => match rerank_format {
                RerankFormat::F32 => {}
                RerankFormat::F16
                | RerankFormat::RaBitQ4
                | RerankFormat::RaBitQ8
                | RerankFormat::TurboQuant => pgrx::error!(
                    "ec_ivf rerank_placement = 'source' reads the existing f32 source vector and only supports rerank_format = 'f32'; use rerank_placement = 'index' for persisted compact payloads or 'source_diagnostic' for the legacy query-time conversion benchmark"
                ),
                RerankFormat::Auto | RerankFormat::RaBitQ2 => unreachable!(
                    "coarse_rerank rerank_format should be resolved or rejected"
                ),
            },
            RerankPlacement::Table => pgrx::error!(
                "ec_ivf rerank_placement = 'table' is reserved for real table-owned persisted rerank payloads and is not implemented yet; use rerank_placement = 'source' for the existing f32 source vector or 'index' for persisted compact payloads"
            ),
            // Task 111h: index placement persists compact payloads in packed
            // scorer-width rerank groups. It is only meaningful with compact
            // rerank_format values; f32 keeps the source vector and has no
            // compact payload to place index-side.
            RerankPlacement::Index => match rerank_format {
                RerankFormat::F16
                | RerankFormat::RaBitQ4
                | RerankFormat::RaBitQ8
                | RerankFormat::TurboQuant => {}
                RerankFormat::F32 => pgrx::error!(
                    "ec_ivf rerank_placement = 'index' requires a compact rerank_format ('f16', 'rabitq4', 'rabitq8', or 'turboquant'); 'f32' keeps the source vector, so use rerank_placement = 'source'"
                ),
                RerankFormat::Auto | RerankFormat::RaBitQ2 => pgrx::error!(
                    "ec_ivf rerank_placement = 'index' supports rerank_format = 'f16', 'rabitq4', 'rabitq8', or 'turboquant'; '{}' is not implemented",
                    rerank_format.reloption_name()
                ),
            },
            RerankPlacement::SourceDiagnostic => match rerank_format {
                RerankFormat::F32
                | RerankFormat::F16
                | RerankFormat::RaBitQ4
                | RerankFormat::RaBitQ8
                | RerankFormat::TurboQuant => {}
                RerankFormat::Auto | RerankFormat::RaBitQ2 => unreachable!(
                    "coarse_rerank rerank_format should be resolved or rejected"
                ),
            },
        }
        match rerank {
            RerankMode::Auto => rerank = RerankMode::HeapF32,
            RerankMode::HeapF32 => {}
            RerankMode::Off | RerankMode::SourceColumn => pgrx::error!(
                "ec_ivf storage_format = 'coarse_rerank' requires rerank = 'auto' or rerank = 'heap_f32'"
            ),
        }
    }

    let rabitq_residual = reloptions.rabitq_residual != 0;
    if rabitq_residual && storage_format != StorageFormat::RaBitQ {
        pgrx::error!(
            "ec_ivf rabitq_residual = 1 requires storage_format = 'rabitq' (got '{}')",
            storage_format.reloption_name()
        );
    }
    let rabitq_only_rerank_knob_set = rabitq_rerank_score == RaBitQRerankScoreMode::LeastSquares
        || reloptions.rabitq_rerank_clip != EC_IVF_DEFAULT_RABITQ_RERANK_CLIP;
    if rabitq_only_rerank_knob_set
        && !(storage_format == StorageFormat::CoarseRerank
            && matches!(rerank_format, RerankFormat::RaBitQ4 | RerankFormat::RaBitQ8))
    {
        pgrx::error!(
            "ec_ivf RaBitQ rerank scoring knobs require storage_format = 'coarse_rerank' with rerank_format = 'rabitq4' or 'rabitq8'"
        );
    }
    if rabitq_rerank_score == RaBitQRerankScoreMode::ExactDequant
        && !(storage_format == StorageFormat::CoarseRerank
            && matches!(
                rerank_format,
                RerankFormat::RaBitQ4 | RerankFormat::RaBitQ8 | RerankFormat::TurboQuant
            ))
    {
        pgrx::error!(
            "ec_ivf rerank_exact_dequant requires storage_format = 'coarse_rerank' with rerank_format = 'rabitq4', 'rabitq8', or 'turboquant'"
        );
    }
    if reloptions.stage2_final_rerank_width > 0
        && !(storage_format == StorageFormat::CoarseRerank
            && rerank_placement == RerankPlacement::Index
            && rerank_format == RerankFormat::TurboQuant)
    {
        pgrx::error!(
            "ec_ivf stage2_final_rerank_width requires storage_format = 'coarse_rerank', rerank_placement = 'index', and rerank_format = 'turboquant'"
        );
    }
    if reloptions.rerank_group_width > 0
        && !(storage_format == StorageFormat::CoarseRerank
            && rerank_placement == RerankPlacement::Index
            && matches!(
                rerank_format,
                RerankFormat::F16
                    | RerankFormat::RaBitQ4
                    | RerankFormat::RaBitQ8
                    | RerankFormat::TurboQuant
            ))
    {
        pgrx::error!(
            "ec_ivf rerank_group_width requires storage_format = 'coarse_rerank', rerank_placement = 'index', and a compact rerank_format"
        );
    }
    if turboquant_profile == TurboQuantProfile::TqPlus
        && !matches!(
            storage_format,
            StorageFormat::Auto | StorageFormat::TurboQuant
        )
    {
        pgrx::error!(
            "ec_ivf turboquant_profile = 'tqplus' currently requires storage_format = 'turboquant' or auto; coarse_rerank turboquant sidecar support is not wired yet"
        );
    }

    EcIvfOptions {
        nlists: reloptions.nlists,
        nprobe: reloptions.nprobe,
        rerank_width: reloptions.rerank_width,
        rerank_group_width: reloptions.rerank_group_width,
        stage2_final_rerank_width: reloptions.stage2_final_rerank_width,
        training_sample_rows: reloptions.training_sample_rows,
        seed: reloptions.seed,
        pq_group_size: reloptions.pq_group_size,
        posting_slack_percent: reloptions.posting_slack_percent,
        quant_bits: if storage_format == StorageFormat::CoarseRerank {
            1
        } else {
            reloptions.quant_bits
        },
        coarse_bits,
        dense_posting_blocks: storage_format == StorageFormat::CoarseRerank
            || match reloptions.dense_posting_blocks {
                // Task 143 promotion: auto (-1) resolves to dense for the
                // TurboQuant lane (recall byte-identical to row at every
                // measured nprobe/scale cell, storage −10%, latency win at
                // 100k/1m). RaBitQ auto stays row pending its own promotion
                // decision (Task 111a closeout kept it gated).
                -1 => matches!(
                    storage_format,
                    StorageFormat::Auto | StorageFormat::TurboQuant
                ),
                value => value != 0,
            },
        dense_posting_typed_layout: storage_format == StorageFormat::CoarseRerank
            || reloptions.dense_posting_typed_layout != 0,
        rabitq_residual,
        rabitq_rerank_score,
        rabitq_rerank_clip: reloptions.rabitq_rerank_clip,
        storage_format,
        turboquant_profile,
        rerank,
        coarse_format,
        rerank_placement,
        rerank_format,
    }
}

pub(super) fn relation_options(index_relation: NonNull<pg_sys::RelationData>) -> EcIvfOptions {
    let Some(reloptions) = EcIvfReloptionsView::from_relation(index_relation) else {
        return EcIvfOptions::DEFAULT;
    };
    reloptions.to_options()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reloptions() -> EcIvfReloptions {
        EcIvfReloptions {
            vl_len_: 0,
            nlists: 64,
            nprobe: 32,
            rerank_width: 50,
            rerank_group_width: 0,
            stage2_final_rerank_width: 0,
            training_sample_rows: 10_000,
            seed: 42,
            pq_group_size: 0,
            posting_slack_percent: 0,
            quant_bits: 4,
            coarse_bits: 0,
            dense_posting_blocks: 0,
            dense_posting_typed_layout: 0,
            rabitq_residual: 0,
            rabitq_rerank_least_squares: 0,
            rerank_exact_dequant: 0,
            rabitq_rerank_clip: EC_IVF_DEFAULT_RABITQ_RERANK_CLIP,
            storage_format_offset: 0,
            quantizer_offset: 0,
            rerank_offset: 0,
            coarse_format_offset: 0,
            rerank_placement_offset: 0,
            rerank_format_offset: 0,
        }
    }

    #[test]
    fn storage_format_parse_accepts_coarse_rerank() {
        let parsed = StorageFormat::parse_reloption("coarse_rerank").unwrap();

        assert_eq!(parsed, StorageFormat::CoarseRerank);
        assert_eq!(parsed.reloption_name(), "coarse_rerank");
    }

    #[test]
    fn coarse_rerank_preset_resolves_dense_rabitq1_heap_f32() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.coarse_format, CoarseFormat::RaBitQ);
        assert_eq!(options.coarse_bits, 1);
        assert_eq!(options.quant_bits, 1);
        assert!(options.dense_posting_blocks);
        assert!(options.dense_posting_typed_layout);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Source);
        assert_eq!(options.rerank_format, RerankFormat::F32);
    }

    #[test]
    fn coarse_rerank_keeps_explicit_source_f32() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            Some("heap_f32".into()),
            Some("rabitq".into()),
            Some("source".into()),
            Some("f32".into()),
        );

        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.coarse_format, CoarseFormat::RaBitQ);
        assert_eq!(options.rerank_placement, RerankPlacement::Source);
        assert_eq!(options.rerank_format, RerankFormat::F32);
    }

    #[test]
    fn coarse_rerank_accepts_explicit_phase2_contract() {
        let mut reloptions = reloptions();
        reloptions.coarse_bits = 1;

        let options = build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            Some("heap_f32".into()),
            Some("rabitq".into()),
            Some("heap".into()),
            Some("heap_f32".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.coarse_format, CoarseFormat::RaBitQ);
        assert_eq!(options.coarse_bits, 1);
        assert_eq!(options.quant_bits, 1);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Source);
        assert_eq!(options.rerank_format, RerankFormat::F32);
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_table_placement_until_real_table_payloads_exist() {
        build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            Some("heap_f32".into()),
            Some("rabitq".into()),
            Some("table".into()),
            Some("f32".into()),
        );
    }

    #[test]
    fn rerank_format_parse_accepts_f16() {
        assert_eq!(
            RerankFormat::parse_reloption("f16").unwrap(),
            RerankFormat::F16
        );
        assert_eq!(
            RerankFormat::parse_reloption("heap_f16").unwrap(),
            RerankFormat::F16
        );
        assert_eq!(RerankFormat::F16.reloption_name(), "f16");
    }

    #[test]
    fn coarse_rerank_accepts_f16_rerank_format() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("f16".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::F16);
    }

    #[test]
    fn coarse_rerank_accepts_rabitq4_rerank_format() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("rabitq4".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::RaBitQ4);
    }

    #[test]
    fn coarse_rerank_accepts_rabitq8_rerank_format() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("rabitq8".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::RaBitQ8);
    }

    #[test]
    fn coarse_rerank_accepts_rabitq_rerank_scoring_knobs() {
        let mut reloptions = reloptions();
        reloptions.rabitq_rerank_least_squares = 1;
        reloptions.rabitq_rerank_clip = 4;

        let options = build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("rabitq8".into()),
        );

        assert_eq!(options.rerank_format, RerankFormat::RaBitQ8);
        assert_eq!(
            options.rabitq_rerank_score,
            RaBitQRerankScoreMode::LeastSquares
        );
        assert_eq!(options.rabitq_rerank_clip, 4);
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_rabitq_rerank_knobs_for_non_rabitq_format() {
        let mut reloptions = reloptions();
        reloptions.rabitq_rerank_least_squares = 1;

        build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("f16".into()),
        );
    }

    #[test]
    fn coarse_rerank_accepts_exact_dequant_for_compact_quantized_formats() {
        for format in ["rabitq4", "rabitq8", "turboquant"] {
            let mut reloptions = reloptions();
            reloptions.rerank_exact_dequant = 1;

            let options = build_options_from_reloptions(
                &reloptions,
                Some("coarse_rerank".into()),
                None,
                None,
                None,
                None,
                Some(format.into()),
            );

            assert_eq!(
                options.rabitq_rerank_score,
                RaBitQRerankScoreMode::ExactDequant
            );
        }
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_exact_dequant_for_f16() {
        let mut reloptions = reloptions();
        reloptions.rerank_exact_dequant = 1;

        build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("f16".into()),
        );
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_conflicting_score_mode_flags() {
        let mut reloptions = reloptions();
        reloptions.rabitq_rerank_least_squares = 1;
        reloptions.rerank_exact_dequant = 1;

        build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("rabitq8".into()),
        );
    }

    #[test]
    fn coarse_rerank_accepts_turboquant_rerank_format() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("turboquant".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::TurboQuant);
    }

    #[test]
    fn coarse_rerank_accepts_turboquant_stage2_final_width() {
        let mut reloptions = reloptions();
        reloptions.rerank_width = 100;
        reloptions.rerank_group_width = 16;
        reloptions.stage2_final_rerank_width = 25;

        let options = build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("turboquant".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_width, 100);
        assert_eq!(options.rerank_group_width, 16);
        assert_eq!(options.stage2_final_rerank_width, 25);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::TurboQuant);
    }

    #[test]
    #[should_panic]
    fn rerank_group_width_rejects_non_index_compact_rerank() {
        let mut reloptions = reloptions();
        reloptions.rerank_group_width = 16;

        build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("source".into()),
            Some("f32".into()),
        );
    }

    #[test]
    #[should_panic]
    fn stage2_final_width_rejects_non_turboquant_rerank_format() {
        let mut reloptions = reloptions();
        reloptions.stage2_final_rerank_width = 25;

        build_options_from_reloptions(
            &reloptions,
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("rabitq8".into()),
        );
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_source_placement_with_compact_format() {
        build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("source".into()),
            Some("f16".into()),
        );
    }

    #[test]
    fn coarse_rerank_accepts_source_diagnostic_with_compact_format() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("source_diagnostic".into()),
            Some("f16".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::SourceDiagnostic);
        assert_eq!(options.rerank_format, RerankFormat::F16);
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_rabitq2_rerank_format() {
        build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            None,
            Some("rabitq2".into()),
        );
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_index_placement_with_default_f32() {
        // Task 111g (003b): index placement requires a compact rerank_format;
        // the default (auto -> f32) keeps the heap source and is rejected.
        build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            None,
        );
    }

    #[test]
    #[should_panic]
    fn coarse_rerank_rejects_index_placement_with_explicit_f32() {
        build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("f32".into()),
        );
    }

    #[test]
    fn coarse_rerank_accepts_index_placement_with_f16() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("f16".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::F16);
    }

    #[test]
    fn coarse_rerank_accepts_index_placement_with_rabitq4() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("rabitq4".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::RaBitQ4);
    }

    #[test]
    fn coarse_rerank_accepts_index_placement_with_rabitq8() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("rabitq8".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::RaBitQ8);
    }

    #[test]
    fn coarse_rerank_accepts_index_placement_with_turboquant() {
        let options = build_options_from_reloptions(
            &reloptions(),
            Some("coarse_rerank".into()),
            None,
            None,
            None,
            Some("index".into()),
            Some("turboquant".into()),
        );

        assert_eq!(options.storage_format, StorageFormat::CoarseRerank);
        assert_eq!(options.rerank, RerankMode::HeapF32);
        assert_eq!(options.rerank_placement, RerankPlacement::Index);
        assert_eq!(options.rerank_format, RerankFormat::TurboQuant);
    }
}
