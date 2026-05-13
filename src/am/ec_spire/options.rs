use std::ffi::CString;
use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};

use super::quantizer::SpireAssignmentPayloadFormat;
use super::{
    EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT, EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
    EC_SPIRE_DEFAULT_MAX_CANDIDATE_ROWS, EC_SPIRE_DEFAULT_NLISTS, EC_SPIRE_DEFAULT_NPROBE,
    EC_SPIRE_DEFAULT_PQ_GROUP_SIZE, EC_SPIRE_DEFAULT_RECURSIVE_FANOUT,
    EC_SPIRE_DEFAULT_RERANK_WIDTH, EC_SPIRE_DEFAULT_SEED, EC_SPIRE_DEFAULT_TOP_GRAPH_ALPHA,
    EC_SPIRE_DEFAULT_TOP_GRAPH_BUILD_LIST_SIZE, EC_SPIRE_DEFAULT_TOP_GRAPH_DEGREE,
    EC_SPIRE_DEFAULT_TOP_GRAPH_ENABLED, EC_SPIRE_DEFAULT_TOP_GRAPH_SEARCH_LIST_SIZE,
    EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS, EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT,
    EC_SPIRE_MAX_LOCAL_STORE_COUNT, EC_SPIRE_MAX_MAX_CANDIDATE_ROWS, EC_SPIRE_MAX_NLISTS,
    EC_SPIRE_MAX_NPROBE, EC_SPIRE_MAX_PQ_GROUP_SIZE, EC_SPIRE_MAX_RECURSIVE_FANOUT,
    EC_SPIRE_MAX_RERANK_WIDTH, EC_SPIRE_MAX_SEED, EC_SPIRE_MAX_TOP_GRAPH_ALPHA,
    EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE, EC_SPIRE_MAX_TOP_GRAPH_DEGREE,
    EC_SPIRE_MAX_TOP_GRAPH_ENABLED, EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE,
    EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS, EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT,
    EC_SPIRE_MIN_LOCAL_STORE_COUNT, EC_SPIRE_MIN_MAX_CANDIDATE_ROWS, EC_SPIRE_MIN_NLISTS,
    EC_SPIRE_MIN_NPROBE, EC_SPIRE_MIN_PQ_GROUP_SIZE, EC_SPIRE_MIN_RECURSIVE_FANOUT,
    EC_SPIRE_MIN_RERANK_WIDTH, EC_SPIRE_MIN_SEED, EC_SPIRE_MIN_TOP_GRAPH_ALPHA,
    EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE, EC_SPIRE_MIN_TOP_GRAPH_DEGREE,
    EC_SPIRE_MIN_TOP_GRAPH_ENABLED, EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE,
    EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS,
};

const EC_SPIRE_SESSION_NPROBE_UNSET: i32 = -1;
const EC_SPIRE_SESSION_RERANK_WIDTH_UNSET: i32 = -1;
const EC_SPIRE_SESSION_MAX_CANDIDATE_ROWS_UNSET: i32 = -1;
const EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES: usize = 32;
const EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1000;
const EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1_000_000;
const EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET: i32 = 0;
const EC_SPIRE_MAX_REMOTE_SEARCH_LIMIT: i32 = 1_000_000;
const EC_SPIRE_MAX_REMOTE_SEARCH_CONCURRENCY_LIMIT: i32 = 4096;
const EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS: i32 = 0;
const EC_SPIRE_MAX_REMOTE_SEARCH_TIMEOUT_MS: i32 = 3_600_000;
const EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW: i32 = 1024;
const EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW: i32 = 1_073_741_824;
const EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH: i32 = 64;
const EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_COST_ROUTING_DIMENSION_SCALE: f64 = 0.01;
pub(super) const EC_SPIRE_DEFAULT_COST_LEAF_DIMENSION_SCALE: f64 = 0.01;
pub(super) const EC_SPIRE_DEFAULT_COST_INDEX_PAGE_SCALE: f64 = 1.0;
pub(super) const EC_SPIRE_DEFAULT_COST_LOCAL_STORE_PAGE_FANOUT_SCALE: f64 = 0.05;
pub(super) const EC_SPIRE_DEFAULT_COST_STORAGE_SCORING_MULTIPLIER: f64 = 1.0;
pub(super) const EC_SPIRE_DEFAULT_COST_RERANK_MULTIPLIER: f64 = 1.35;
const EC_SPIRE_MAX_COST_SCALE: f64 = 1_000_000.0;
#[cfg(any(test, feature = "pg_test"))]
const EC_SPIRE_MAX_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE: i32 = 10_000;

static EC_SPIRE_NPROBE_GUC: GucSetting<i32> = GucSetting::<i32>::new(EC_SPIRE_SESSION_NPROBE_UNSET);
static EC_SPIRE_RERANK_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_RERANK_WIDTH_UNSET);
static EC_SPIRE_MAX_CANDIDATE_ROWS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_MAX_CANDIDATE_ROWS_UNSET);
static EC_SPIRE_ADAPTIVE_NPROBE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static EC_SPIRE_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS);
static EC_SPIRE_REMOTE_SEARCH_MAX_NODES_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET);
static EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET);
static EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_PER_NODE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET);
static EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET);
static EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_PER_NODE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET);
static EC_SPIRE_REMOTE_SEARCH_CONNECT_TIMEOUT_MS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS);
static EC_SPIRE_REMOTE_SEARCH_STATEMENT_TIMEOUT_MS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS);
static EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW);
static EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH);
static EC_SPIRE_COST_ROUTING_DIMENSION_SCALE_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_ROUTING_DIMENSION_SCALE);
static EC_SPIRE_COST_LEAF_DIMENSION_SCALE_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_LEAF_DIMENSION_SCALE);
static EC_SPIRE_COST_INDEX_PAGE_SCALE_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_INDEX_PAGE_SCALE);
static EC_SPIRE_COST_LOCAL_STORE_PAGE_FANOUT_SCALE_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_LOCAL_STORE_PAGE_FANOUT_SCALE);
static EC_SPIRE_COST_STORAGE_SCORING_MULTIPLIER_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_STORAGE_SCORING_MULTIPLIER);
static EC_SPIRE_COST_RERANK_MULTIPLIER_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_COST_RERANK_MULTIPLIER);
static EC_SPIRE_REMOTE_SEARCH_CONSISTENCY_MODE_GUC: GucSetting<
    SpireRemoteSearchConsistencyModeGuc,
> = GucSetting::<SpireRemoteSearchConsistencyModeGuc>::new(
    SpireRemoteSearchConsistencyModeGuc::Strict,
);
static EC_SPIRE_REMOTE_TUPLE_TRANSPORT_GUC: GucSetting<SpireRemoteTupleTransportGuc> =
    GucSetting::<SpireRemoteTupleTransportGuc>::new(SpireRemoteTupleTransportGuc::Auto);
#[cfg(any(test, feature = "pg_test"))]
static EC_SPIRE_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PostgresGucEnum)]
pub(super) enum SpireRemoteSearchConsistencyModeGuc {
    #[name = c"strict"]
    Strict,
    #[name = c"degraded"]
    Degraded,
}

impl SpireRemoteSearchConsistencyModeGuc {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PostgresGucEnum)]
pub(super) enum SpireRemoteTupleTransportGuc {
    #[name = c"auto"]
    Auto,
    #[name = c"json_tuple_payload_v1"]
    JsonTuplePayloadV1,
    #[name = c"pg_binary_attr_v1"]
    PgBinaryAttrV1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcSpireReloptions {
    vl_len_: i32,
    nlists: i32,
    recursive_fanout: i32,
    local_store_count: i32,
    boundary_replica_count: i32,
    nprobe: i32,
    rerank_width: i32,
    max_candidate_rows: i32,
    training_sample_rows: i32,
    seed: i32,
    pq_group_size: i32,
    top_graph_enabled: i32,
    top_graph_degree: i32,
    top_graph_build_list_size: i32,
    top_graph_alpha: f64,
    top_graph_search_list_size: i32,
    nprobe_per_level_offset: i32,
    storage_format_offset: i32,
    quantizer_offset: i32,
    source_identity_offset: i32,
    local_store_tablespaces_offset: i32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireStorageFormat {
    Auto = 0,
    TurboQuant = 1,
    PqFastScan = 2,
    RaBitQ = 3,
}

impl SpireStorageFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "turboquant" => Ok(Self::TurboQuant),
            "pq_fastscan" => Ok(Self::PqFastScan),
            "rabitq" => Ok(Self::RaBitQ),
            other => Err(format!(
                "invalid ec_spire storage_format reloption: expected 'auto', 'turboquant', 'pq_fastscan', or 'rabitq', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::TurboQuant => "turboquant",
            Self::PqFastScan => "pq_fastscan",
            Self::RaBitQ => "rabitq",
        }
    }

    pub(super) fn assignment_payload_format(self) -> SpireAssignmentPayloadFormat {
        match self {
            Self::Auto | Self::TurboQuant => SpireAssignmentPayloadFormat::TurboQuant,
            Self::PqFastScan => SpireAssignmentPayloadFormat::PqFastScan,
            Self::RaBitQ => SpireAssignmentPayloadFormat::RaBitQ,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireSourceIdentityProvider {
    None,
    Include,
}

impl SpireSourceIdentityProvider {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "include" => Ok(Self::Include),
            other => Err(format!(
                "invalid ec_spire source_identity reloption: expected 'include', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Include => "include",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EcSpireOptions {
    pub(super) nlists: i32,
    pub(super) recursive_fanout: i32,
    pub(super) local_store_count: i32,
    pub(super) boundary_replica_count: i32,
    pub(super) nprobe: i32,
    pub(super) rerank_width: i32,
    pub(super) max_candidate_rows: i32,
    pub(super) training_sample_rows: i32,
    pub(super) seed: i32,
    pub(super) pq_group_size: i32,
    pub(super) top_graph_enabled: i32,
    pub(super) top_graph_degree: i32,
    pub(super) top_graph_build_list_size: i32,
    pub(super) top_graph_alpha: f32,
    pub(super) top_graph_search_list_size: i32,
    pub(super) nprobe_per_level: Option<Vec<u32>>,
    pub(super) storage_format: SpireStorageFormat,
    pub(super) source_identity: SpireSourceIdentityProvider,
    pub(super) local_store_tablespaces: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireTopGraphOptionPlan {
    pub(super) enabled: bool,
    pub(super) graph_degree: u32,
    pub(super) build_list_size: u32,
    pub(super) alpha: f32,
    pub(super) search_list_size: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireLocalStoreTablespacePlanEntry {
    pub(super) local_store_id: u32,
    pub(super) tablespace_oid: u32,
}

impl EcSpireOptions {
    pub(super) const DEFAULT: Self = Self {
        nlists: EC_SPIRE_DEFAULT_NLISTS,
        recursive_fanout: EC_SPIRE_DEFAULT_RECURSIVE_FANOUT,
        local_store_count: EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
        boundary_replica_count: EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT,
        nprobe: EC_SPIRE_DEFAULT_NPROBE,
        rerank_width: EC_SPIRE_DEFAULT_RERANK_WIDTH,
        max_candidate_rows: EC_SPIRE_DEFAULT_MAX_CANDIDATE_ROWS,
        training_sample_rows: EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS,
        seed: EC_SPIRE_DEFAULT_SEED,
        pq_group_size: EC_SPIRE_DEFAULT_PQ_GROUP_SIZE,
        top_graph_enabled: EC_SPIRE_DEFAULT_TOP_GRAPH_ENABLED,
        top_graph_degree: EC_SPIRE_DEFAULT_TOP_GRAPH_DEGREE,
        top_graph_build_list_size: EC_SPIRE_DEFAULT_TOP_GRAPH_BUILD_LIST_SIZE,
        top_graph_alpha: EC_SPIRE_DEFAULT_TOP_GRAPH_ALPHA,
        top_graph_search_list_size: EC_SPIRE_DEFAULT_TOP_GRAPH_SEARCH_LIST_SIZE,
        nprobe_per_level: None,
        storage_format: SpireStorageFormat::Auto,
        source_identity: SpireSourceIdentityProvider::None,
        local_store_tablespaces: None,
    };

    pub(super) fn requested_pq_group_size(&self) -> Option<usize> {
        if self.pq_group_size > 0 {
            Some(self.pq_group_size as usize)
        } else {
            None
        }
    }

    pub(super) fn assignment_payload_format(&self) -> SpireAssignmentPayloadFormat {
        self.storage_format.assignment_payload_format()
    }

    pub(super) fn recursive_fanout(&self) -> Option<u32> {
        validate_recursive_fanout_value(self.recursive_fanout)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        match self.recursive_fanout {
            0 => None,
            value if value >= 2 => Some(value as u32),
            _ => unreachable!("recursive_fanout validation rejects value 1"),
        }
    }

    pub(super) fn top_graph_plan(&self) -> Result<SpireTopGraphOptionPlan, String> {
        validate_top_graph_enabled_value(self.top_graph_enabled)?;
        validate_top_graph_degree_value(self.top_graph_degree)?;
        validate_top_graph_build_list_size_value(self.top_graph_build_list_size)?;
        validate_top_graph_alpha_value(self.top_graph_alpha)?;
        validate_top_graph_search_list_size_value(self.top_graph_search_list_size)?;
        Ok(SpireTopGraphOptionPlan {
            enabled: self.top_graph_enabled != 0,
            graph_degree: u32::try_from(self.top_graph_degree)
                .map_err(|_| "ec_spire top_graph_degree reloption must fit u32".to_owned())?,
            build_list_size: u32::try_from(self.top_graph_build_list_size).map_err(|_| {
                "ec_spire top_graph_build_list_size reloption must fit u32".to_owned()
            })?,
            alpha: self.top_graph_alpha,
            search_list_size: match self.top_graph_search_list_size {
                0 => None,
                value => Some(u32::try_from(value).map_err(|_| {
                    "ec_spire top_graph_search_list_size reloption must fit u32".to_owned()
                })?),
            },
        })
    }

    pub(super) fn nprobe_per_level_values(&self) -> Vec<u32> {
        self.nprobe_per_level.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRecursiveNprobePolicy {
    leaf_level_nprobe: u32,
    nprobe_per_level: [u32; EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES],
    nprobe_per_level_len: usize,
    adaptive_nprobe: bool,
    adaptive_score_gap_micros: i32,
}

impl SpireRecursiveNprobePolicy {
    pub(super) fn conservative(leaf_level_nprobe: u32) -> Result<Self, String> {
        Self::from_level_values(leaf_level_nprobe, Vec::new())
    }

    pub(super) fn from_level_values(
        leaf_level_nprobe: u32,
        nprobe_per_level: Vec<u32>,
    ) -> Result<Self, String> {
        Self::from_level_values_with_adaptive(
            leaf_level_nprobe,
            nprobe_per_level,
            false,
            EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
        )
    }

    pub(super) fn from_level_values_with_adaptive(
        leaf_level_nprobe: u32,
        nprobe_per_level: Vec<u32>,
        adaptive_nprobe: bool,
        adaptive_score_gap_micros: i32,
    ) -> Result<Self, String> {
        if leaf_level_nprobe == 0 && !nprobe_per_level.is_empty() {
            return Err(
                "ec_spire recursive scan requires leaf-level nprobe > 0 when per-level nprobe is configured"
                    .to_owned(),
            );
        }
        if nprobe_per_level.len() > EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES {
            return Err(format!(
                "ec_spire nprobe_per_level supports at most {EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES} entries"
            ));
        }
        if nprobe_per_level.contains(&0) {
            return Err("ec_spire nprobe_per_level entries must be greater than 0".to_owned());
        }
        validate_adaptive_nprobe_score_gap_micros(adaptive_score_gap_micros)?;
        let nprobe_per_level_len = nprobe_per_level.len();
        let mut nprobe_per_level_array = [0; EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES];
        nprobe_per_level_array[..nprobe_per_level_len].copy_from_slice(&nprobe_per_level);
        Ok(Self {
            leaf_level_nprobe,
            nprobe_per_level: nprobe_per_level_array,
            nprobe_per_level_len,
            adaptive_nprobe,
            adaptive_score_gap_micros,
        })
    }

    pub(super) fn nprobe_for_parent_level(&self, parent_level: u16) -> u32 {
        if parent_level <= 1 {
            return self.leaf_level_nprobe;
        }
        let level_index = usize::from(parent_level - 2);
        if level_index < self.nprobe_per_level_len {
            self.nprobe_per_level[level_index]
        } else {
            1
        }
    }

    pub(super) fn configured_nprobe_for_level(&self, level: u16) -> Option<u32> {
        if level <= 1 {
            return None;
        }
        let level_index = usize::from(level - 2);
        if level_index < self.nprobe_per_level_len {
            Some(self.nprobe_per_level[level_index])
        } else {
            None
        }
    }

    pub(super) fn adaptive_nprobe(&self) -> bool {
        self.adaptive_nprobe
    }

    pub(super) fn adaptive_score_gap_micros(&self) -> i32 {
        self.adaptive_score_gap_micros
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRecursiveRouteBudget {
    pub(super) beam_width: usize,
    pub(super) max_leaf_routes: usize,
    pub(super) max_routing_expansions: usize,
}

impl SpireRecursiveRouteBudget {
    pub(super) const fn unbounded() -> Self {
        Self {
            beam_width: usize::MAX,
            max_leaf_routes: usize::MAX,
            max_routing_expansions: usize::MAX,
        }
    }
}

fn validate_recursive_fanout_value(value: i32) -> Result<(), String> {
    if value == 0 || value >= 2 {
        Ok(())
    } else {
        Err("ec_spire recursive_fanout reloption must be 0 or at least 2".to_owned())
    }
}

fn validate_local_store_count_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_LOCAL_STORE_COUNT..=EC_SPIRE_MAX_LOCAL_STORE_COUNT).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire local_store_count reloption must be between {EC_SPIRE_MIN_LOCAL_STORE_COUNT} and {EC_SPIRE_MAX_LOCAL_STORE_COUNT}, got {value}"
        ))
    }
}

fn validate_boundary_replica_count_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT..=EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT).contains(&value)
    {
        Ok(())
    } else {
        Err(format!(
            "ec_spire boundary_replica_count reloption must be between {EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT} and {EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT}, got {value}"
        ))
    }
}

fn validate_max_candidate_rows_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_MAX_CANDIDATE_ROWS..=EC_SPIRE_MAX_MAX_CANDIDATE_ROWS).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire max_candidate_rows reloption must be between {EC_SPIRE_MIN_MAX_CANDIDATE_ROWS} and {EC_SPIRE_MAX_MAX_CANDIDATE_ROWS}, got {value}"
        ))
    }
}

fn validate_adaptive_nprobe_score_gap_micros(value: i32) -> Result<(), String> {
    if (0..=EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire adaptive_nprobe_score_gap_micros must be between 0 and {EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS}, got {value}"
        ))
    }
}

fn validate_top_graph_enabled_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_TOP_GRAPH_ENABLED..=EC_SPIRE_MAX_TOP_GRAPH_ENABLED).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire top_graph_enabled reloption must be 0 or 1, got {value}"
        ))
    }
}

fn validate_top_graph_degree_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_TOP_GRAPH_DEGREE..=EC_SPIRE_MAX_TOP_GRAPH_DEGREE).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire top_graph_degree reloption must be between {EC_SPIRE_MIN_TOP_GRAPH_DEGREE} and {EC_SPIRE_MAX_TOP_GRAPH_DEGREE}, got {value}"
        ))
    }
}

fn validate_top_graph_build_list_size_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE..=EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE)
        .contains(&value)
    {
        Ok(())
    } else {
        Err(format!(
            "ec_spire top_graph_build_list_size reloption must be between {EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE} and {EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE}, got {value}"
        ))
    }
}

fn validate_top_graph_alpha_value(value: f32) -> Result<(), String> {
    if value.is_finite()
        && (EC_SPIRE_MIN_TOP_GRAPH_ALPHA..=EC_SPIRE_MAX_TOP_GRAPH_ALPHA).contains(&value)
    {
        Ok(())
    } else {
        Err(format!(
            "ec_spire top_graph_alpha reloption must be finite and between {EC_SPIRE_MIN_TOP_GRAPH_ALPHA} and {EC_SPIRE_MAX_TOP_GRAPH_ALPHA}, got {value}"
        ))
    }
}

fn validate_top_graph_search_list_size_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE..=EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE)
        .contains(&value)
    {
        Ok(())
    } else {
        Err(format!(
            "ec_spire top_graph_search_list_size reloption must be between {EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE} and {EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE}, got {value}"
        ))
    }
}

fn normalize_local_store_tablespaces_reloption(
    value: &str,
    local_store_count: i32,
) -> Result<String, String> {
    validate_local_store_count_value(local_store_count)?;
    let names = value.split(',').map(str::trim).collect::<Vec<_>>();
    if names.iter().any(|name| name.is_empty()) {
        return Err(
            "ec_spire local_store_tablespaces reloption must not contain empty names".to_owned(),
        );
    }
    let expected_count = usize::try_from(local_store_count)
        .map_err(|_| "ec_spire local_store_count must be non-negative".to_owned())?;
    if names.len() != expected_count {
        return Err(format!(
            "ec_spire local_store_tablespaces reloption must name exactly {expected_count} tablespace(s), got {}",
            names.len()
        ));
    }
    Ok(names.join(","))
}

fn parse_nprobe_per_level_reloption(value: &str) -> Result<Vec<u32>, String> {
    let levels = value.split(',').map(str::trim).collect::<Vec<_>>();
    if levels.is_empty() || levels.iter().any(|level| level.is_empty()) {
        return Err(
            "ec_spire nprobe_per_level reloption must not contain empty entries".to_owned(),
        );
    }
    if levels.len() > EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES {
        return Err(format!(
            "ec_spire nprobe_per_level supports at most {EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES} entries"
        ));
    }
    levels
        .into_iter()
        .map(|level| {
            let parsed = level.parse::<u32>().map_err(|_| {
                format!(
                    "ec_spire nprobe_per_level reloption entries must be positive integers, got '{level}'"
                )
            })?;
            if parsed == 0 || parsed > EC_SPIRE_MAX_NPROBE as u32 {
                return Err(format!(
                    "ec_spire nprobe_per_level entries must be between 1 and {EC_SPIRE_MAX_NPROBE}, got {parsed}"
                ));
            }
            Ok(parsed)
        })
        .collect()
}

pub(super) fn plan_local_store_tablespaces_with_resolver(
    local_store_count: i32,
    index_tablespace_oid: u32,
    local_store_tablespaces: Option<&str>,
    mut resolve_tablespace: impl FnMut(&str) -> Result<u32, String>,
) -> Result<Vec<SpireLocalStoreTablespacePlanEntry>, String> {
    validate_local_store_count_value(local_store_count)?;
    let store_count = usize::try_from(local_store_count)
        .map_err(|_| "ec_spire local_store_count must be non-negative".to_owned())?;
    let tablespace_oids = if let Some(local_store_tablespaces) = local_store_tablespaces {
        let normalized = normalize_local_store_tablespaces_reloption(
            local_store_tablespaces,
            local_store_count,
        )?;
        normalized
            .split(',')
            .map(&mut resolve_tablespace)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![index_tablespace_oid; store_count]
    };
    tablespace_oids
        .into_iter()
        .enumerate()
        .map(|(index, tablespace_oid)| {
            let local_store_id = u32::try_from(index)
                .map_err(|_| "ec_spire local_store_id exceeds u32".to_owned())?;
            Ok(SpireLocalStoreTablespacePlanEntry {
                local_store_id,
                tablespace_oid,
            })
        })
        .collect()
}

pub(super) unsafe fn resolve_local_store_tablespace_plan(
    index_relation: pg_sys::Relation,
    options: &EcSpireOptions,
) -> Result<Vec<SpireLocalStoreTablespacePlanEntry>, String> {
    if index_relation.is_null() {
        return Err("ec_spire local store tablespace plan needs a valid index relation".to_owned());
    }
    let index_tablespace_oid = unsafe { (*(*index_relation).rd_rel).reltablespace }.into();
    plan_local_store_tablespaces_with_resolver(
        options.local_store_count,
        index_tablespace_oid,
        options.local_store_tablespaces.as_deref(),
        |name| unsafe { resolve_tablespace_name(name) },
    )
}

unsafe fn resolve_tablespace_name(name: &str) -> Result<u32, String> {
    let c_name = CString::new(name)
        .map_err(|_| "ec_spire local_store_tablespaces cannot contain NUL bytes".to_owned())?;
    let oid = unsafe { pg_sys::get_tablespace_oid(c_name.as_ptr(), true) };
    if oid == pg_sys::InvalidOid {
        return Err(format!(
            "ec_spire local_store_tablespaces names unknown tablespace '{name}'"
        ));
    }
    Ok(oid.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireNprobeResolution {
    pub(super) relation_nprobe: u32,
    pub(super) session_nprobe: Option<u32>,
    pub(super) effective_nprobe: u32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRerankWidthResolution {
    pub(super) relation_rerank_width: i32,
    pub(super) session_rerank_width: Option<i32>,
    pub(super) effective_rerank_width: i32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireCandidateLimitResolution {
    pub(super) relation_max_candidate_rows: i32,
    pub(super) session_max_candidate_rows: Option<i32>,
    pub(super) effective_max_candidate_rows: i32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireCandidateDedupeMode {
    NoReplicaDedupeDisabled,
    VecIdDedupeEnabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireSingleLevelScanPlan {
    pub(super) leaf_count: u32,
    pub(super) nprobe: u32,
    pub(super) nprobe_source: &'static str,
    pub(super) recursive_nprobe_policy: SpireRecursiveNprobePolicy,
    pub(super) recursive_route_budget: SpireRecursiveRouteBudget,
    pub(super) payload_format: SpireAssignmentPayloadFormat,
    pub(super) rerank_width: usize,
    pub(super) rerank_width_source: &'static str,
    pub(super) candidate_limit: Option<usize>,
    pub(super) dedupe_mode: SpireCandidateDedupeMode,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_spire.nprobe",
        c"Session override for ec_spire leaf PID probe count.",
        c"Overrides ec_spire index nprobe reloption when set to 1 or higher; -1 uses the relation value.",
        &EC_SPIRE_NPROBE_GUC,
        EC_SPIRE_SESSION_NPROBE_UNSET,
        EC_SPIRE_MAX_NPROBE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.rerank_width",
        c"Session override for ec_spire exact-rerank frontier width.",
        c"Overrides ec_spire index rerank_width reloption when set to 0 or higher; -1 uses the relation value.",
        &EC_SPIRE_RERANK_WIDTH_GUC,
        EC_SPIRE_SESSION_RERANK_WIDTH_UNSET,
        EC_SPIRE_MAX_RERANK_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.max_candidate_rows",
        c"Session override for ec_spire quantized candidate row budget.",
        c"Overrides ec_spire index max_candidate_rows reloption when set to 0 or higher; 0 uses the hard automatic ceiling; -1 uses the relation value.",
        &EC_SPIRE_MAX_CANDIDATE_ROWS_GUC,
        EC_SPIRE_SESSION_MAX_CANDIDATE_ROWS_UNSET,
        EC_SPIRE_MAX_MAX_CANDIDATE_ROWS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_spire.adaptive_nprobe",
        c"Enable deterministic adaptive ec_spire nprobe routing.",
        c"Diagnostic Phase 9.7 mode; when enabled, routing may reduce per-query nprobe by half when the selected centroid frontier is separated by the configured score-gap threshold.",
        &EC_SPIRE_ADAPTIVE_NPROBE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.adaptive_nprobe_score_gap_micros",
        c"Score-gap threshold for ec_spire adaptive nprobe.",
        c"Inner-product score gap, multiplied by 1,000,000, required between the retained adaptive frontier and the next centroid before adaptive nprobe reduces routing breadth.",
        &EC_SPIRE_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC,
        0,
        EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_max_nodes",
        c"Session cap for ec_spire remote-search libpq node fanout.",
        c"Maximum ready remote nodes admitted to a single libpq remote-search dispatch; 0 disables this cap.",
        &EC_SPIRE_REMOTE_SEARCH_MAX_NODES_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_max_pids",
        c"Session cap for ec_spire remote-search libpq PID fanout.",
        c"Maximum remote leaf PIDs admitted to a single libpq remote-search dispatch; 0 disables this cap.",
        &EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_max_pids_per_node",
        c"Session cap for ec_spire remote-search libpq PIDs per node.",
        c"Maximum remote leaf PIDs admitted for one ready remote node in a single libpq remote-search dispatch; 0 disables this cap.",
        &EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_PER_NODE_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_max_concurrent_dispatches",
        c"Session cap for concurrent ec_spire remote-search libpq dispatches.",
        c"Maximum concurrent remote-search libpq dispatches admitted across PostgreSQL backends on the coordinator; 0 disables this cap.",
        &EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_CONCURRENCY_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_max_concurrent_dispatches_per_node",
        c"Session cap for concurrent ec_spire remote-search libpq dispatches per remote node.",
        c"Maximum concurrent remote-search libpq dispatches admitted for one remote node across PostgreSQL backends on the coordinator; 0 disables this cap.",
        &EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_PER_NODE_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_CONCURRENCY_LIMIT,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_connect_timeout_ms",
        c"Session connect-timeout budget for ec_spire remote-search libpq work.",
        c"Connect timeout in milliseconds for production remote-search libpq execution; 0 leaves the conninfo/provider default unchanged.",
        &EC_SPIRE_REMOTE_SEARCH_CONNECT_TIMEOUT_MS_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_TIMEOUT_MS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_statement_timeout_ms",
        c"Session statement-timeout budget for ec_spire remote-search libpq work.",
        c"Remote statement timeout in milliseconds for production remote-search libpq execution; 0 leaves the remote session default unchanged.",
        &EC_SPIRE_REMOTE_SEARCH_STATEMENT_TIMEOUT_MS_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_TIMEOUT_MS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.max_remote_payload_bytes_per_row",
        c"Session cap for one ec_spire remote typed tuple payload row.",
        c"Maximum decoded tuple-payload bytes accepted from one remote heap row before strict mode fails closed; default 1024 comes from packet 30975 payload measurements plus a rounded 4x safety margin.",
        &EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW_GUC,
        1,
        EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.max_remote_payload_rows_per_batch",
        c"Session cap for one ec_spire remote payload batch.",
        c"Maximum selected PIDs or returned remote heap rows accepted for one remote payload batch before strict mode fails closed; default 64 matches the Phase 12 local capacity target.",
        &EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH_GUC,
        1,
        EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_routing_dimension_scale",
        c"SPIRE routing-score dimension cost scale.",
        c"Multiplier applied to vector dimensions when estimating routing-level scoring CPU; default 0.01 preserves packet 30976 cost calibration.",
        &EC_SPIRE_COST_ROUTING_DIMENSION_SCALE_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_leaf_dimension_scale",
        c"SPIRE leaf-score dimension cost scale.",
        c"Multiplier applied to vector dimensions when estimating leaf candidate scoring CPU; default 0.01 preserves packet 30976 cost calibration.",
        &EC_SPIRE_COST_LEAF_DIMENSION_SCALE_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_index_page_scale",
        c"SPIRE index page cost scale.",
        c"Multiplier applied to seq_page_cost for SPIRE index page reads; default 1.0 preserves packet 30976 cost calibration.",
        &EC_SPIRE_COST_INDEX_PAGE_SCALE_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_local_store_page_fanout_scale",
        c"SPIRE local-store page fanout cost scale.",
        c"Additional index page cost multiplier per local store beyond the first; default 0.05 preserves packet 30976 cost calibration.",
        &EC_SPIRE_COST_LOCAL_STORE_PAGE_FANOUT_SCALE_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_storage_scoring_multiplier",
        c"SPIRE storage-format scoring cost multiplier.",
        c"Session multiplier applied on top of the calibrated storage-format scoring baseline; default 1.0 preserves packet 30976 modeled-cost rows.",
        &EC_SPIRE_COST_STORAGE_SCORING_MULTIPLIER_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.cost_rerank_multiplier",
        c"SPIRE full-frontier rerank cost multiplier.",
        c"Candidate CPU multiplier used when effective rerank_width is 0; default 1.35 preserves packet 30976 cost calibration.",
        &EC_SPIRE_COST_RERANK_MULTIPLIER_GUC,
        0.0,
        EC_SPIRE_MAX_COST_SCALE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_enum_guc(
        c"ec_spire.remote_search_consistency_mode",
        c"Session consistency policy for production ec_spire remote search.",
        c"Single per-query production remote-search consistency policy. Values: strict fails closed on required remote failures; degraded skips failed remotes and reports partial-result diagnostics.",
        &EC_SPIRE_REMOTE_SEARCH_CONSISTENCY_MODE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_enum_guc(
        c"ec_spire.remote_tuple_transport",
        c"Session tuple-payload transport policy for production ec_spire remote search.",
        c"Controls coordinator selection of remote tuple-payload transport. Values: auto uses the remote endpoint default, json_tuple_payload_v1 is a legacy compatibility value that no longer enables production payload dispatch, pg_binary_attr_v1 forces typed binary payloads when the remote advertises that ready capability.",
        &EC_SPIRE_REMOTE_TUPLE_TRANSPORT_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    #[cfg(any(test, feature = "pg_test"))]
    GucRegistry::define_int_guc(
        c"ec_spire.remote_search_governance_test_namespace",
        c"Test-only namespace for ec_spire remote-search governance advisory locks.",
        c"Only available in pg_test builds; isolates parallel governance tests from cluster-wide advisory-lock keys.",
        &EC_SPIRE_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_nprobe() -> i32 {
    EC_SPIRE_NPROBE_GUC.get()
}

pub(super) fn current_session_rerank_width() -> i32 {
    EC_SPIRE_RERANK_WIDTH_GUC.get()
}

pub(super) fn current_session_max_candidate_rows() -> i32 {
    EC_SPIRE_MAX_CANDIDATE_ROWS_GUC.get()
}

pub(super) fn current_session_adaptive_nprobe() -> bool {
    EC_SPIRE_ADAPTIVE_NPROBE_GUC.get()
}

pub(super) fn current_session_adaptive_nprobe_score_gap_micros() -> i32 {
    EC_SPIRE_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC.get()
}

pub(super) fn current_session_remote_search_max_nodes() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_MAX_NODES_GUC.get()
}

pub(super) fn current_session_remote_search_max_pids() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_GUC.get()
}

pub(super) fn current_session_remote_search_max_pids_per_node() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_PER_NODE_GUC.get()
}

pub(super) fn current_session_remote_search_max_concurrent_dispatches() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_GUC.get()
}

pub(super) fn current_session_remote_search_max_concurrent_dispatches_per_node() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_PER_NODE_GUC.get()
}

pub(super) fn current_session_remote_search_connect_timeout_ms() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_CONNECT_TIMEOUT_MS_GUC.get()
}

pub(super) fn current_session_remote_search_statement_timeout_ms() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_STATEMENT_TIMEOUT_MS_GUC.get()
}

pub(super) fn current_session_max_remote_payload_bytes_per_row() -> i32 {
    EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW_GUC.get()
}

pub(super) fn current_session_max_remote_payload_rows_per_batch() -> i32 {
    EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH_GUC.get()
}

pub(super) fn current_session_cost_routing_dimension_scale() -> f64 {
    EC_SPIRE_COST_ROUTING_DIMENSION_SCALE_GUC.get()
}

pub(super) fn current_session_cost_leaf_dimension_scale() -> f64 {
    EC_SPIRE_COST_LEAF_DIMENSION_SCALE_GUC.get()
}

pub(super) fn current_session_cost_index_page_scale() -> f64 {
    EC_SPIRE_COST_INDEX_PAGE_SCALE_GUC.get()
}

pub(super) fn current_session_cost_local_store_page_fanout_scale() -> f64 {
    EC_SPIRE_COST_LOCAL_STORE_PAGE_FANOUT_SCALE_GUC.get()
}

pub(super) fn current_session_cost_storage_scoring_multiplier() -> f64 {
    EC_SPIRE_COST_STORAGE_SCORING_MULTIPLIER_GUC.get()
}

pub(super) fn current_session_cost_rerank_multiplier() -> f64 {
    EC_SPIRE_COST_RERANK_MULTIPLIER_GUC.get()
}

pub(super) fn current_session_remote_search_consistency_mode_name() -> &'static str {
    EC_SPIRE_REMOTE_SEARCH_CONSISTENCY_MODE_GUC.get().as_str()
}

pub(super) fn current_session_remote_tuple_transport() -> SpireRemoteTupleTransportGuc {
    EC_SPIRE_REMOTE_TUPLE_TRANSPORT_GUC.get()
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn current_session_remote_search_governance_test_namespace() -> i32 {
    EC_SPIRE_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE_GUC.get()
}

#[cfg(not(any(test, feature = "pg_test")))]
pub(super) fn current_session_remote_search_governance_test_namespace() -> i32 {
    0
}

pub(super) fn resolve_scan_nprobe(nlists: u32, relation_nprobe: u32) -> SpireNprobeResolution {
    resolve_scan_nprobe_values(nlists, relation_nprobe, current_session_nprobe())
}

pub(super) fn resolve_scan_rerank_width(relation_rerank_width: i32) -> SpireRerankWidthResolution {
    resolve_scan_rerank_width_values(relation_rerank_width, current_session_rerank_width())
}

pub(super) fn resolve_scan_max_candidate_rows(
    relation_max_candidate_rows: i32,
) -> SpireCandidateLimitResolution {
    resolve_scan_max_candidate_rows_values(
        relation_max_candidate_rows,
        current_session_max_candidate_rows(),
    )
}

pub(super) fn resolve_single_level_scan_plan(
    leaf_count: u32,
    options: EcSpireOptions,
) -> Result<SpireSingleLevelScanPlan, String> {
    resolve_single_level_scan_plan_values_with_candidate_budget_and_adaptive(
        leaf_count,
        options,
        current_session_nprobe(),
        current_session_rerank_width(),
        current_session_max_candidate_rows(),
        current_session_adaptive_nprobe(),
        current_session_adaptive_nprobe_score_gap_micros(),
    )
}

pub(super) fn resolve_single_level_scan_plan_values(
    leaf_count: u32,
    options: EcSpireOptions,
    session_nprobe_value: i32,
    session_rerank_width_value: i32,
) -> Result<SpireSingleLevelScanPlan, String> {
    resolve_single_level_scan_plan_values_with_candidate_budget(
        leaf_count,
        options,
        session_nprobe_value,
        session_rerank_width_value,
        EC_SPIRE_SESSION_MAX_CANDIDATE_ROWS_UNSET,
    )
}

pub(super) fn resolve_single_level_scan_plan_values_with_candidate_budget(
    leaf_count: u32,
    options: EcSpireOptions,
    session_nprobe_value: i32,
    session_rerank_width_value: i32,
    session_max_candidate_rows_value: i32,
) -> Result<SpireSingleLevelScanPlan, String> {
    resolve_single_level_scan_plan_values_with_candidate_budget_and_adaptive(
        leaf_count,
        options,
        session_nprobe_value,
        session_rerank_width_value,
        session_max_candidate_rows_value,
        false,
        EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS,
    )
}

fn resolve_single_level_scan_plan_values_with_candidate_budget_and_adaptive(
    leaf_count: u32,
    options: EcSpireOptions,
    session_nprobe_value: i32,
    session_rerank_width_value: i32,
    session_max_candidate_rows_value: i32,
    adaptive_nprobe: bool,
    adaptive_score_gap_micros: i32,
) -> Result<SpireSingleLevelScanPlan, String> {
    let relation_nprobe = u32::try_from(options.nprobe)
        .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
    if options.rerank_width < 0 {
        return Err("ec_spire rerank_width reloption must be non-negative".to_owned());
    }
    validate_max_candidate_rows_value(options.max_candidate_rows)?;

    let nprobe = resolve_scan_nprobe_values(leaf_count, relation_nprobe, session_nprobe_value);
    let recursive_nprobe_policy = SpireRecursiveNprobePolicy::from_level_values_with_adaptive(
        nprobe.effective_nprobe,
        options.nprobe_per_level_values(),
        adaptive_nprobe,
        adaptive_score_gap_micros,
    )?;
    let recursive_route_budget =
        resolve_recursive_route_budget(leaf_count, nprobe.effective_nprobe)?;
    let rerank_width =
        resolve_scan_rerank_width_values(options.rerank_width, session_rerank_width_value);
    let rerank_width_usize = usize::try_from(rerank_width.effective_rerank_width)
        .map_err(|_| "ec_spire rerank_width exceeds usize".to_owned())?;
    let max_candidate_rows = resolve_scan_max_candidate_rows_values(
        options.max_candidate_rows,
        session_max_candidate_rows_value,
    );
    let max_candidate_rows_usize = usize::try_from(max_candidate_rows.effective_max_candidate_rows)
        .map_err(|_| "ec_spire max_candidate_rows exceeds usize".to_owned())?;
    let candidate_limit = if rerank_width_usize > 0 {
        Some(rerank_width_usize.min(max_candidate_rows_usize))
    } else {
        Some(max_candidate_rows_usize)
    };

    Ok(SpireSingleLevelScanPlan {
        leaf_count,
        nprobe: nprobe.effective_nprobe,
        nprobe_source: nprobe.source,
        recursive_nprobe_policy,
        recursive_route_budget,
        payload_format: options.assignment_payload_format(),
        rerank_width: rerank_width_usize,
        rerank_width_source: rerank_width.source,
        candidate_limit,
        dedupe_mode: if options.boundary_replica_count > 0 {
            SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
    })
}

pub(super) fn resolve_recursive_route_budget(
    leaf_count: u32,
    effective_nprobe: u32,
) -> Result<SpireRecursiveRouteBudget, String> {
    if leaf_count == 0 || effective_nprobe == 0 {
        return Ok(SpireRecursiveRouteBudget {
            beam_width: 0,
            max_leaf_routes: 0,
            max_routing_expansions: 0,
        });
    }
    let beam_width = usize::try_from(effective_nprobe)
        .map_err(|_| "ec_spire recursive beam width exceeds usize".to_owned())?;
    let max_leaf_routes = usize::try_from(effective_nprobe)
        .map_err(|_| "ec_spire recursive max leaf routes exceeds usize".to_owned())?;
    let leaf_count_usize = usize::try_from(leaf_count)
        .map_err(|_| "ec_spire recursive leaf count exceeds usize".to_owned())?;
    // `nprobe_per_level` remains a local per-parent exploration input. Until a
    // separate beam reloption lands, the leaf-level effective nprobe is the
    // final global cap for routed internal parents and leaf routes.
    Ok(SpireRecursiveRouteBudget {
        beam_width,
        max_leaf_routes: max_leaf_routes.min(leaf_count_usize),
        max_routing_expansions: leaf_count_usize.max(beam_width),
    })
}

fn resolve_scan_nprobe_values(
    nlists: u32,
    relation_nprobe: u32,
    session_nprobe_value: i32,
) -> SpireNprobeResolution {
    let session_nprobe = match session_nprobe_value {
        value if value > 0 => Some(value as u32),
        _ => None,
    };
    if nlists == 0 {
        return SpireNprobeResolution {
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

    SpireNprobeResolution {
        relation_nprobe,
        session_nprobe,
        effective_nprobe: requested.clamp(1, nlists),
        source,
    }
}

fn resolve_scan_rerank_width_values(
    relation_rerank_width: i32,
    session_rerank_width_value: i32,
) -> SpireRerankWidthResolution {
    let session_rerank_width = match session_rerank_width_value {
        value if value >= 0 => Some(value),
        _ => None,
    };
    let (effective_rerank_width, source) = match session_rerank_width {
        Some(value) => (value.clamp(0, EC_SPIRE_MAX_RERANK_WIDTH), "session"),
        None => (relation_rerank_width, "relation"),
    };

    SpireRerankWidthResolution {
        relation_rerank_width,
        session_rerank_width,
        effective_rerank_width,
        source,
    }
}

fn resolve_scan_max_candidate_rows_values(
    relation_max_candidate_rows: i32,
    session_max_candidate_rows_value: i32,
) -> SpireCandidateLimitResolution {
    let session_max_candidate_rows = match session_max_candidate_rows_value {
        value if value >= 0 => Some(value),
        _ => None,
    };
    let (configured, configured_source) = match session_max_candidate_rows {
        Some(value) => (value, "session"),
        None => (relation_max_candidate_rows, "relation"),
    };
    let (effective_max_candidate_rows, source) = if configured == 0 {
        (EC_SPIRE_MAX_MAX_CANDIDATE_ROWS, "auto")
    } else {
        (
            configured.clamp(1, EC_SPIRE_MAX_MAX_CANDIDATE_ROWS),
            configured_source,
        )
    };

    SpireCandidateLimitResolution {
        relation_max_candidate_rows,
        session_max_candidate_rows,
        effective_max_candidate_rows,
        source,
    }
}

fn auto_nprobe(nlists: u32) -> u32 {
    if nlists == 0 {
        return 0;
    }
    (nlists as f64).sqrt().ceil() as u32
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<EcSpireReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nlists".as_ptr(),
                c"Number of single-level SPIRE-IVF leaf PIDs; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_NLISTS,
                EC_SPIRE_MIN_NLISTS,
                EC_SPIRE_MAX_NLISTS,
                offset_of!(EcSpireReloptions, nlists) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"recursive_fanout".as_ptr(),
                c"Opt-in recursive SPIRE routing fanout; 0 keeps single-level build behavior, values must be at least 2."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_RECURSIVE_FANOUT,
                EC_SPIRE_MIN_RECURSIVE_FANOUT,
                EC_SPIRE_MAX_RECURSIVE_FANOUT,
                offset_of!(EcSpireReloptions, recursive_fanout) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"local_store_count".as_ptr(),
                c"Number of local SPIRE partition-store relations to plan for; 1 keeps embedded single-store behavior."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
                EC_SPIRE_MIN_LOCAL_STORE_COUNT,
                EC_SPIRE_MAX_LOCAL_STORE_COUNT,
                offset_of!(EcSpireReloptions, local_store_count) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"boundary_replica_count".as_ptr(),
                c"Maximum secondary SPIRE leaf assignments per vector; 0 keeps primary-only assignment."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT,
                EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT,
                EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT,
                offset_of!(EcSpireReloptions, boundary_replica_count) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nprobe".as_ptr(),
                c"Number of SPIRE leaf PIDs to probe during scan; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_NPROBE,
                EC_SPIRE_MIN_NPROBE,
                EC_SPIRE_MAX_NPROBE,
                offset_of!(EcSpireReloptions, nprobe) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"rerank_width".as_ptr(),
                c"Number of quantized candidates to exact-rerank; 0 reranks the full candidate frontier."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_RERANK_WIDTH,
                EC_SPIRE_MIN_RERANK_WIDTH,
                EC_SPIRE_MAX_RERANK_WIDTH,
                offset_of!(EcSpireReloptions, rerank_width) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"max_candidate_rows".as_ptr(),
                c"Hard cap on quantized candidate rows retained before exact rerank; 0 uses the automatic ceiling."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_MAX_CANDIDATE_ROWS,
                EC_SPIRE_MIN_MAX_CANDIDATE_ROWS,
                EC_SPIRE_MAX_MAX_CANDIDATE_ROWS,
                offset_of!(EcSpireReloptions, max_candidate_rows) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"training_sample_rows".as_ptr(),
                c"Maximum rows sampled for SPIRE centroid training; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS,
                EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS,
                EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS,
                offset_of!(EcSpireReloptions, training_sample_rows) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"seed".as_ptr(),
                c"Deterministic seed for SPIRE centroid training and quantizer defaults.".as_ptr(),
                EC_SPIRE_DEFAULT_SEED,
                EC_SPIRE_MIN_SEED,
                EC_SPIRE_MAX_SEED,
                offset_of!(EcSpireReloptions, seed) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"pq_group_size".as_ptr(),
                c"Grouped-PQ subvector size for storage_format = 'pq_fastscan'; 0 chooses the default."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_PQ_GROUP_SIZE,
                EC_SPIRE_MIN_PQ_GROUP_SIZE,
                EC_SPIRE_MAX_PQ_GROUP_SIZE,
                offset_of!(EcSpireReloptions, pq_group_size) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"top_graph_enabled".as_ptr(),
                c"Enable SPIRE top-graph build/scan plumbing; 0 keeps flat recursive routing."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_TOP_GRAPH_ENABLED,
                EC_SPIRE_MIN_TOP_GRAPH_ENABLED,
                EC_SPIRE_MAX_TOP_GRAPH_ENABLED,
                offset_of!(EcSpireReloptions, top_graph_enabled) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"top_graph_degree".as_ptr(),
                c"Maximum Vamana out-degree for the SPIRE top graph.".as_ptr(),
                EC_SPIRE_DEFAULT_TOP_GRAPH_DEGREE,
                EC_SPIRE_MIN_TOP_GRAPH_DEGREE,
                EC_SPIRE_MAX_TOP_GRAPH_DEGREE,
                offset_of!(EcSpireReloptions, top_graph_degree) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"top_graph_build_list_size".as_ptr(),
                c"Vamana build search-list size for the SPIRE top graph.".as_ptr(),
                EC_SPIRE_DEFAULT_TOP_GRAPH_BUILD_LIST_SIZE,
                EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE,
                EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE,
                offset_of!(EcSpireReloptions, top_graph_build_list_size) as i32,
            );
            pg_sys::add_local_real_reloption(
                &mut relopts,
                c"top_graph_alpha".as_ptr(),
                c"Vamana alpha-pruning slack for the SPIRE top graph.".as_ptr(),
                EC_SPIRE_DEFAULT_TOP_GRAPH_ALPHA as f64,
                EC_SPIRE_MIN_TOP_GRAPH_ALPHA as f64,
                EC_SPIRE_MAX_TOP_GRAPH_ALPHA as f64,
                offset_of!(EcSpireReloptions, top_graph_alpha) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"top_graph_search_list_size".as_ptr(),
                c"Vamana scan search-list size for the SPIRE top graph; 0 derives from nprobe."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_TOP_GRAPH_SEARCH_LIST_SIZE,
                EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE,
                EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE,
                offset_of!(EcSpireReloptions, top_graph_search_list_size) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"nprobe_per_level".as_ptr(),
                c"Comma-separated recursive SPIRE nprobe values for levels above 1, ordered from level 2 upward; omitted levels use the conservative policy."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, nprobe_per_level_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"storage_format".as_ptr(),
                c"SPIRE assignment payload quantizer profile: 'turboquant', 'pq_fastscan', 'rabitq', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, storage_format_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"quantizer".as_ptr(),
                c"Alias for storage_format: SPIRE assignment payload quantizer profile 'turboquant', 'pq_fastscan', 'rabitq', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, quantizer_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"source_identity".as_ptr(),
                c"Stable SPIRE vector identity provider: 'include' reads one INCLUDE column as a UUID or exact-16-byte bytea source identity."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, source_identity_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"local_store_tablespaces".as_ptr(),
                c"Comma-separated tablespace names for local SPIRE stores; repeated names are allowed for same-device baselines."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, local_store_tablespaces_offset) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}

unsafe fn read_string_reloption(
    rd_options: *mut pg_sys::varlena,
    offset: i32,
    name: &str,
) -> Option<String> {
    if offset == 0 {
        return None;
    }

    let value_ptr = unsafe {
        rd_options
            .cast::<u8>()
            .add(offset as usize)
            .cast::<std::ffi::c_char>()
    };
    let value = unsafe { std::ffi::CStr::from_ptr(value_ptr) }
        .to_str()
        .unwrap_or_else(|e| pgrx::error!("invalid ec_spire {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid ec_spire {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> EcSpireOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return EcSpireOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<EcSpireReloptions>() };
    validate_recursive_fanout_value(reloptions.recursive_fanout)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_local_store_count_value(reloptions.local_store_count)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_boundary_replica_count_value(reloptions.boundary_replica_count)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_max_candidate_rows_value(reloptions.max_candidate_rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_top_graph_enabled_value(reloptions.top_graph_enabled)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_top_graph_degree_value(reloptions.top_graph_degree)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_top_graph_build_list_size_value(reloptions.top_graph_build_list_size)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_top_graph_alpha_value(reloptions.top_graph_alpha as f32)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_top_graph_search_list_size_value(reloptions.top_graph_search_list_size)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let storage_format_reloption = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.storage_format_offset,
            "storage_format",
        )
    };
    let quantizer_reloption =
        unsafe { read_string_reloption(rd_options, reloptions.quantizer_offset, "quantizer") };
    if let (Some(storage_format), Some(quantizer)) =
        (&storage_format_reloption, &quantizer_reloption)
    {
        if storage_format != quantizer {
            pgrx::error!(
                "ec_spire storage_format and quantizer reloptions conflict: storage_format = '{}', quantizer = '{}'",
                storage_format,
                quantizer
            );
        }
    }
    let storage_format = storage_format_reloption
        .or(quantizer_reloption)
        .map(|value| {
            SpireStorageFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        })
        .unwrap_or(SpireStorageFormat::Auto);
    let source_identity = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.source_identity_offset,
            "source_identity",
        )
    }
    .map(|value| {
        SpireSourceIdentityProvider::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
    })
    .unwrap_or(SpireSourceIdentityProvider::None);
    let local_store_tablespaces = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.local_store_tablespaces_offset,
            "local_store_tablespaces",
        )
    }
    .map(|value| {
        normalize_local_store_tablespaces_reloption(&value, reloptions.local_store_count)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
    });
    let nprobe_per_level = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.nprobe_per_level_offset,
            "nprobe_per_level",
        )
    }
    .map(|value| parse_nprobe_per_level_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}")));

    EcSpireOptions {
        nlists: reloptions.nlists,
        recursive_fanout: reloptions.recursive_fanout,
        local_store_count: reloptions.local_store_count,
        boundary_replica_count: reloptions.boundary_replica_count,
        nprobe: reloptions.nprobe,
        rerank_width: reloptions.rerank_width,
        max_candidate_rows: reloptions.max_candidate_rows,
        training_sample_rows: reloptions.training_sample_rows,
        seed: reloptions.seed,
        pq_group_size: reloptions.pq_group_size,
        top_graph_enabled: reloptions.top_graph_enabled,
        top_graph_degree: reloptions.top_graph_degree,
        top_graph_build_list_size: reloptions.top_graph_build_list_size,
        top_graph_alpha: reloptions.top_graph_alpha as f32,
        top_graph_search_list_size: reloptions.top_graph_search_list_size,
        nprobe_per_level,
        storage_format,
        source_identity,
        local_store_tablespaces,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_local_store_tablespaces_reloption, parse_nprobe_per_level_reloption,
        plan_local_store_tablespaces_with_resolver, resolve_recursive_route_budget,
        resolve_scan_max_candidate_rows_values, resolve_scan_nprobe_values,
        resolve_scan_rerank_width_values, resolve_single_level_scan_plan_values,
        resolve_single_level_scan_plan_values_with_candidate_budget,
        validate_boundary_replica_count_value, validate_local_store_count_value,
        validate_max_candidate_rows_value, validate_recursive_fanout_value, EcSpireOptions,
        SpireCandidateDedupeMode, SpireRecursiveRouteBudget, SpireSourceIdentityProvider,
        SpireStorageFormat, SpireTopGraphOptionPlan, EC_SPIRE_MAX_MAX_CANDIDATE_ROWS,
    };
    use crate::am::ec_spire::quantizer::SpireAssignmentPayloadFormat;

    #[test]
    fn storage_format_reloption_parses_and_maps_to_assignment_payload_format() {
        assert_eq!(
            SpireStorageFormat::parse_reloption("auto").unwrap(),
            SpireStorageFormat::Auto
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("turboquant").unwrap(),
            SpireStorageFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("pq_fastscan").unwrap(),
            SpireStorageFormat::PqFastScan
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("rabitq").unwrap(),
            SpireStorageFormat::RaBitQ
        );
        assert!(SpireStorageFormat::parse_reloption("bad").is_err());

        assert_eq!(
            SpireStorageFormat::Auto.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::RaBitQ.assignment_payload_format(),
            SpireAssignmentPayloadFormat::RaBitQ
        );
    }

    #[test]
    fn source_identity_reloption_parses_provider() {
        assert_eq!(
            SpireSourceIdentityProvider::parse_reloption("include").unwrap(),
            SpireSourceIdentityProvider::Include
        );
        assert_eq!(
            SpireSourceIdentityProvider::Include.reloption_name(),
            "include"
        );
        assert!(SpireSourceIdentityProvider::parse_reloption("uuid").is_err());
    }

    #[test]
    fn recursive_fanout_validation_rejects_one() {
        assert!(validate_recursive_fanout_value(0).is_ok());
        assert!(validate_recursive_fanout_value(2).is_ok());
        assert!(validate_recursive_fanout_value(32).is_ok());
        assert!(validate_recursive_fanout_value(1).is_err());
    }

    #[test]
    fn local_store_count_validation_bounds_phase4_surface() {
        assert!(validate_local_store_count_value(1).is_ok());
        assert!(validate_local_store_count_value(16).is_ok());
        assert!(validate_local_store_count_value(0).is_err());
        assert!(validate_local_store_count_value(17).is_err());
    }

    #[test]
    fn boundary_replica_count_validation_bounds_phase5_surface() {
        assert!(validate_boundary_replica_count_value(0).is_ok());
        assert!(validate_boundary_replica_count_value(8).is_ok());
        assert!(validate_boundary_replica_count_value(-1).is_err());
        assert!(validate_boundary_replica_count_value(9).is_err());
    }

    #[test]
    fn local_store_tablespaces_normalizes_and_allows_repeated_names() {
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a, fast_a", 2).unwrap(),
            "fast_a,fast_a"
        );
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a", 1).unwrap(),
            "fast_a"
        );
        assert!(normalize_local_store_tablespaces_reloption("fast_a", 2).is_err());
        assert!(normalize_local_store_tablespaces_reloption("fast_a,", 2).is_err());
    }

    #[test]
    fn local_store_tablespace_plan_resolves_names_and_repeats() {
        let plan = plan_local_store_tablespaces_with_resolver(
            3,
            999,
            Some("fast_a,fast_a,fast_b"),
            |name| match name {
                "fast_a" => Ok(10),
                "fast_b" => Ok(11),
                other => Err(format!("unknown tablespace {other}")),
            },
        )
        .unwrap();

        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 10);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 10);
        assert_eq!(plan[2].local_store_id, 2);
        assert_eq!(plan[2].tablespace_oid, 11);
    }

    #[test]
    fn local_store_tablespace_plan_inherits_index_tablespace_by_default() {
        let plan =
            plan_local_store_tablespaces_with_resolver(2, 999, None, |_| unreachable!()).unwrap();

        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 999);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 999);
    }

    #[test]
    fn local_store_tablespace_plan_rejects_unknown_or_mismatched_names() {
        assert!(
            plan_local_store_tablespaces_with_resolver(2, 999, Some("fast_a"), |_| Ok(10)).is_err()
        );
        assert!(
            plan_local_store_tablespaces_with_resolver(1, 999, Some("missing"), |name| Err(
                format!("unknown tablespace {name}")
            ),)
            .is_err()
        );
    }

    #[test]
    fn scan_nprobe_resolution_uses_session_relation_and_auto_sources() {
        assert_eq!(resolve_scan_nprobe_values(0, 5, -1).effective_nprobe, 0);

        let auto = resolve_scan_nprobe_values(17, 0, -1);
        assert_eq!(auto.effective_nprobe, 5);
        assert_eq!(auto.source, "auto");

        let relation = resolve_scan_nprobe_values(17, 3, -1);
        assert_eq!(relation.effective_nprobe, 3);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_nprobe_values(17, 3, 99);
        assert_eq!(session.session_nprobe, Some(99));
        assert_eq!(session.effective_nprobe, 17);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn nprobe_per_level_reloption_parses_upper_level_values() {
        assert_eq!(
            parse_nprobe_per_level_reloption("2, 3").unwrap(),
            vec![2, 3]
        );
        assert!(parse_nprobe_per_level_reloption("0").is_err());
        assert!(parse_nprobe_per_level_reloption("2,").is_err());
        assert!(parse_nprobe_per_level_reloption("bad").is_err());
        assert!(parse_nprobe_per_level_reloption(&["1"; 33].join(",")).is_err());
    }

    #[test]
    fn scan_rerank_width_resolution_uses_session_or_relation() {
        let relation = resolve_scan_rerank_width_values(128, -1);
        assert_eq!(relation.effective_rerank_width, 128);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_rerank_width_values(128, 0);
        assert_eq!(session.session_rerank_width, Some(0));
        assert_eq!(session.effective_rerank_width, 0);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn scan_max_candidate_rows_resolution_uses_session_relation_and_auto_sources() {
        let auto = resolve_scan_max_candidate_rows_values(0, -1);
        assert_eq!(auto.effective_max_candidate_rows, 10_000_000);
        assert_eq!(auto.source, "auto");

        let relation = resolve_scan_max_candidate_rows_values(128, -1);
        assert_eq!(relation.effective_max_candidate_rows, 128);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_max_candidate_rows_values(128, 7);
        assert_eq!(session.session_max_candidate_rows, Some(7));
        assert_eq!(session.effective_max_candidate_rows, 7);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn default_options_match_phase1_config_contract() {
        let options = EcSpireOptions::DEFAULT;

        assert_eq!(options.nlists, 0);
        assert_eq!(options.recursive_fanout, 0);
        assert_eq!(options.recursive_fanout(), None);
        assert_eq!(options.local_store_count, 1);
        assert_eq!(options.boundary_replica_count, 0);
        assert_eq!(options.nprobe, 0);
        assert_eq!(options.rerank_width, 0);
        assert_eq!(options.max_candidate_rows, 0);
        assert_eq!(options.training_sample_rows, 0);
        assert_eq!(options.seed, 42);
        assert_eq!(options.requested_pq_group_size(), None);
        assert_eq!(
            options.top_graph_plan().unwrap(),
            SpireTopGraphOptionPlan {
                enabled: false,
                graph_degree: 32,
                build_list_size: 100,
                alpha: 1.2,
                search_list_size: None,
            }
        );
        assert_eq!(options.storage_format, SpireStorageFormat::Auto);
        assert_eq!(options.source_identity, SpireSourceIdentityProvider::None);
        assert_eq!(options.nprobe_per_level, None);
        assert_eq!(options.local_store_tablespaces, None);
        assert_eq!(
            options.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
    }

    #[test]
    fn single_level_scan_plan_resolves_runtime_knobs() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 4,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 3,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 1000,
            seed: 7,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::RaBitQ,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: Some("fast_a".to_owned()),
        };

        let plan = resolve_single_level_scan_plan_values(17, options.clone(), -1, -1).unwrap();

        assert_eq!(plan.leaf_count, 17);
        assert_eq!(plan.nprobe, 3);
        assert_eq!(plan.nprobe_source, "relation");
        assert_eq!(plan.payload_format, SpireAssignmentPayloadFormat::RaBitQ);
        assert_eq!(plan.rerank_width, 128);
        assert_eq!(plan.rerank_width_source, "relation");
        assert_eq!(plan.candidate_limit, Some(128));
        assert_eq!(
            plan.recursive_route_budget,
            SpireRecursiveRouteBudget {
                beam_width: 3,
                max_leaf_routes: 3,
                max_routing_expansions: 17,
            }
        );
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
        assert_eq!(options.recursive_fanout(), Some(4));
    }

    #[test]
    fn single_level_scan_plan_carries_recursive_per_level_nprobe_policy() {
        let options = EcSpireOptions {
            nprobe: 2,
            nprobe_per_level: Some(vec![3, 4]),
            ..EcSpireOptions::DEFAULT
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(plan.nprobe, 2);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(1), 2);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(2), 3);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(3), 4);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(4), 1);
        assert_eq!(plan.recursive_route_budget.beam_width, 2);
        assert_eq!(plan.recursive_route_budget.max_leaf_routes, 2);
        assert_eq!(plan.recursive_route_budget.max_routing_expansions, 17);
    }

    #[test]
    fn recursive_route_budget_resolves_finite_scan_guardrails() {
        assert_eq!(
            resolve_recursive_route_budget(100, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 7,
                max_leaf_routes: 7,
                max_routing_expansions: 100,
            }
        );
        assert_eq!(
            resolve_recursive_route_budget(3, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 7,
                max_leaf_routes: 3,
                max_routing_expansions: 7,
            }
        );
        assert_eq!(
            resolve_recursive_route_budget(0, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 0,
                max_leaf_routes: 0,
                max_routing_expansions: 0,
            }
        );
    }

    #[test]
    fn single_level_scan_plan_uses_session_overrides_and_full_rerank() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 0,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, 99, 0).unwrap();

        assert_eq!(plan.nprobe, 17);
        assert_eq!(plan.nprobe_source, "session");
        assert_eq!(
            plan.payload_format,
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(plan.rerank_width, 0);
        assert_eq!(plan.rerank_width_source, "session");
        assert_eq!(plan.candidate_limit, Some(10_000_000));
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
    }

    #[test]
    fn single_level_scan_plan_applies_hard_candidate_budget_to_full_rerank() {
        let options = EcSpireOptions {
            nlists: 17,
            nprobe: 0,
            rerank_width: 0,
            max_candidate_rows: 3,
            ..EcSpireOptions::DEFAULT
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(plan.rerank_width, 0);
        assert_eq!(plan.candidate_limit, Some(3));

        let options = EcSpireOptions {
            nlists: 17,
            nprobe: 0,
            rerank_width: 128,
            max_candidate_rows: 5,
            ..EcSpireOptions::DEFAULT
        };

        let plan =
            resolve_single_level_scan_plan_values_with_candidate_budget(17, options, -1, -1, 4)
                .unwrap();

        assert_eq!(plan.rerank_width, 128);
        assert_eq!(plan.candidate_limit, Some(4));
    }

    #[test]
    fn single_level_scan_plan_rejects_invalid_manual_options() {
        let invalid = EcSpireOptions {
            nlists: 0,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: -1,
            rerank_width: 0,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid.clone(), -1, -1).is_err());

        let invalid = EcSpireOptions {
            nprobe: 0,
            rerank_width: -1,
            ..invalid
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid, -1, -1).is_err());

        let invalid = EcSpireOptions {
            max_candidate_rows: -1,
            ..EcSpireOptions::DEFAULT
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid, -1, -1).is_err());
        assert!(validate_max_candidate_rows_value(EC_SPIRE_MAX_MAX_CANDIDATE_ROWS + 1).is_err());
    }

    #[test]
    fn single_level_scan_plan_enables_vec_id_dedupe_for_replica_capable_indexes() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 1,
            nprobe: 3,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::VecIdDedupeEnabled
        );
    }

    #[test]
    fn top_graph_option_plan_resolves_enabled_params_and_auto_search_list() {
        let options = EcSpireOptions {
            top_graph_enabled: 1,
            top_graph_degree: 64,
            top_graph_build_list_size: 200,
            top_graph_alpha: 1.4,
            top_graph_search_list_size: 0,
            ..EcSpireOptions::DEFAULT
        };

        assert_eq!(
            options.top_graph_plan().unwrap(),
            SpireTopGraphOptionPlan {
                enabled: true,
                graph_degree: 64,
                build_list_size: 200,
                alpha: 1.4,
                search_list_size: None,
            }
        );

        let explicit_search = EcSpireOptions {
            top_graph_search_list_size: 37,
            ..options
        };
        assert_eq!(
            explicit_search.top_graph_plan().unwrap().search_list_size,
            Some(37)
        );
    }

    #[test]
    fn top_graph_option_plan_rejects_invalid_values() {
        let invalid_enabled = EcSpireOptions {
            top_graph_enabled: 2,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_enabled.top_graph_plan().is_err());

        let invalid_degree = EcSpireOptions {
            top_graph_degree: 0,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_degree.top_graph_plan().is_err());

        let invalid_alpha = EcSpireOptions {
            top_graph_alpha: f32::NAN,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_alpha.top_graph_plan().is_err());
    }
}
