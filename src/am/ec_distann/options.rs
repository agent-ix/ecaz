use std::{
    ffi::CString,
    mem::{offset_of, size_of},
    ptr,
};

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use crate::am::common::callback::pg_am_callback;

use super::generation_descriptor::DistannHeadSizingAttestation;
use super::{
    page::{
        DISTANN_NEIGHBOR_CODEC_GROUPED_PQ, DISTANN_NEIGHBOR_CODEC_RABITQ,
        DISTANN_NEIGHBOR_CODEC_TURBOQUANT,
    },
    ECDISTANN_DEFAULT_ALPHA, ECDISTANN_DEFAULT_BEAM_WIDTH, ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
    ECDISTANN_DEFAULT_BUILD_SHARDS, ECDISTANN_DEFAULT_CANDIDATE_HEAP_LIMIT,
    ECDISTANN_DEFAULT_CLOSURE_EPSILON, ECDISTANN_DEFAULT_GRAPH_DEGREE,
    ECDISTANN_DEFAULT_HEAD_INDEX_CAP, ECDISTANN_DEFAULT_HOP_ROUNDS, ECDISTANN_DEFAULT_TOP_K,
    ECDISTANN_MAX_ALPHA, ECDISTANN_MAX_BEAM_WIDTH, ECDISTANN_MAX_BUILD_LIST_SIZE,
    ECDISTANN_MAX_BUILD_SHARDS, ECDISTANN_MAX_CANDIDATE_HEAP_LIMIT, ECDISTANN_MAX_CLOSURE_EPSILON,
    ECDISTANN_MAX_GRAPH_DEGREE, ECDISTANN_MAX_HEAD_INDEX_CAP, ECDISTANN_MAX_HOP_ROUNDS,
    ECDISTANN_MAX_TOP_K, ECDISTANN_MIN_ALPHA, ECDISTANN_MIN_BUILD_LIST_SIZE,
    ECDISTANN_MIN_BUILD_SHARDS, ECDISTANN_MIN_CLOSURE_EPSILON, ECDISTANN_MIN_GRAPH_DEGREE,
    ECDISTANN_MIN_HEAD_INDEX_CAP,
};

/// FR-081 beam width BW: frontier candidates expanded per hop round.
static ECDISTANN_BEAM_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_BEAM_WIDTH);

/// FR-081 candidate frontier bound L. This is independent of the expansion
/// budget so the pushdown threshold is derived from a real retained heap.
static ECDISTANN_CANDIDATE_HEAP_LIMIT_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_CANDIDATE_HEAP_LIMIT);

/// NFR-021 clause 4: opt-in for the FR-084 coordinator traversal replica, a
/// non-conforming accelerator that serves traversal from a coordinator-resident
/// copy of every owner's graph record and full-precision vector. Default off —
/// the sharded owner path is the default (Task 210 P1).
static ECDISTANN_ALLOW_NONCONFORMING_REPLICA_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

/// NFR-021 clause 3: search the FR-080 head as roster shards, so no node holds
/// the whole head (Task 210 P2a). A/B-able against the coordinator-local head.
static ECDISTANN_SHARDED_HEAD_SEARCH_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);

/// DISTRIBUTEDANN 4.1 head replica count (Task 210 P2b): additional roster
/// nodes that may serve a head shard, so head CPU is not bound to one machine
/// per shard. 0 means each shard is served by its owner.
static ECDISTANN_HEAD_REPLICA_COUNT_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);

/// TRAV-30 bounded gateway copies (Task 210 P3): how many gateway nodes'
/// routing payloads a coordinator may cache. A stated constant, never a
/// function of N. 0 disables gateway copies.
static ECDISTANN_GATEWAY_COPY_CAPACITY_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);

/// FR-089 bounded coordinator crown capacity in `(vec_id, search_code)` entries.
static ECDISTANN_CROWN_CAPACITY_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);
/// FR-089 explicit width-pruning arm. Off by default; only population-complete
/// crown shards may be omitted by the caller.
static ECDISTANN_CROWN_WIDTH_PRUNING_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
/// FR-090 explicit fused-head-hop arm. Off until the crown activation gate is
/// demonstrated by a Task 212 packet.
static ECDISTANN_FUSED_HEAD_HOP_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

/// NFR-021 clause 3 (Task 210 P2a): persist the FR-080 head as membership only,
/// so the coordinator holds landmark ids -- bounded by capacity C -- and never a
/// second copy of the landmark vectors, which already live on their owners.
static ECDISTANN_SHARD_HEAD_STORAGE_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);

/// FR-081 hop-round budget H: BW x H is the hard per-query expansion cap
/// (NFR-019).
static ECDISTANN_HOP_ROUNDS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_HOP_ROUNDS);

/// Result-heap size k used by the FR-081 convergence early-exit.
static ECDISTANN_TOP_K_GUC: GucSetting<i32> = GucSetting::<i32>::new(ECDISTANN_DEFAULT_TOP_K);

static ECDISTANN_SCAN_PROFILE_NOTICE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static ECDISTANN_PHYSICAL_EPOCH_CACHE_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_SEED_MODE_GUC: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_HEAD_SEARCH_WIDTH_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_HEAD_SEED_COUNT_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_EXACT_NEIGHBOR_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_HEAD_POLICY_GUC: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_TRAINING_QUERY_PATH_GUC: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_MATERIALIZATION_BATCH_SIZE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(-1);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_OWNER_PAYLOAD_PLAN_CACHE_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(false);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_TYPED_LOCATOR_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_PACKED_PAYLOAD_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_EXPANDED_LOCATOR_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_PAYLOAD_PROJECTION_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
#[cfg(feature = "distann-head-attribution-benchmark")]
static ECDISTANN_BENCHMARK_TRAVERSAL_REPLICA_FAIL_BATCH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(-1);
/// ADR-085 D12 production policy. This is deliberately not a GUC or reloption.
pub(super) const PRODUCTION_MATERIALIZATION_BATCH_SIZE: usize = 10;
const ECDISTANN_MAX_BENCHMARK_HEAD_WIDTH: i32 = 4096;
const ECDISTANN_DEFAULT_REMOTE_CONNECT_TIMEOUT_MS: i32 = 5_000;
const ECDISTANN_DEFAULT_REMOTE_STATEMENT_TIMEOUT_MS: i32 = 120_000;
const ECDISTANN_MAX_REMOTE_TIMEOUT_MS: i32 = 3_600_000;
static ECDISTANN_REMOTE_CONNECT_TIMEOUT_MS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_REMOTE_CONNECT_TIMEOUT_MS);
static ECDISTANN_REMOTE_STATEMENT_TIMEOUT_MS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISTANN_DEFAULT_REMOTE_STATEMENT_TIMEOUT_MS);
static ECDISTANN_REPLICA_CONTROL_PASSWORD_FILE_GUC: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);

/// NFR-020 fault-injection hook (debug only): when >= 0, an ec_distann scan
/// raises an error at the start of the hop round with this 0-based index, after
/// all earlier rounds have executed. Setting it to a value >= 1 injects a
/// `hop_round_failure_mid_beam` fault (a mid-beam remote/round failure that must
/// discard partial results and error, never return a partial result as
/// complete). -1 (the default) disables injection.
static ECDISTANN_DEBUG_FAIL_HOP_ROUND_GUC: GucSetting<i32> = GucSetting::<i32>::new(-1);

/// NFR-020 fault injection (debug only): when true, the local expander raises the
/// FR-079 case-(c) `OwnedRecordMissing` structural fault on the first vec_id it is
/// asked to expand, simulating a `missing_node_record` (an owned record absent
/// from the node's directory). The scan must error, never silently under-return.
static ECDISTANN_DEBUG_MISSING_NODE_RECORD_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "pg_test")]
static ECDISTANN_DEBUG_FORCE_FRONTIER_RETRY_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
// Packet 051 rejected the batch-consistent append-when-room candidate on the
// dominant 50k heldout quality gate, so robust-prune-all remains the shipped
// path. The toggle remains userset for attributed diagnostic measurement.
static ECDISTANN_DEBUG_DISABLE_APPEND_WHEN_ROOM_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(true);
static ECDISTANN_DEBUG_RETRY_ATTRIBUTION_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
#[cfg(feature = "pg_test")]
static ECDISTANN_DEBUG_FAIL_CROWN_POPULATION_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

/// NFR-020 fault injection (debug only): when true, `graph_insert_record` (the
/// FR-083 fold path) raises an error after staging the new node + directory pages
/// but before publishing the metadata, simulating a `mid-insert failure`. The
/// aborting transaction must roll the staged pages back so no partial record is
/// ever visible to a scan.
static ECDISTANN_DEBUG_FAIL_INSERT_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

/// Packet 005 transaction-boundary fault: fail after a complete handoff batch
/// has been decoded and all row datums prepared, but before the first physical
/// row/graph/directory insertion.
static ECDISTANN_DEBUG_FAIL_HANDOFF_AFTER_PREPARE_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(false);

/// Task 179 T4a crash-window fault: fail after every participant has returned
/// its Published acknowledgement but before the coordinator active-pointer
/// compare-and-swap. The recovery transaction must roll back participant-local
/// state and leave the already-committed Pending decision recoverable.
static ECDISTANN_DEBUG_FAIL_RECOVER_AFTER_PUBLISH_ACK_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(false);

/// Task 199 traversal-replica crash-window fault: terminate the mutating
/// backend after the dedicated control transaction has committed Ready ->
/// Stale, but before the caller can receive its ordinary retry error.
static ECDISTANN_DEBUG_CRASH_AFTER_REPLICA_CONTROL_COMMIT_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(false);

/// 0=off, 1=absent index TID, 2=callback vector mismatch, 3=callback identity
/// mismatch. Used to exercise otherwise race-impossible single-snapshot checks.
static ECDISTANN_DEBUG_SOURCE_CAPTURE_FAULT_GUC: GucSetting<i32> = GucSetting::<i32>::new(0);

/// NFR-020 fault injection (debug only): when true, `apply_record_writes` errors
/// after flipping the tombstone flags but before returning, simulating a lost
/// remote tombstone write (mid-delete failure). The aborting transaction must
/// roll the flag writes back so the record stays live — a lost tombstone write
/// errors and never silently resurrects (or half-deletes) the row.
static ECDISTANN_DEBUG_FAIL_TOMBSTONE_WRITE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcDistannReloptions {
    vl_len_: i32,
    graph_degree: i32,
    build_list_size: i32,
    head_index_cap: i32,
    head_sampling_rate: f64,
    head_cap_floor: i32,
    head_cap_ceiling: i32,
    build_shards: i32,
    distributed_control: bool,
    // Postgres real reloptions are stored as C doubles; downcast to f32 when
    // constructing `EcDistannOptions` (same posture as ec_diskann alpha).
    alpha: f64,
    closure_epsilon: f64,
    neighbor_code_format_offset: i32,
    source_identity_offset: i32,
    head_construction_offset: i32,
    covering_payload_attnums_offset: i32,
}

/// Task 207 construction A/B. The graph topology (`build_shards`) and the
/// head construction must be independently selectable so a measurement can
/// hold the sharded graph fixed while changing only the head pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadConstruction {
    StitchedBfs,
    PartitionUnion,
}

impl HeadConstruction {
    pub(crate) const DEFAULT: Self = Self::StitchedBfs;

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::StitchedBfs => "stitched_bfs",
            Self::PartitionUnion => "partition_union",
        }
    }

    fn parse_reloption(raw: &str) -> Result<Self, String> {
        match raw {
            "stitched_bfs" => Ok(Self::StitchedBfs),
            "partition_union" => Ok(Self::PartitionUnion),
            other => Err(format!(
                "invalid ec_distann head_construction reloption: expected 'stitched_bfs' or 'partition_union', got {other:?}"
            )),
        }
    }
}

/// Neighbor-code codec selected by the `neighbor_code_format` reloption.
/// ADR-085 D7 said "GroupedPq default, pinned at M0"; the M0 measurement
/// (task-162 packet 002) pinned it to RaBitQ: gpq tops out at 0.9245
/// recall@10 at 50k where rbq reaches 0.9950, and TQ does not fit a page
/// at the default R.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NeighborCodeFormat {
    GroupedPq,
    RaBitQ,
    TurboQuant,
}

impl NeighborCodeFormat {
    pub(super) const DEFAULT: Self = Self::RaBitQ;

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

    /// Recover the format from a persisted metadata codec kind (the inverse of
    /// `metadata_kind`); used by incremental insert to rebuild the codec.
    pub(crate) fn from_metadata_kind(kind: u8) -> Result<Self, String> {
        match kind {
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ => Ok(Self::GroupedPq),
            DISTANN_NEIGHBOR_CODEC_RABITQ => Ok(Self::RaBitQ),
            DISTANN_NEIGHBOR_CODEC_TURBOQUANT => Ok(Self::TurboQuant),
            other => Err(format!("unknown ec_distann neighbor codec kind {other}")),
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

    // Kept as the canonical inverse of parse_reloption for diagnostics and
    // future reloptions introspection; the current SQL path only parses.
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
    pub(super) head_sampling_rate: f64,
    pub(super) head_cap_floor: i32,
    pub(super) head_cap_ceiling: i32,
    /// FR-077 build-shard count (0 = auto, 1 = monolithic fallback, >=2
    /// sharded closure-overlap build + stitch).
    pub(super) build_shards: i32,
    pub(super) alpha: f32,
    pub(super) closure_epsilon: f32,
    pub(super) neighbor_code_format: NeighborCodeFormat,
    pub(super) source_identity: DistannSourceIdentityProvider,
    /// FR-075 logical coordinator index. False preserves the legacy v4 local
    /// graph build and metadata bytes; true creates metadata only.
    pub(super) distributed_control: bool,
    pub(super) head_construction: HeadConstruction,
    /// Task 229 opt-in physical row-tier cover. `None` preserves the legacy
    /// descriptor and relation set byte-for-byte.
    pub(super) covering_payload_attnums: Option<Vec<u16>>,
}

impl EcDistannOptions {
    pub(super) const DEFAULT: Self = Self {
        graph_degree: ECDISTANN_DEFAULT_GRAPH_DEGREE,
        build_list_size: ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
        head_index_cap: ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
        head_sampling_rate: 0.0,
        head_cap_floor: ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
        head_cap_ceiling: ECDISTANN_MAX_HEAD_INDEX_CAP,
        build_shards: ECDISTANN_DEFAULT_BUILD_SHARDS,
        alpha: ECDISTANN_DEFAULT_ALPHA,
        closure_epsilon: ECDISTANN_DEFAULT_CLOSURE_EPSILON,
        neighbor_code_format: NeighborCodeFormat::DEFAULT,
        source_identity: DistannSourceIdentityProvider::None,
        distributed_control: false,
        head_construction: HeadConstruction::DEFAULT,
        covering_payload_attnums: None,
    };

    pub(super) fn validate_head_sizing_inputs(&self) -> Result<(), String> {
        let floor = u32::try_from(self.head_cap_floor)
            .map_err(|_| "EC_HEAD_SIZING: head_cap_floor is outside the v1 domain".to_owned())?;
        let ceiling = u32::try_from(self.head_cap_ceiling)
            .map_err(|_| "EC_HEAD_SIZING: head_cap_ceiling is outside the v1 domain".to_owned())?;
        if !self.head_sampling_rate.is_finite() || self.head_sampling_rate < 0.0 {
            return Err(
                "EC_HEAD_SIZING: head_sampling_rate must be finite and non-negative".to_owned(),
            );
        }
        if floor < DistannHeadSizingAttestation::MIN_CAPACITY
            || floor > DistannHeadSizingAttestation::MAX_CAPACITY
            || ceiling < DistannHeadSizingAttestation::MIN_CAPACITY
            || ceiling > DistannHeadSizingAttestation::MAX_CAPACITY
            || floor > ceiling
        {
            return Err(
                "EC_HEAD_SIZING: head sampling law bounds are outside 16..=1048576".to_owned(),
            );
        }
        Ok(())
    }

    pub(super) fn resolve_head_sizing(
        &self,
        captured_record_count: u64,
        trained_policy: bool,
    ) -> Result<(u32, Option<DistannHeadSizingAttestation>), String> {
        self.validate_head_sizing_inputs()?;
        let floor = self.head_cap_floor as u32;
        let ceiling = self.head_cap_ceiling as u32;
        if self.head_sampling_rate == 0.0 {
            let explicit = u32::try_from(self.head_index_cap).map_err(|_| {
                "EC_HEAD_SIZING: explicit head_index_cap is out of range".to_owned()
            })?;
            if trained_policy && explicit != 4096 {
                return Err("EC_HEAD_TRAINING: trained head policy requires cap 4096".to_owned());
            }
            return Ok((explicit, None));
        }
        let resolved = DistannHeadSizingAttestation::resolve(
            self.head_sampling_rate,
            floor,
            ceiling,
            captured_record_count,
        )?;
        if trained_policy && resolved != 4096 {
            return Err(
                "EC_HEAD_TRAINING: trained head policy requires resolved cap 4096".to_owned(),
            );
        }
        Ok((
            resolved,
            Some(DistannHeadSizingAttestation {
                resolved_capacity: resolved,
                sample_count: 0,
                rate_bits: self.head_sampling_rate.to_bits(),
                floor,
                ceiling,
                captured_record_count,
                law_active: true,
            }),
        ))
    }
}

fn parse_covering_payload_attnums(raw: &str) -> Result<Vec<u16>, String> {
    let invalid = |reason: &str| {
        format!(
            "invalid ec_distann covering_payload_attnums reloption: {reason}; expected 1 to {} strictly increasing positive physical attnums in canonical comma-separated form",
            super::payload_sidecar::DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES
        )
    };
    if raw.is_empty() {
        return Err(invalid("value is empty"));
    }
    let tokens = raw.split(',').collect::<Vec<_>>();
    if tokens.len() > super::payload_sidecar::DISTANN_PAYLOAD_COVER_MAX_ATTRIBUTES {
        return Err(invalid("attribute count exceeds the format bound"));
    }
    let mut attnums = Vec::with_capacity(tokens.len());
    let mut previous = 0_u16;
    for token in tokens {
        if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid("value contains a non-decimal attribute"));
        }
        let attnum = token
            .parse::<u16>()
            .map_err(|_| invalid("attribute is outside the positive smallint domain"))?;
        if attnum == 0 {
            return Err(invalid("attribute zero is not physical"));
        }
        if token != attnum.to_string() {
            return Err(invalid("attribute spelling is not canonical"));
        }
        if attnum <= previous {
            return Err(invalid("attributes are not strictly increasing and unique"));
        }
        attnums.push(attnum);
        previous = attnum;
    }
    Ok(attnums)
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
        c"ec_distann.candidate_heap_limit",
        c"FR-081 retained candidate heap limit (L) for ec_distann scans.",
        c"The coordinator retains at most L best unexpanded candidates and derives the owner code-score threshold from the L-th candidate. Each owner applies candidate_limit=L once to the merged response batch.",
        &ECDISTANN_CANDIDATE_HEAP_LIMIT_GUC,
        1,
        ECDISTANN_MAX_CANDIDATE_HEAP_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_string_guc(
        c"ec_distann.benchmark_head_policy",
        c"Task 181 benchmark-only head landmark builder.",
        c"Accepted values are current_sample, geometry_landmarks, graph_landmarks, training_landmarks, training_region_balanced, and training_query_facility. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_HEAD_POLICY_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_string_guc(
        c"ec_distann.benchmark_training_query_path",
        c"Task 181 benchmark-only disjoint training-query TSV path.",
        c"Training landmark policies read this server-local TSV. The file must contain id and JSON-array vector columns and must not be the evaluation slice. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_TRAINING_QUERY_PATH_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_exact_neighbor",
        c"Task 180 benchmark-only exact neighbor scoring oracle.",
        c"When enabled, the physical coordinator replaces persisted RaBitQ neighbor-code distances with exact source-vector distances for the same bounded seed and traversal requests. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_EXACT_NEIGHBOR_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_int_guc(
        c"ec_distann.benchmark_traversal_replica_fail_batch",
        c"Task 198 benchmark-only traversal replica mid-scan fault.",
        c"When nonnegative, the replica expander fails before that zero-based batch. The coordinator must discard replica state and restart on owners. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_TRAVERSAL_REPLICA_FAIL_BATCH_GUC,
        -1,
        ECDISTANN_MAX_HOP_ROUNDS,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_int_guc(
        c"ec_distann.benchmark_materialization_batch_size",
        c"Task 191 benchmark-only final-payload policy override.",
        c"Minus one uses the production fixed lazy-10 policy. Zero selects the eager A/B control. A positive value selects an explicit deterministic ranked-window size for benchmark/test use. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_MATERIALIZATION_BATCH_SIZE_GUC,
        -1,
        ECDISTANN_MAX_BENCHMARK_HEAD_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_owner_payload_plan_cache",
        c"Task 193 benchmark-only owner payload prepared-plan cache arm.",
        c"When enabled, physical payload requests reuse a generation-owned, projection-fingerprinted SPI plan. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_OWNER_PAYLOAD_PLAN_CACHE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_typed_locator",
        c"Task 218 benchmark-only typed row locator arm.",
        c"Bind tid[] instead of formatting ctids as text for owner payload SQL.",
        &ECDISTANN_BENCHMARK_TYPED_LOCATOR_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_packed_payload",
        c"Task 220 benchmark-only packed owner payload arm.",
        c"Pack owner payload bytes into one flat buffer instead of constructing a bytea[] per row.",
        &ECDISTANN_BENCHMARK_PACKED_PAYLOAD_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_expanded_locator",
        c"Task 221 benchmark-only expanded owner locator arm.",
        c"Carry the owner row-tier TID from physical expansion to payload materialization, skipping the owner directory lookup.",
        &ECDISTANN_BENCHMARK_EXPANDED_LOCATOR_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_bool_guc(
        c"ec_distann.benchmark_payload_projection",
        c"Task 222 benchmark-only payload projection arm.",
        c"On uses the production exact/fail-closed attribute mask. Off forces the historical all-column payload control. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_PAYLOAD_PROJECTION_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.gateway_copy_capacity",
        c"TRAV-30 bounded gateway copy capacity (Task 210 P3).",
        c"How many frequently traversed gateway nodes' routing payloads (neighbour ids and neighbour codes -- never full-precision vectors, never the whole graph) a coordinator may cache. The bound is a constant independent of corpus size, which is what distinguishes this from the withdrawn FR-084 full-graph replica. 0 disables gateway copies.",
        &ECDISTANN_GATEWAY_COPY_CAPACITY_GUC,
        0,
        65536,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.crown_capacity",
        c"FR-089 bounded coordinator crown capacity in search-code entries.",
        c"Zero disables the epoch-scoped crown. The bound is independent of corpus size and head size; entries contain only vec_id and quantized search code.",
        &ECDISTANN_CROWN_CAPACITY_GUC,
        0,
        ECDISTANN_MAX_HEAD_INDEX_CAP,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.crown_width_pruning",
        c"FR-089 explicit crown width-pruning measurement arm.",
        c"When enabled, only crown-covered promising head shards may be narrowed; the default preserves the full sharded head fan-out.",
        &ECDISTANN_CROWN_WIDTH_PRUNING_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.fused_head_hop",
        c"FR-090 fused crown seed expansion measurement arm.",
        c"When enabled with a population-complete crown, crown-ranked seed ids enter the first ordinary expansion request and the dedicated head fan-out is skipped.",
        &ECDISTANN_FUSED_HEAD_HOP_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.head_replica_count",
        c"DISTRIBUTEDANN 4.1 head replica count (Task 210 P2b).",
        c"Additional roster nodes that may serve a head shard. 0 means each shard is served by its owner. A replica holds only the bounded head shard, never the O(N) graph or row tier, so it satisfies NFR-021's allowance for replicating a bounded structure.",
        &ECDISTANN_HEAD_REPLICA_COUNT_GUC,
        0,
        64,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.shard_head_storage",
        c"NFR-021 clause 3 membership-only head persistence (Task 210 P2a).",
        c"Read at build time; default on (Task 210 review, 2026-07-31: the sharded shape is the shipped default, not a benchmark arm). On a multi-owner roster the coordinator persists head landmark ids and the head graph but not the landmark vectors, because each landmark's full-precision vector already lives on the owner its FR-078 placement hash selects. Single-owner rosters keep full vectors, where central and distributed coincide. The read path derives from the persisted shape.",
        &ECDISTANN_SHARD_HEAD_STORAGE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.sharded_head_search",
        c"NFR-021 clause 3 sharded FR-080 head search (Task 210 P2a).",
        c"Default on (Task 210 review, 2026-07-31). Each owner searches the head landmarks it owns under the FR-078 placement hash, reading their co-placed full-precision vectors locally, and the coordinator merges bounded per-owner seeds. No landmark vector crosses the wire. A membership-only persisted head always uses this path regardless of the GUC.",
        &ECDISTANN_SHARDED_HEAD_SEARCH_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.allow_nonconforming_replica",
        c"Opt-in for the FR-084 coordinator traversal replica (non-conforming).",
        c"Default off. When enabled, a Ready coordinator traversal replica serves traversal from a coordinator-resident copy of every owner's graph record and full-precision vector, which is O(N) single-node state and does not satisfy NFR-021. The sharded owner path is the default; this accelerator is never a valid decision control under NFR-022.",
        &ECDISTANN_ALLOW_NONCONFORMING_REPLICA_GUC,
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
        c"Initial FR-081 convergence early-exit bar: the scan may stop once k exact results cannot be improved by the beam's best unvisited code distance. When a consumer reads past the proven top-k prefix (e.g. a SQL LIMIT above k), the scan transparently re-runs with a doubled bar (iterative deepening), so correctness does not depend on matching this GUC to the query LIMIT; it is a performance hint.",
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
    GucRegistry::define_bool_guc(
        c"ec_distann.physical_epoch_cache",
        c"Cache validated immutable physical epoch descriptors and head graphs per backend.",
        c"Avoids rebuilding the bounded Vamana head graph and revalidating immutable candidate bytes on every scan open. The cache is bounded and keyed by control OID, logical UUID, build id, and fingerprint. Disable only for A/B measurement or diagnosis.",
        &ECDISTANN_PHYSICAL_EPOCH_CACHE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_string_guc(
        c"ec_distann.replica_control_password_file",
        c"Server-side password file for the dedicated traversal-replica control connection.",
        c"Optional absolute path to a server-owner-readable mode-0600 file containing only the extension-owner password. The control connection never authenticates as the invoking DML user. Leave unset when local peer or certificate authentication is configured. Changes require a configuration reload.",
        &ECDISTANN_REPLICA_CONTROL_PASSWORD_FILE_GUC,
        GucContext::Sighup,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_string_guc(
        c"ec_distann.benchmark_seed_mode",
        c"Task 180 benchmark-only physical seed attribution mode.",
        c"Accepted values are persisted_head, head_sample_exact, head_hierarchy, and owner_scan. This GUC is absent from normal production builds.",
        &ECDISTANN_BENCHMARK_SEED_MODE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_int_guc(
        c"ec_distann.benchmark_head_search_width",
        c"Task 180 benchmark-only approximate head-search width.",
        c"Zero preserves the production width derived from beam_width; positive values decouple approximate head search from the distributed beam.",
        &ECDISTANN_BENCHMARK_HEAD_SEARCH_WIDTH_GUC,
        0,
        ECDISTANN_MAX_BENCHMARK_HEAD_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "distann-head-attribution-benchmark")]
    GucRegistry::define_int_guc(
        c"ec_distann.benchmark_head_seed_count",
        c"Task 180 benchmark-only returned head-seed count.",
        c"Zero preserves the production count derived from beam_width; positive values decouple returned seeds from approximate head-search width.",
        &ECDISTANN_BENCHMARK_HEAD_SEED_COUNT_GUC,
        0,
        ECDISTANN_MAX_BENCHMARK_HEAD_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.remote_connect_timeout_ms",
        c"Connect-timeout budget for ec_distann remote participant RPCs.",
        c"Maximum milliseconds spent opening a remote participant connection. The nonzero default bounds waits while the coordinator holds lifecycle or scan state.",
        &ECDISTANN_REMOTE_CONNECT_TIMEOUT_MS_GUC,
        1,
        ECDISTANN_MAX_REMOTE_TIMEOUT_MS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.remote_statement_timeout_ms",
        c"Statement and client-call timeout budget for ec_distann remote participant RPCs.",
        c"Maximum milliseconds for each remote lifecycle, expansion, or materialization statement. The transport configures the same remote statement_timeout and a bounded client-side deadline.",
        &ECDISTANN_REMOTE_STATEMENT_TIMEOUT_MS_GUC,
        1,
        ECDISTANN_MAX_REMOTE_TIMEOUT_MS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.debug_fail_hop_round",
        c"NFR-020 fault injection (debug): fail an ec_distann scan at this 0-based hop round.",
        c"When >= 0, the scan raises an error at the start of the hop round with this index, after earlier rounds executed. Values >= 1 inject a hop_round_failure_mid_beam fault used by the multinode fault-drill matrix (TC-042). -1 disables injection.",
        &ECDISTANN_DEBUG_FAIL_HOP_ROUND_GUC,
        -1,
        ECDISTANN_MAX_HOP_ROUNDS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_missing_node_record",
        c"NFR-020 fault injection (debug): force an FR-079 case-(c) missing owned record.",
        c"When on, the local expander raises OwnedRecordMissing on the first vec_id it expands, simulating a missing_node_record so the multinode fault-drill matrix (TC-042) can assert the scan errors rather than under-returning. Off by default.",
        &ECDISTANN_DEBUG_MISSING_NODE_RECORD_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "pg_test")]
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_force_frontier_retry",
        c"Task 167 test fault: force one owner frontier lookup miss before retry.",
        c"When on, the first retained physical node lookup in this backend is forced to miss. The intent-gated retry must recover it; pg_test only.",
        &ECDISTANN_DEBUG_FORCE_FRONTIER_RETRY_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_disable_append_when_room",
        c"Task 167 A/B control: disable free-capacity backlink append.",
        c"When on, a backlink target with spare degree follows robust-prune union instead of appending directly. Production defaults on because packet 051's isolated 50k heldout gate rejected append-when-room.",
        &ECDISTANN_DEBUG_DISABLE_APPEND_WHEN_ROOM_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_retry_attribution",
        c"Task 167 diagnostic: record visibility-retry attribution rows.",
        c"When on, visibility retries may write to the fixture-owned public.ec_distann_retry_attribution relation when it exists. Off by default; production scans never enable this implicitly.",
        &ECDISTANN_DEBUG_RETRY_ATTRIBUTION_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(feature = "pg_test")]
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_fail_crown_population",
        c"Task 212 test fault: make bounded crown population fail before serving.",
        c"When on, physical crown population returns no cache so the scan must use the ordinary full-head fallback. This GUC exists only in pg_test builds and is off by default.",
        &ECDISTANN_DEBUG_FAIL_CROWN_POPULATION_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_fail_insert",
        c"NFR-020 fault injection (debug): fail a graph insert after staging, before publish.",
        c"When on, graph_insert_record errors after staging the new node + directory pages but before publishing metadata, simulating a mid-insert failure so the fault-drill matrix (TC-042/TC-043) can assert the aborting transaction rolls staged pages back. Off by default.",
        &ECDISTANN_DEBUG_FAIL_INSERT_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_fail_handoff_after_prepare",
        c"Task 179 fault injection: fail a handoff batch after preparation and before insertion.",
        c"When on, participant stage errors after complete canonical decode and type I/O preparation but before the first row-tier, graph, directory, journal, or progress mutation. Off by default.",
        &ECDISTANN_DEBUG_FAIL_HANDOFF_AFTER_PREPARE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_fail_recover_after_publish_ack",
        c"Task 179 fault injection: fail T4a after participant publish acknowledgement.",
        c"When on, publish recovery errors after exact participant acknowledgement and before active-pointer mutation. Off by default.",
        &ECDISTANN_DEBUG_FAIL_RECOVER_AFTER_PUBLISH_ACK_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_crash_after_replica_control_commit",
        c"Task 199 fault injection: terminate after traversal-replica invalidation commits.",
        c"When on, a mutation that commits Ready-to-Stale through the dedicated control connection terminates its backend before returning the ordinary retry error. Off by default.",
        &ECDISTANN_DEBUG_CRASH_AFTER_REPLICA_CONTROL_COMMIT_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.debug_source_capture_fault",
        c"Task 179 source-callback mismatch fault selector.",
        c"Debug-only deterministic source capture fault: 0 off, 1 absent index TID, 2 vector mismatch, 3 identity mismatch.",
        &ECDISTANN_DEBUG_SOURCE_CAPTURE_FAULT_GUC,
        0,
        3,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_distann.debug_fail_tombstone_write",
        c"NFR-020 fault injection (debug): fail a tombstone write after the flag flip.",
        c"When on, apply_record_writes errors after flipping tombstone flags but before returning, simulating a lost remote tombstone write (mid-delete). The fault-drill matrix (TC-042) asserts the aborting transaction rolls the flags back so the record stays live. Off by default.",
        &ECDISTANN_DEBUG_FAIL_TOMBSTONE_WRITE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_top_k() -> usize {
    usize::try_from(ECDISTANN_TOP_K_GUC.get())
        .unwrap_or(1)
        .max(1)
}

pub(super) fn scan_profile_notice_enabled() -> bool {
    ECDISTANN_SCAN_PROFILE_NOTICE_GUC.get()
}

pub(super) fn physical_epoch_cache_enabled() -> bool {
    ECDISTANN_PHYSICAL_EPOCH_CACHE_GUC.get()
}

pub(super) fn retry_attribution_enabled() -> bool {
    ECDISTANN_DEBUG_RETRY_ATTRIBUTION_GUC.get()
}

pub(super) fn debug_fail_crown_population() -> bool {
    #[cfg(feature = "pg_test")]
    {
        return ECDISTANN_DEBUG_FAIL_CROWN_POPULATION_GUC.get();
    }
    #[cfg(not(feature = "pg_test"))]
    false
}

pub(super) fn materialization_batch_size() -> usize {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return usize::try_from(ECDISTANN_BENCHMARK_MATERIALIZATION_BATCH_SIZE_GUC.get())
            .unwrap_or(PRODUCTION_MATERIALIZATION_BATCH_SIZE);
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    PRODUCTION_MATERIALIZATION_BATCH_SIZE
}

pub(super) fn benchmark_owner_payload_plan_cache() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_OWNER_PAYLOAD_PLAN_CACHE_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    false
}

pub(super) fn benchmark_typed_locator() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_TYPED_LOCATOR_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    false
}

pub(super) fn benchmark_packed_payload() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_PACKED_PAYLOAD_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    false
}

pub(super) fn benchmark_expanded_locator() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_EXPANDED_LOCATOR_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    false
}

pub(super) fn payload_projection_enabled() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_PAYLOAD_PROJECTION_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    true
}

pub(super) fn benchmark_traversal_replica_fail_batch() -> Option<usize> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return usize::try_from(ECDISTANN_BENCHMARK_TRAVERSAL_REPLICA_FAIL_BATCH_GUC.get()).ok();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicalSeedMode {
    PersistedHead,
    HeadSampleExact,
    HeadHierarchy,
    OwnerScan,
}

impl PhysicalSeedMode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::PersistedHead => "persisted_head",
            Self::HeadSampleExact => "head_sample_exact",
            Self::HeadHierarchy => "head_hierarchy",
            Self::OwnerScan => "owner_scan",
        }
    }
}

pub(super) fn current_physical_seed_mode() -> Result<PhysicalSeedMode, String> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        let configured = ECDISTANN_BENCHMARK_SEED_MODE_GUC
            .get()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let configured = configured.trim();
        if configured.is_empty() {
            return Ok(if cfg!(feature = "distann-legacy-seed-benchmark") {
                PhysicalSeedMode::OwnerScan
            } else {
                PhysicalSeedMode::PersistedHead
            });
        }
        return match configured {
            "persisted_head" => Ok(PhysicalSeedMode::PersistedHead),
            "head_sample_exact" => Ok(PhysicalSeedMode::HeadSampleExact),
            "head_hierarchy" => Ok(PhysicalSeedMode::HeadHierarchy),
            "owner_scan" => Ok(PhysicalSeedMode::OwnerScan),
            other => Err(format!(
                "EC_BAD_INPUT: ec_distann.benchmark_seed_mode must be persisted_head, head_sample_exact, head_hierarchy, or owner_scan; got {other:?}"
            )),
        };
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        Ok(PhysicalSeedMode::PersistedHead)
    }
}

pub(super) fn current_head_search_width(production_default: usize) -> usize {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return usize::try_from(ECDISTANN_BENCHMARK_HEAD_SEARCH_WIDTH_GUC.get())
            .ok()
            .filter(|value| *value > 0)
            .unwrap_or(production_default);
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        production_default
    }
}

pub(super) fn current_head_seed_count(production_default: usize) -> usize {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return usize::try_from(ECDISTANN_BENCHMARK_HEAD_SEED_COUNT_GUC.get())
            .ok()
            .filter(|value| *value > 0)
            .unwrap_or(production_default);
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        production_default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BenchmarkHeadPolicy {
    CurrentSample,
    GeometryLandmarks,
    GraphLandmarks,
    TrainingLandmarks,
    TrainingRegionBalanced,
    TrainingQueryFacility,
}

impl BenchmarkHeadPolicy {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentSample => "current_sample",
            Self::GeometryLandmarks => "geometry_landmarks",
            Self::GraphLandmarks => "graph_landmarks",
            Self::TrainingLandmarks => "training_landmarks",
            Self::TrainingRegionBalanced => "training_region_balanced",
            Self::TrainingQueryFacility => "training_query_facility",
        }
    }
}

pub(crate) fn current_benchmark_head_policy() -> Result<BenchmarkHeadPolicy, String> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        let configured = ECDISTANN_BENCHMARK_HEAD_POLICY_GUC
            .get()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        return match configured.trim() {
            "" | "current_sample" => Ok(BenchmarkHeadPolicy::CurrentSample),
            "geometry_landmarks" => Ok(BenchmarkHeadPolicy::GeometryLandmarks),
            "graph_landmarks" => Ok(BenchmarkHeadPolicy::GraphLandmarks),
            "training_landmarks" => Ok(BenchmarkHeadPolicy::TrainingLandmarks),
            "training_region_balanced" => Ok(BenchmarkHeadPolicy::TrainingRegionBalanced),
            "training_query_facility" => Ok(BenchmarkHeadPolicy::TrainingQueryFacility),
            other => Err(format!(
                "EC_BAD_INPUT: ec_distann.benchmark_head_policy has unsupported value {other:?}"
            )),
        };
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        Ok(BenchmarkHeadPolicy::CurrentSample)
    }
}

pub(crate) fn benchmark_training_query_path() -> Option<String> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_TRAINING_QUERY_PATH_GUC
            .get()
            .map(|value| value.to_string_lossy().trim().to_owned())
            .filter(|value| !value.is_empty());
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        None
    }
}

pub(super) fn benchmark_exact_neighbor() -> bool {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        return ECDISTANN_BENCHMARK_EXACT_NEIGHBOR_GUC.get();
    }
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    {
        false
    }
}

pub(super) fn remote_connect_timeout_ms() -> u64 {
    u64::try_from(ECDISTANN_REMOTE_CONNECT_TIMEOUT_MS_GUC.get())
        .unwrap_or(ECDISTANN_DEFAULT_REMOTE_CONNECT_TIMEOUT_MS as u64)
        .max(1)
}

pub(super) fn remote_statement_timeout_ms() -> u64 {
    u64::try_from(ECDISTANN_REMOTE_STATEMENT_TIMEOUT_MS_GUC.get())
        .unwrap_or(ECDISTANN_DEFAULT_REMOTE_STATEMENT_TIMEOUT_MS as u64)
        .max(1)
}

pub(super) fn replica_control_password_file() -> Option<String> {
    ECDISTANN_REPLICA_CONTROL_PASSWORD_FILE_GUC
        .get()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
}

pub(super) fn current_beam_width() -> usize {
    usize::try_from(ECDISTANN_BEAM_WIDTH_GUC.get())
        .unwrap_or(1)
        .max(1)
}

pub(super) fn current_candidate_heap_limit() -> usize {
    usize::try_from(ECDISTANN_CANDIDATE_HEAP_LIMIT_GUC.get())
        .unwrap_or(1)
        .max(1)
}

/// NFR-021 clause 4 (Task 210 P1): the non-conforming traversal replica is
/// opened only under an explicit opt-in. Off by default in every build profile.
/// NFR-021 clause 3 (Task 210 P2a): search the head as roster shards.
/// DISTRIBUTEDANN 4.1 head replica count (Task 210 P2b).
/// TRAV-30 bounded gateway copy capacity (Task 210 P3).
pub(super) fn gateway_copy_capacity() -> usize {
    usize::try_from(ECDISTANN_GATEWAY_COPY_CAPACITY_GUC.get()).unwrap_or(0)
}

pub(super) fn head_replica_count() -> usize {
    usize::try_from(ECDISTANN_HEAD_REPLICA_COUNT_GUC.get()).unwrap_or(0)
}

/// NFR-021 clause 3 (Task 210 P2a): persist the head as membership only.
pub(super) fn shard_head_storage() -> bool {
    ECDISTANN_SHARD_HEAD_STORAGE_GUC.get()
}

pub(super) fn sharded_head_search() -> bool {
    ECDISTANN_SHARDED_HEAD_SEARCH_GUC.get()
}

pub(super) fn allow_nonconforming_replica() -> bool {
    ECDISTANN_ALLOW_NONCONFORMING_REPLICA_GUC.get()
}

pub(super) fn current_hop_rounds() -> usize {
    usize::try_from(ECDISTANN_HOP_ROUNDS_GUC.get())
        .unwrap_or(1)
        .max(1)
}

/// NFR-020 fault injection: the 0-based hop round to fail at, or `None` when
/// disabled (the -1 default). See `ec_distann.debug_fail_hop_round`.
pub(super) fn debug_fail_hop_round() -> Option<usize> {
    usize::try_from(ECDISTANN_DEBUG_FAIL_HOP_ROUND_GUC.get()).ok()
}

/// NFR-020 fault injection: force the FR-079 case-(c) missing-record fault.
pub(super) fn debug_missing_node_record() -> bool {
    ECDISTANN_DEBUG_MISSING_NODE_RECORD_GUC.get()
}

#[cfg(feature = "pg_test")]
pub(super) fn debug_force_frontier_retry() -> bool {
    ECDISTANN_DEBUG_FORCE_FRONTIER_RETRY_GUC.get()
}

pub(super) fn debug_disable_append_when_room() -> bool {
    ECDISTANN_DEBUG_DISABLE_APPEND_WHEN_ROOM_GUC.get()
}

/// NFR-020 fault injection: fail a graph insert after staging, before publish.
pub(super) fn debug_fail_insert() -> bool {
    ECDISTANN_DEBUG_FAIL_INSERT_GUC.get()
}

pub(super) fn debug_fail_handoff_after_prepare() -> bool {
    ECDISTANN_DEBUG_FAIL_HANDOFF_AFTER_PREPARE_GUC.get()
}

pub(super) fn debug_fail_recover_after_publish_ack() -> bool {
    ECDISTANN_DEBUG_FAIL_RECOVER_AFTER_PUBLISH_ACK_GUC.get()
}

pub(super) fn debug_crash_after_replica_control_commit() -> bool {
    ECDISTANN_DEBUG_CRASH_AFTER_REPLICA_CONTROL_COMMIT_GUC.get()
}

pub(super) fn debug_source_capture_fault() -> i32 {
    ECDISTANN_DEBUG_SOURCE_CAPTURE_FAULT_GUC.get()
}

/// NFR-020 fault injection: fail a tombstone write after the flag flip.
pub(super) fn debug_fail_tombstone_write() -> bool {
    ECDISTANN_DEBUG_FAIL_TOMBSTONE_WRITE_GUC.get()
}

pub(super) fn crown_capacity() -> usize {
    usize::try_from(ECDISTANN_CROWN_CAPACITY_GUC.get()).unwrap_or(0)
}

pub(super) fn crown_width_pruning() -> bool {
    ECDISTANN_CROWN_WIDTH_PRUNING_GUC.get()
}

pub(super) fn fused_head_hop() -> bool {
    ECDISTANN_FUSED_HEAD_HOP_GUC.get()
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
            b"head_sampling_rate\0".as_ptr().cast(),
            b"FR-088 head capacity sampling rate; zero keeps the explicit head_index_cap law-inactive.\0"
                .as_ptr()
                .cast(),
            0.0,
            -1.0e308,
            1.0e308,
            offset_of!(EcDistannReloptions, head_sampling_rate) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"head_cap_floor\0".as_ptr().cast(),
            b"FR-088 lower bound for the resolved head sampling law.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
            i32::MIN,
            i32::MAX,
            offset_of!(EcDistannReloptions, head_cap_floor) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"head_cap_ceiling\0".as_ptr().cast(),
            b"FR-088 upper bound for the resolved head sampling law.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_MAX_HEAD_INDEX_CAP,
            i32::MIN,
            i32::MAX,
            offset_of!(EcDistannReloptions, head_cap_ceiling) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"build_shards\0".as_ptr().cast(),
            b"FR-077 build-shard count: 0 = auto, 1 = monolithic fallback, >=2 sharded closure-overlap build + stitch.\0"
                .as_ptr()
                .cast(),
            ECDISTANN_DEFAULT_BUILD_SHARDS,
            ECDISTANN_MIN_BUILD_SHARDS,
            ECDISTANN_MAX_BUILD_SHARDS,
            offset_of!(EcDistannReloptions, build_shards) as i32,
        );
        pg_sys::add_local_bool_reloption(
            &mut relopts,
            b"distributed_control\0".as_ptr().cast(),
            b"Build-mode option: create a Task 179 logical distributed-control index with no local graph records. ALTER takes effect only on an explicit identity/state-destroying REINDEX.\0"
                .as_ptr()
                .cast(),
            false,
            offset_of!(EcDistannReloptions, distributed_control) as i32,
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
            b"FR-076 neighbor-code codec: 'rabitq' (default, ADR-085 D7 M0 measurement), 'grouped_pq', or 'turboquant'.\0"
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
        pg_sys::add_local_string_reloption(
            &mut relopts,
            b"head_construction\0".as_ptr().cast(),
            b"Task 207 head construction: 'stitched_bfs' control or 'partition_union' candidate.\0"
                .as_ptr()
                .cast(),
            ptr::null(),
            None,
            None,
            offset_of!(EcDistannReloptions, head_construction_offset) as i32,
        );
        pg_sys::add_local_string_reloption(
            &mut relopts,
            b"covering_payload_attnums\0".as_ptr().cast(),
            b"Task 229 build-time scalar payload cover as canonical, strictly increasing physical attnums (maximum 16).\0"
                .as_ptr()
                .cast(),
            ptr::null(),
            None,
            None,
            offset_of!(EcDistannReloptions, covering_payload_attnums_offset) as i32,
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
            Some(value) => {
                NeighborCodeFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
            }
            None => NeighborCodeFormat::DEFAULT,
        };
        let source_identity = match self
            .read_string_reloption(reloptions.source_identity_offset, "source_identity")
        {
            Some(value) => DistannSourceIdentityProvider::parse_reloption(&value)
                .unwrap_or_else(|e| pgrx::error!("{e}")),
            None => DistannSourceIdentityProvider::None,
        };
        let head_construction = match self
            .read_string_reloption(reloptions.head_construction_offset, "head_construction")
        {
            Some(value) => {
                HeadConstruction::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
            }
            None => HeadConstruction::DEFAULT,
        };
        let covering_payload_attnums = self
            .read_string_reloption(
                reloptions.covering_payload_attnums_offset,
                "covering_payload_attnums",
            )
            .map(|value| {
                parse_covering_payload_attnums(&value)
                    .unwrap_or_else(|error| pgrx::error!("{error}"))
            });
        if covering_payload_attnums.is_some() && !reloptions.distributed_control {
            pgrx::error!(
                "invalid ec_distann covering_payload_attnums reloption: a payload cover requires distributed_control=true"
            );
        }

        EcDistannOptions {
            graph_degree: reloptions.graph_degree,
            build_list_size: reloptions.build_list_size,
            head_index_cap: reloptions.head_index_cap,
            head_sampling_rate: reloptions.head_sampling_rate,
            head_cap_floor: reloptions.head_cap_floor,
            head_cap_ceiling: reloptions.head_cap_ceiling,
            build_shards: reloptions.build_shards,
            alpha: reloptions.alpha as f32,
            closure_epsilon: reloptions.closure_epsilon as f32,
            neighbor_code_format,
            source_identity,
            distributed_control: reloptions.distributed_control,
            head_construction,
            covering_payload_attnums,
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
        parse_covering_payload_attnums, DistannSourceIdentityProvider, EcDistannOptions,
        NeighborCodeFormat, ECDISTANN_DEFAULT_HEAD_INDEX_CAP,
    };

    #[test]
    fn distann_default_options_match_spec_defaults() {
        let defaults = EcDistannOptions::DEFAULT;
        assert_eq!(defaults.graph_degree, 32);
        assert_eq!(defaults.build_list_size, 100);
        assert_eq!(defaults.head_index_cap, ECDISTANN_DEFAULT_HEAD_INDEX_CAP);
        assert_eq!(defaults.head_index_cap, 4096);
        assert_eq!(defaults.neighbor_code_format, NeighborCodeFormat::RaBitQ);
        assert!(!defaults.distributed_control);
        assert_eq!(
            defaults.head_construction,
            super::HeadConstruction::StitchedBfs
        );
        assert_eq!(
            defaults.source_identity,
            DistannSourceIdentityProvider::None
        );
        assert!(defaults.covering_payload_attnums.is_none());
    }

    #[test]
    fn covering_payload_attnums_requires_canonical_bounded_order() {
        assert_eq!(
            parse_covering_payload_attnums("1,3,16").unwrap(),
            vec![1, 3, 16]
        );
        for invalid in [
            "", "0", "01", "1,", ",1", "1, 2", "2,1", "1,1", "65536", "1,+2",
        ] {
            assert!(
                parse_covering_payload_attnums(invalid).is_err(),
                "{invalid:?} must be rejected"
            );
        }
        let too_many = (1..=17)
            .map(|attnum| attnum.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert!(parse_covering_payload_attnums(&too_many).is_err());
    }

    #[test]
    fn distann_default_backlink_strategy_retains_measured_robust_prune() {
        assert!(super::debug_disable_append_when_room());
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

    #[test]
    fn head_sizing_reloption_validation_rejects_invalid_bounds() {
        let mut options = EcDistannOptions::DEFAULT;
        options.head_sampling_rate = -0.01;
        assert!(options.validate_head_sizing_inputs().is_err());

        options = EcDistannOptions::DEFAULT;
        options.head_cap_floor = 8192;
        options.head_cap_ceiling = 4096;
        assert!(options.validate_head_sizing_inputs().is_err());

        options = EcDistannOptions::DEFAULT;
        options.head_cap_floor = 8;
        assert!(options.validate_head_sizing_inputs().is_err());
    }

    #[test]
    fn head_sizing_resolution_covers_untrained_and_trained_policies() {
        let mut options = EcDistannOptions::DEFAULT;
        let (explicit, attestation) = options.resolve_head_sizing(100_000, false).unwrap();
        assert_eq!(explicit, 4096);
        assert!(attestation.is_none());

        options.head_sampling_rate = 0.02;
        options.head_cap_floor = 16;
        options.head_cap_ceiling = 4096;
        let (resolved, attestation) = options.resolve_head_sizing(100_000, false).unwrap();
        assert_eq!(resolved, 2000);
        assert!(attestation.is_some());
        assert!(options.resolve_head_sizing(100_000, true).is_err());

        options.head_sampling_rate = 0.04096;
        let (trained, attestation) = options.resolve_head_sizing(100_000, true).unwrap();
        assert_eq!(trained, 4096);
        assert!(attestation.is_some());
    }
}
