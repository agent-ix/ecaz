use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use crate::am::common::callback::pg_am_callback;

use super::{
    page::{
        DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
        DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
    },
    ECDISTANN_DEFAULT_ALPHA, ECDISTANN_DEFAULT_BEAM_WIDTH, ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
    ECDISTANN_DEFAULT_CLOSURE_EPSILON, ECDISTANN_DEFAULT_GRAPH_DEGREE,
    ECDISTANN_DEFAULT_HEAD_INDEX_CAP, ECDISTANN_DEFAULT_HOP_ROUNDS, ECDISTANN_MAX_ALPHA,
    ECDISTANN_MAX_BEAM_WIDTH, ECDISTANN_MAX_BUILD_LIST_SIZE, ECDISTANN_MAX_CLOSURE_EPSILON,
    ECDISTANN_MAX_GRAPH_DEGREE, ECDISTANN_MAX_HEAD_INDEX_CAP, ECDISTANN_MAX_HOP_ROUNDS,
    ECDISTANN_DEFAULT_TOP_K, ECDISTANN_MAX_TOP_K, ECDISTANN_MIN_ALPHA,
    ECDISTANN_MIN_BUILD_LIST_SIZE, ECDISTANN_MIN_CLOSURE_EPSILON, ECDISTANN_MIN_GRAPH_DEGREE,
    ECDISTANN_MIN_HEAD_INDEX_CAP,
};

/// FR-081 beam width BW: frontier candidates expanded per hop round.
static ECDISTANN_BEAM_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_BEAM_WIDTH);

/// FR-081 hop-round budget H: BW x H is the hard per-query expansion cap
/// (NFR-019).
static ECDISTANN_HOP_ROUNDS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_HOP_ROUNDS);

/// Result-heap size k used by the FR-081 convergence early-exit.
static ECDISTANN_TOP_K_GUC: GucSetting<i32> = GucSetting::<i32>::new(ECDISTANN_DEFAULT_TOP_K);

static ECDISTANN_SCAN_PROFILE_NOTICE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcDistannReloptions {
    vl_len_: i32,
    graph_degree: i32,
    build_list_size: i32,
    head_index_cap: i32,
    // Postgres real reloptions are stored as C doubles; downcast to f32 when
    // constructing `EcDistannOptions` (same posture as ec_diskann alpha).
    alpha: f64,
    closure_epsilon: f64,
    neighbor_code_format_offset: i32,
    source_identity_offset: i32,
}

/// Neighbor-code codec selected by the `neighbor_code_format` reloption
/// (ADR-085 D7: GroupedPq default; rabitq and turboquant are the measured
/// M0 alternatives).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NeighborCodeFormat {
    GroupedPq,
    RaBitQ,
    TurboQuant,
}

impl NeighborCodeFormat {
    pub(super) const DEFAULT: Self = Self::GroupedPq;

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::GroupedPq => "grouped_pq",
            Self::RaBitQ => "rabitq",
            Self::TurboQuant => "turboquant",
        }
    }

    pub(super) fn metadata_kind(self) -> u8 {
        match self {
            Self::GroupedPq => DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            Self::RaBitQ => DISTANN_NEIGHBOR_CODEC_RABITQ,
            Self::TurboQuant => DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
        }
    }

    fn parse_reloption(raw: &str) -> Result<Self, String> {
        match raw {
            "grouped_pq" => Ok(Self::GroupedPq),
            "rabitq" => Ok(Self::RaBitQ),
            "turboquant" => Ok(Self::TurboQuant),
            other => Err(format!(
                "invalid ec_distann neighbor_code_format reloption: expected 'grouped_pq', 'rabitq', or 'turboquant', got {:?}",
                other
            )),
        }
    }
}

/// The shared `source_identity` reloption (ADR-063 identity contract via
/// ADR-068 topology); same surface as ec_spire's provider selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DistannSourceIdentityProvider {
    None,
    Include,
}

impl DistannSourceIdentityProvider {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "include" => Ok(Self::Include),
            other => Err(format!(
                "invalid ec_distann source_identity reloption: expected 'include', got '{other}'"
            )),
        }
    }

    #[allow(dead_code)]
    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Include => "include",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EcDistannOptions {
    pub(super) graph_degree: i32,
    pub(super) build_list_size: i32,
    pub(super) head_index_cap: i32,
    pub(super) alpha: f32,
    pub(super) closure_epsilon: f32,
    pub(super) neighbor_code_format: NeighborCodeFormat,
    pub(super) source_identity: DistannSourceIdentityProvider,
}

impl EcDistannOptions {
    pub(super) const DEFAULT: Self = Self {
        graph_degree: ECDISTANN_DEFAULT_GRAPH_DEGREE,
        build_list_size: ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
        head_index_cap: ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
        alpha: ECDISTANN_DEFAULT_ALPHA,
        closure_epsilon: ECDISTANN_DEFAULT_CLOSURE_EPSILON,
        neighbor_code_format: NeighborCodeFormat::DEFAULT,
        source_identity: DistannSourceIdentityProvider::None,
    };
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_distann.beam_width",
        c"FR-081 hop-round beam width (BW) for ec_distann scans.",
        c"Each hop round expands up to this many best unvisited frontier candidates. BW x ec_distann.hop_rounds is the hard per-query expansion cap (NFR-019). Default matches the ec_diskann batched-beam width measured in Task 168.",
        &ECDISTANN_BEAM_WIDTH_GUC,
        1,
        ECDISTANN_MAX_BEAM_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.hop_rounds",
        c"FR-081 hop-round budget (H) for ec_distann scans.",
        c"Scans terminate after this many hop rounds, or earlier on convergence (ADR-085 D9 fixed-H with early-exit). The default is provisional until the M0 recall-vs-H kill-check measurement pins it.",
        &ECDISTANN_HOP_ROUNDS_GUC,
        1,
        ECDISTANN_MAX_HOP_ROUNDS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.top_k",
        c"Result-heap size k for ec_distann scans.",
        c"Bounds the FR-081 convergence early-exit: the scan may stop once k exact results cannot be improved by the beam's best unvisited code distance. Results themselves are all expanded records, so a query LIMIT above k still gets rows; set k >= the query LIMIT for correct early-exit behavior.",
        &ECDISTANN_TOP_K_GUC,
        1,
        ECDISTANN_MAX_TOP_K,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.scan_profile_notice",
        c"Emit per-query ec_distann FR-081 traversal counters as NOTICE.",
        c"Observability for NFR-019/FR-081-AC-5: rounds executed, records expanded (<= BW x H), neighbors code-scored, early-exit and beam-exhaustion flags. The M4 bench pipeline step consumes the same counters.",
        &ECDISTANN_SCAN_PROFILE_NOTICE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_top_k() -> usize {
    usize::try_from(ECDISTANN_TOP_K_GUC.get()).unwrap_or(1).max(1)
}

pub(super) fn scan_profile_notice_enabled() -> bool {
    ECDISTANN_SCAN_PROFILE_NOTICE_GUC.get()
}

pub(super) fn current_beam_width() -> usize {
    usize::try_from(ECDISTANN_BEAM_WIDTH_GUC.get())
        .unwrap_or(1)
        .max(1)
}

pub(super) fn current_hop_rounds() -> usize {
    usize::try_from(ECDISTANN_HOP_ROUNDS_GUC.get())
        .unwrap_or(1)
        .max(1)
}

pub(super) unsafe extern "C-unwind" fn ec_distann_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    pg_am_callback!({
        let mut relopts = pg_sys::local_relopts::default();

        pg_sys::init_local_reloptions(&mut relopts, size_of::<EcDistannReloptions>());
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"graph_degree\0".as_ptr().cast(),
            b"Fixed neighbor count per global-graph node (R); also caps the FR-076 neighbor-code block.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_GRAPH_DEGREE,
            ECDISTANN_MIN_GRAPH_DEGREE,
            ECDISTANN_MAX_GRAPH_DEGREE,
            offset_of!(EcDistannReloptions, graph_degree) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"build_list_size\0".as_ptr().cast(),
            b"Candidate list width used during Vamana graph construction (L).\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
            ECDISTANN_MIN_BUILD_LIST_SIZE,
            ECDISTANN_MAX_BUILD_LIST_SIZE,
            offset_of!(EcDistannReloptions, build_list_size) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"head_index_cap\0".as_ptr().cast(),
            b"FR-080 coordinator head-index sample cap (C); ADR-085 D3 default, recall sensitivity measured at M0.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
            ECDISTANN_MIN_HEAD_INDEX_CAP,
            ECDISTANN_MAX_HEAD_INDEX_CAP,
            offset_of!(EcDistannReloptions, head_index_cap) as i32,
        );
        pg_sys::add_local_real_reloption(
            &mut relopts,
            b"alpha\0".as_ptr().cast(),
            b"Vamana alpha-pruning slack.\0".as_ptr().cast(),
            ECDISTANN_DEFAULT_ALPHA as f64,
            ECDISTANN_MIN_ALPHA as f64,
            ECDISTANN_MAX_ALPHA as f64,
            offset_of!(EcDistannReloptions, alpha) as i32,
        );
        pg_sys::add_local_real_reloption(
            &mut relopts,
            b"closure_epsilon\0".as_ptr().cast(),
            b"FR-077 build-shard closure-overlap band; unused by the monolithic M0 build.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_CLOSURE_EPSILON as f64,
            ECDISTANN_MIN_CLOSURE_EPSILON as f64,
            ECDISTANN_MAX_CLOSURE_EPSILON as f64,
            offset_of!(EcDistannReloptions, closure_epsilon) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            b"neighbor_code_format\0".as_ptr().cast(),
            b"FR-076 neighbor-code codec: 'grouped_pq' (default), 'rabitq', or 'turboquant'.\0"
                .as_ptr()
                .cast(),
            ptr::null(),
            None,
            None,
            offset_of!(EcDistannReloptions, neighbor_code_format_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            b"source_identity\0".as_ptr().cast(),
            b"ADR-063 source-identity provider; set to 'include' to derive vec_id from row identity.\0"
                .as_ptr()
                .cast(),
            ptr::null(),
            None,
            None,
            offset_of!(EcDistannReloptions, source_identity_offset) as i32,
        );
        pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
    })
}

struct EcDistannReloptionsView {
    rd_options: crate::am::common::reloptions::ReloptionsBlob,
}

impl EcDistannReloptionsView {
    unsafe fn from_relation(index_relation: pg_sys::Relation) -> Option<Self> {
        let index_relation = std::ptr::NonNull::new(index_relation).unwrap_or_else(|| {
            pgrx::error!("ec_distann relation options need a valid index relation")
        });
        let rd_options = crate::storage::relation::relation_options_handle(index_relation);
        let rd_options = std::ptr::NonNull::new(rd_options)?;
        Some(Self {
            rd_options: crate::am::common::reloptions::ReloptionsBlob::new(rd_options),
        })
    }

    fn reloptions(&self) -> &EcDistannReloptions {
        crate::storage::relation::relation_options_layout_ref(self.rd_options.handle())
    }

    fn read_string_reloption(&self, offset: i32, name: &str) -> Option<String> {
        self.rd_options
            .read_string_reloption(offset, "ec_distann", name)
    }

    fn to_options(&self) -> EcDistannOptions {
        let reloptions = self.reloptions();
        let neighbor_code_format = match self.read_string_reloption(
            reloptions.neighbor_code_format_offset,
            "neighbor_code_format",
        ) {
            Some(value) => NeighborCodeFormat::parse_reloption(&value)
                .unwrap_or_else(|e| pgrx::error!("{e}")),
            None => NeighborCodeFormat::DEFAULT,
        };
        let source_identity = match self
            .read_string_reloption(reloptions.source_identity_offset, "source_identity")
        {
            Some(value) => DistannSourceIdentityProvider::parse_reloption(&value)
                .unwrap_or_else(|e| pgrx::error!("{e}")),
            None => DistannSourceIdentityProvider::None,
        };

        EcDistannOptions {
            graph_degree: reloptions.graph_degree,
            build_list_size: reloptions.build_list_size,
            head_index_cap: reloptions.head_index_cap,
            alpha: reloptions.alpha as f32,
            closure_epsilon: reloptions.closure_epsilon as f32,
            neighbor_code_format,
            source_identity,
        }
    }
}

pub(super) fn relation_options(index_relation: pg_sys::Relation) -> EcDistannOptions {
    // SAFETY: callers provide a live PostgreSQL index relation. The view keeps
    // reloption pointer handling scoped to this relation-options layout.
    match unsafe { EcDistannReloptionsView::from_relation(index_relation) } {
        Some(view) => view.to_options(),
        None => EcDistannOptions::DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DistannSourceIdentityProvider, EcDistannOptions, NeighborCodeFormat,
        ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
    };

    #[test]
    fn distann_default_options_match_spec_defaults() {
        let defaults = EcDistannOptions::DEFAULT;
        assert_eq!(defaults.graph_degree, 32);
        assert_eq!(defaults.build_list_size, 100);
        assert_eq!(defaults.head_index_cap, ECDISTANN_DEFAULT_HEAD_INDEX_CAP);
        assert_eq!(defaults.head_index_cap, 4096);
        assert_eq!(defaults.neighbor_code_format, NeighborCodeFormat::GroupedPq);
        assert_eq!(
            defaults.source_identity,
            DistannSourceIdentityProvider::None
        );
    }

    #[test]
    fn distann_neighbor_code_format_parses_all_d7_codecs() {
        assert_eq!(
            NeighborCodeFormat::parse_reloption("grouped_pq").unwrap(),
            NeighborCodeFormat::GroupedPq
        );
        assert_eq!(
            NeighborCodeFormat::parse_reloption("rabitq").unwrap(),
            NeighborCodeFormat::RaBitQ
        );
        assert_eq!(
            NeighborCodeFormat::parse_reloption("turboquant").unwrap(),
            NeighborCodeFormat::TurboQuant
        );
        assert!(NeighborCodeFormat::parse_reloption("opq").is_err());
    }

    #[test]
    fn distann_source_identity_parses_include_only() {
        assert_eq!(
            DistannSourceIdentityProvider::parse_reloption("include").unwrap(),
            DistannSourceIdentityProvider::Include
        );
        assert!(DistannSourceIdentityProvider::parse_reloption("none").is_err());
    }
}
