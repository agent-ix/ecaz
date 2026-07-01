use std::ffi::CString;
use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};

use crate::am::common::callback::pg_am_callback;

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
const EC_SPIRE_SESSION_MAX_ROUTED_CANDIDATE_ROWS_DISABLED: i32 = 0;
const EC_SPIRE_SESSION_LEAF_BLOCK_ROWS_DISABLED: i32 = 0;
const EC_SPIRE_DEFAULT_LEAF_BLOCK_SUMMARY_REPRESENTATIVES: i32 = 2;
const EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED: i32 = 0;
const EC_SPIRE_MAX_LEAF_BLOCK_ROWS: i32 = 4096;
const EC_SPIRE_MAX_LEAF_BLOCK_SUMMARY_REPRESENTATIVES: i32 = 8;
const EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_BLOCKS_PER_LEAF: i32 = 4096;
const EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_GLOBAL_BLOCKS: i32 = 262_144;
const EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_GLOBAL_PROBE_BLOCKS: i32 = 262_144;
const EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_SAMPLE_ROWS_PER_BLOCK: i32 = 64;
const EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT: f64 = 0.8;
const EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT: f64 = 1.0;
const EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT: f64 = 0.0;
const EC_SPIRE_MAX_NPROBE_PER_LEVEL_ENTRIES: usize = 32;
const EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1000;
const EC_SPIRE_MAX_ADAPTIVE_NPROBE_SCORE_GAP_MICROS: i32 = 1_000_000;
const EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET: i32 = 0;
const EC_SPIRE_MAX_REMOTE_SEARCH_LIMIT: i32 = 1_000_000;
const EC_SPIRE_MAX_REMOTE_SEARCH_CONCURRENCY_LIMIT: i32 = 4096;
const EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS: i32 = 0;
const EC_SPIRE_MAX_REMOTE_SEARCH_TIMEOUT_MS: i32 = 3_600_000;
const EC_SPIRE_DEFAULT_REMOTE_SEARCH_CONNECTION_POOL_SIZE: i32 = 16;
const EC_SPIRE_MAX_REMOTE_SEARCH_CONNECTION_POOL_SIZE: i32 = 1024;
const EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW: i32 = 1024;
const EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW: i32 = 1_073_741_824;
const EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH: i32 = 128;
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
static EC_SPIRE_MAX_ROUTED_CANDIDATE_ROWS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_MAX_ROUTED_CANDIDATE_ROWS_DISABLED);
static EC_SPIRE_CANDIDATE_BATCH_SCORING_GUC: GucSetting<bool> = GucSetting::<bool>::new(true);
static EC_SPIRE_PRE_MATERIALIZATION_PRUNE_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static EC_SPIRE_REMOTE_SEARCH_GLOBAL_PRE_HEAP_MERGE_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(false);
static EC_SPIRE_LEAF_BLOCK_ROWS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_LEAF_BLOCK_ROWS_DISABLED);
static EC_SPIRE_LEAF_BLOCK_SUMMARY_REPRESENTATIVES_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_LEAF_BLOCK_SUMMARY_REPRESENTATIVES);
static EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_BLOCKS_PER_LEAF_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED);
static EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_GLOBAL_BLOCKS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED);
static EC_SPIRE_LEAF_BLOCK_PRUNING_GLOBAL_PROBE_BLOCKS_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED);
static EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_ROWS_PER_BLOCK_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED);
static EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT);
static EC_SPIRE_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT);
static EC_SPIRE_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT_GUC: GucSetting<f64> =
    GucSetting::<f64>::new(EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT);
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
static EC_SPIRE_REMOTE_SEARCH_CONNECTION_POOL_SIZE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_DEFAULT_REMOTE_SEARCH_CONNECTION_POOL_SIZE);
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
            // Pq-FastScan parses and the index can be created empty, but it is a
            // permanent exclusion (Task 106 slice 4): SPIRE has no grouped-PQ
            // model persistence, so a populated build errors and the assignment
            // payload reports `deferred_model_metadata` / unscannable via the
            // options snapshot. Documented in ADR-077 §8 / aggregate matrix §4.
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

pub(super) fn resolve_local_store_tablespace_plan(
    index_relation: pg_sys::Relation,
    options: &EcSpireOptions,
) -> Result<Vec<SpireLocalStoreTablespacePlanEntry>, String> {
    if index_relation.is_null() {
        return Err("ec_spire local store tablespace plan needs a valid index relation".to_owned());
    }
    let index_relation = std::ptr::NonNull::new(index_relation).ok_or_else(|| {
        "ec_spire local store tablespace plan needs a valid index relation".to_owned()
    })?;
    let index_tablespace_oid =
        crate::storage::relation::relation_tablespace_handle(index_relation).into();
    plan_local_store_tablespaces_with_resolver(
        options.local_store_count,
        index_tablespace_oid,
        options.local_store_tablespaces.as_deref(),
        resolve_tablespace_name,
    )
}

fn resolve_tablespace_name(name: &str) -> Result<u32, String> {
    let c_name = CString::new(name)
        .map_err(|_| "ec_spire local_store_tablespaces cannot contain NUL bytes".to_owned())?;
    // SAFETY: c_name is a NUL-terminated CString that lives for the lookup call.
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
    pub(super) max_routed_candidate_rows: Option<usize>,
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
    GucRegistry::define_int_guc(
        c"ec_spire.max_routed_candidate_rows",
        c"Session cap for ec_spire routed leaf assignment rows.",
        c"Caps the estimated base assignment rows selected by leaf routing before leaf payloads are read; 0 disables this cap.",
        &EC_SPIRE_MAX_ROUTED_CANDIDATE_ROWS_GUC,
        EC_SPIRE_SESSION_MAX_ROUTED_CANDIDATE_ROWS_DISABLED,
        EC_SPIRE_MAX_MAX_CANDIDATE_ROWS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_spire.candidate_batch_scoring",
        c"Enable Task 87 ec_spire CandidateBatch assignment scoring.",
        c"Diagnostic Task 87 switch; enabled by default. Disable only to compare the structural CandidateBatch route against the pre-Task-87 scalar TurboQuant assignment scoring path.",
        &EC_SPIRE_CANDIDATE_BATCH_SCORING_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_spire.pre_materialization_prune",
        c"Enable Task 122 ec_spire bounded pre-materialization candidate pruning.",
        c"Diagnostic Task 122/123 switch; disabled by default pending Task 131. Recall-safe and latency-neutral in the Task 123 packet 019 b2/b4 A/B, but its leaf-side engagement (rows pruned) was never measured, so it ships as opt-in plumbing rather than a promoted default. Enable for A/B measurement of the bounded score/top-k/materialization prune.",
        &EC_SPIRE_PRE_MATERIALIZATION_PRUNE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_spire.remote_search_global_pre_heap_merge",
        c"Enable Task 131 SPIRE global merge-before-heap pruning.",
        c"Diagnostic Task 131 switch; disabled by default. When enabled, production distributed reads globally merge compact remote candidate batches before non-payload remote heap resolution and request heap rows only for globally surviving candidates.",
        &EC_SPIRE_REMOTE_SEARCH_GLOBAL_PRE_HEAP_MERGE_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_rows",
        c"Build-time SPIRE leaf summary block row count.",
        c"Controls optional leaf V3 block-summary materialization for new index builds; 0 disables summaries. Task 79 evidence uses this to sweep leaf-local block sizes.",
        &EC_SPIRE_LEAF_BLOCK_ROWS_GUC,
        EC_SPIRE_SESSION_LEAF_BLOCK_ROWS_DISABLED,
        EC_SPIRE_MAX_LEAF_BLOCK_ROWS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_summary_representatives",
        c"Build-time SPIRE RaBitQ summary representatives per leaf block.",
        c"Controls how many encoded representatives are stored in each RaBitQ leaf block summary for new index builds. Existing indexes keep their persisted representative count.",
        &EC_SPIRE_LEAF_BLOCK_SUMMARY_REPRESENTATIVES_GUC,
        1,
        EC_SPIRE_MAX_LEAF_BLOCK_SUMMARY_REPRESENTATIVES,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_pruning_max_blocks_per_leaf",
        c"Session cap for SPIRE leaf block-summary pruning.",
        c"Maximum scored summary blocks retained per routed leaf before row scoring; 0 disables leaf-local block pruning and scans full leaves.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_BLOCKS_PER_LEAF_GUC,
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED,
        EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_BLOCKS_PER_LEAF,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_pruning_max_global_blocks",
        c"Session cap for SPIRE global leaf block-summary pruning.",
        c"Maximum scored summary blocks retained across all routed leaves before row scoring; 0 disables global block pruning and uses leaf-local pruning settings.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_GLOBAL_BLOCKS_GUC,
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED,
        EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_GLOBAL_BLOCKS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_pruning_global_probe_blocks",
        c"Session first-pass block count for sampled SPIRE global leaf block pruning.",
        c"When greater than ec_spire.leaf_block_pruning_max_global_blocks and sample rows are enabled, retains this many summary-ranked blocks for row sampling before the final global block cap is applied; 0 disables sampled global probing.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_GLOBAL_PROBE_BLOCKS_GUC,
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED,
        EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_GLOBAL_PROBE_BLOCKS,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.leaf_block_pruning_sample_rows_per_block",
        c"Session row sample count for sampled SPIRE global leaf block pruning.",
        c"Number of deterministic rows scored from each first-pass block before final global block selection; sampled rows enter normal candidate accounting. 0 disables sampled global probing.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_ROWS_PER_BLOCK_GUC,
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED,
        EC_SPIRE_MAX_LEAF_BLOCK_PRUNING_SAMPLE_ROWS_PER_BLOCK,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.leaf_block_pruning_sample_summary_prior_weight",
        c"Summary prior weight for sampled SPIRE global leaf block pruning.",
        c"Blends summary score with strong sampled-row evidence during final global block selection; 1.0 keeps summary-only order, lower values let sampled rows raise block score. Samples never lower summary score.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT_GUC,
        0.0,
        1.0,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.leaf_block_pruning_summary_radius_weight",
        c"RaBitQ radius weight for SPIRE leaf block-summary pruning.",
        c"Scales the RaBitQ summary radius term used when ranking leaf blocks; 0.0 ranks by encoded summary payload only and 1.0 uses the full Cauchy-Schwarz upper bound.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT_GUC,
        0.0,
        1.0,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_float_guc(
        c"ec_spire.leaf_block_pruning_route_prior_weight",
        c"Routing score prior weight for SPIRE global leaf block pruning.",
        c"Adds this multiple of the routed leaf score to each leaf block summary score during global block selection; 0.0 preserves summary-only ordering.",
        &EC_SPIRE_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT_GUC,
        0.0,
        1.0,
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
        c"ec_spire.remote_search_connection_pool_size",
        c"Per-backend ec_spire remote-search connection pool size.",
        c"Maximum idle remote-search libpq/TLS sessions retained by one coordinator backend; 0 disables pooling.",
        &EC_SPIRE_REMOTE_SEARCH_CONNECTION_POOL_SIZE_GUC,
        0,
        EC_SPIRE_MAX_REMOTE_SEARCH_CONNECTION_POOL_SIZE,
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
        c"Maximum selected PIDs or returned remote heap rows accepted for one remote payload batch before strict mode fails closed; default 128 covers the Phase 13 representative k=100 lane with a small safety margin.",
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
    if cfg!(test) {
        EC_SPIRE_SESSION_NPROBE_UNSET
    } else {
        EC_SPIRE_NPROBE_GUC.get()
    }
}

#[cfg(test)]
pub(super) fn candidate_batch_scoring_enabled() -> bool {
    true
}

#[cfg(not(test))]
pub(super) fn candidate_batch_scoring_enabled() -> bool {
    EC_SPIRE_CANDIDATE_BATCH_SCORING_GUC.get()
}

#[cfg(test)]
pub(super) fn pre_materialization_prune_enabled() -> bool {
    true
}

#[cfg(not(test))]
pub(super) fn pre_materialization_prune_enabled() -> bool {
    EC_SPIRE_PRE_MATERIALIZATION_PRUNE_GUC.get()
}

#[cfg(test)]
pub(super) fn remote_search_global_pre_heap_merge_enabled() -> bool {
    true
}

#[cfg(not(test))]
pub(super) fn remote_search_global_pre_heap_merge_enabled() -> bool {
    EC_SPIRE_REMOTE_SEARCH_GLOBAL_PRE_HEAP_MERGE_GUC.get()
}

pub(super) fn current_session_rerank_width() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_RERANK_WIDTH_UNSET
    } else {
        EC_SPIRE_RERANK_WIDTH_GUC.get()
    }
}

pub(super) fn current_session_max_candidate_rows() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_MAX_CANDIDATE_ROWS_UNSET
    } else {
        EC_SPIRE_MAX_CANDIDATE_ROWS_GUC.get()
    }
}

pub(super) fn current_session_max_routed_candidate_rows() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_MAX_ROUTED_CANDIDATE_ROWS_DISABLED
    } else {
        EC_SPIRE_MAX_ROUTED_CANDIDATE_ROWS_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_rows() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_LEAF_BLOCK_ROWS_DISABLED
    } else {
        EC_SPIRE_LEAF_BLOCK_ROWS_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_summary_representatives() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_LEAF_BLOCK_SUMMARY_REPRESENTATIVES
    } else {
        EC_SPIRE_LEAF_BLOCK_SUMMARY_REPRESENTATIVES_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_max_blocks_per_leaf() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_BLOCKS_PER_LEAF_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_max_global_blocks() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_MAX_GLOBAL_BLOCKS_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_global_probe_blocks() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_GLOBAL_PROBE_BLOCKS_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_sample_rows_per_block() -> i32 {
    if cfg!(test) {
        EC_SPIRE_SESSION_LEAF_BLOCK_PRUNING_DISABLED
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_ROWS_PER_BLOCK_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_sample_summary_prior_weight() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_SAMPLE_SUMMARY_PRIOR_WEIGHT_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_summary_radius_weight() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_SUMMARY_RADIUS_WEIGHT_GUC.get()
    }
}

pub(super) fn current_session_leaf_block_pruning_route_prior_weight() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT
    } else {
        EC_SPIRE_LEAF_BLOCK_PRUNING_ROUTE_PRIOR_WEIGHT_GUC.get()
    }
}

pub(super) fn current_session_adaptive_nprobe() -> bool {
    if cfg!(test) {
        false
    } else {
        EC_SPIRE_ADAPTIVE_NPROBE_GUC.get()
    }
}

pub(super) fn current_session_adaptive_nprobe_score_gap_micros() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_ADAPTIVE_NPROBE_SCORE_GAP_MICROS
    } else {
        EC_SPIRE_ADAPTIVE_NPROBE_SCORE_GAP_MICROS_GUC.get()
    }
}

pub(super) fn current_session_remote_search_max_nodes() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET
    } else {
        EC_SPIRE_REMOTE_SEARCH_MAX_NODES_GUC.get()
    }
}

pub(super) fn current_session_remote_search_max_pids() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET
    } else {
        EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_GUC.get()
    }
}

pub(super) fn current_session_remote_search_max_pids_per_node() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET
    } else {
        EC_SPIRE_REMOTE_SEARCH_MAX_PIDS_PER_NODE_GUC.get()
    }
}

pub(super) fn current_session_remote_search_max_concurrent_dispatches() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET
    } else {
        EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_GUC.get()
    }
}

pub(super) fn current_session_remote_search_max_concurrent_dispatches_per_node() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_LIMIT_UNSET
    } else {
        EC_SPIRE_REMOTE_SEARCH_MAX_CONCURRENT_DISPATCHES_PER_NODE_GUC.get()
    }
}

pub(super) fn current_session_remote_search_connect_timeout_ms() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS
    } else {
        EC_SPIRE_REMOTE_SEARCH_CONNECT_TIMEOUT_MS_GUC.get()
    }
}

pub(super) fn current_session_remote_search_statement_timeout_ms() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_TIMEOUT_MS
    } else {
        EC_SPIRE_REMOTE_SEARCH_STATEMENT_TIMEOUT_MS_GUC.get()
    }
}

pub(super) fn current_session_remote_search_connection_pool_size() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_REMOTE_SEARCH_CONNECTION_POOL_SIZE
    } else {
        EC_SPIRE_REMOTE_SEARCH_CONNECTION_POOL_SIZE_GUC.get()
    }
}

pub(super) fn current_session_max_remote_payload_bytes_per_row() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW
    } else {
        EC_SPIRE_MAX_REMOTE_PAYLOAD_BYTES_PER_ROW_GUC.get()
    }
}

pub(super) fn current_session_max_remote_payload_rows_per_batch() -> i32 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH
    } else {
        EC_SPIRE_MAX_REMOTE_PAYLOAD_ROWS_PER_BATCH_GUC.get()
    }
}

pub(super) fn current_session_cost_routing_dimension_scale() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_ROUTING_DIMENSION_SCALE
    } else {
        EC_SPIRE_COST_ROUTING_DIMENSION_SCALE_GUC.get()
    }
}

pub(super) fn current_session_cost_leaf_dimension_scale() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_LEAF_DIMENSION_SCALE
    } else {
        EC_SPIRE_COST_LEAF_DIMENSION_SCALE_GUC.get()
    }
}

pub(super) fn current_session_cost_index_page_scale() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_INDEX_PAGE_SCALE
    } else {
        EC_SPIRE_COST_INDEX_PAGE_SCALE_GUC.get()
    }
}

pub(super) fn current_session_cost_local_store_page_fanout_scale() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_LOCAL_STORE_PAGE_FANOUT_SCALE
    } else {
        EC_SPIRE_COST_LOCAL_STORE_PAGE_FANOUT_SCALE_GUC.get()
    }
}

pub(super) fn current_session_cost_storage_scoring_multiplier() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_STORAGE_SCORING_MULTIPLIER
    } else {
        EC_SPIRE_COST_STORAGE_SCORING_MULTIPLIER_GUC.get()
    }
}

pub(super) fn current_session_cost_rerank_multiplier() -> f64 {
    if cfg!(test) {
        EC_SPIRE_DEFAULT_COST_RERANK_MULTIPLIER
    } else {
        EC_SPIRE_COST_RERANK_MULTIPLIER_GUC.get()
    }
}

pub(super) fn current_session_remote_search_consistency_mode_name() -> &'static str {
    if cfg!(test) {
        SpireRemoteSearchConsistencyModeGuc::Strict.as_str()
    } else {
        EC_SPIRE_REMOTE_SEARCH_CONSISTENCY_MODE_GUC.get().as_str()
    }
}

pub(super) fn current_session_remote_tuple_transport() -> SpireRemoteTupleTransportGuc {
    if cfg!(test) {
        SpireRemoteTupleTransportGuc::Auto
    } else {
        EC_SPIRE_REMOTE_TUPLE_TRANSPORT_GUC.get()
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn current_session_remote_search_governance_test_namespace() -> i32 {
    if cfg!(test) {
        0
    } else {
        EC_SPIRE_REMOTE_SEARCH_GOVERNANCE_TEST_NAMESPACE_GUC.get()
    }
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
        current_session_max_routed_candidate_rows(),
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
        EC_SPIRE_SESSION_MAX_ROUTED_CANDIDATE_ROWS_DISABLED,
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
    session_max_routed_candidate_rows_value: i32,
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
    let max_routed_candidate_rows =
        resolve_scan_max_routed_candidate_rows_value(session_max_routed_candidate_rows_value)?;

    Ok(SpireSingleLevelScanPlan {
        leaf_count,
        nprobe: nprobe.effective_nprobe,
        nprobe_source: nprobe.source,
        recursive_nprobe_policy,
        recursive_route_budget,
        max_routed_candidate_rows,
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

fn resolve_scan_max_routed_candidate_rows_value(value: i32) -> Result<Option<usize>, String> {
    if value <= 0 {
        return Ok(None);
    }
    if value > EC_SPIRE_MAX_MAX_CANDIDATE_ROWS {
        return Err(format!(
            "ec_spire.max_routed_candidate_rows must be between 0 and {EC_SPIRE_MAX_MAX_CANDIDATE_ROWS}, got {value}"
        ));
    }
    usize::try_from(value)
        .map(Some)
        .map_err(|_| "ec_spire.max_routed_candidate_rows exceeds usize".to_owned())
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
    // SAFETY: PostgreSQL invokes amoptions with a reloptions Datum and validate
    // flag; the guarded closure registers the local reloptions layout and
    // returns the bytea allocated by PostgreSQL's reloptions builder.
    pg_am_callback!({
        let mut relopts = pg_sys::local_relopts::default();

        pg_sys::init_local_reloptions(&mut relopts, size_of::<EcSpireReloptions>());
        pg_sys::add_local_int_reloption(
            &mut relopts,
            c"nlists".as_ptr(),
            c"Number of single-level SPIRE-IVF leaf PIDs; 0 chooses an automatic value.".as_ptr(),
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
            c"Enable SPIRE top-graph build/scan plumbing; 0 keeps flat recursive routing.".as_ptr(),
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

struct EcSpireReloptionsView {
    rd_options: crate::am::common::reloptions::ReloptionsBlob,
}

impl EcSpireReloptionsView {
    fn reloptions(&self) -> &EcSpireReloptions {
        crate::storage::relation::relation_options_layout_ref(self.rd_options.handle())
    }

    fn read_string_reloption(&self, offset: i32, name: &str) -> Option<String> {
        self.rd_options
            .read_string_reloption(offset, "ec_spire", name)
    }

    fn validate(&self) {
        let reloptions = self.reloptions();
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
    }

    fn to_options(&self) -> EcSpireOptions {
        self.validate();
        let reloptions = self.reloptions();
        let storage_format_reloption =
            self.read_string_reloption(reloptions.storage_format_offset, "storage_format");
        let quantizer_reloption =
            self.read_string_reloption(reloptions.quantizer_offset, "quantizer");
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
        let source_identity = self
            .read_string_reloption(reloptions.source_identity_offset, "source_identity")
            .map(|value| {
                SpireSourceIdentityProvider::parse_reloption(&value)
                    .unwrap_or_else(|e| pgrx::error!("{e}"))
            })
            .unwrap_or(SpireSourceIdentityProvider::None);
        let local_store_tablespaces = self
            .read_string_reloption(
                reloptions.local_store_tablespaces_offset,
                "local_store_tablespaces",
            )
            .map(|value| {
                normalize_local_store_tablespaces_reloption(&value, reloptions.local_store_count)
                    .unwrap_or_else(|e| pgrx::error!("{e}"))
            });
        let nprobe_per_level = self
            .read_string_reloption(reloptions.nprobe_per_level_offset, "nprobe_per_level")
            .map(|value| {
                parse_nprobe_per_level_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
            });

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
}

pub(super) fn relation_options(index_relation: pg_sys::Relation) -> EcSpireOptions {
    let index_relation = std::ptr::NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_spire relation options need a valid index relation"));
    let rd_options = crate::storage::relation::relation_options_handle(index_relation);
    let Some(rd_options) = std::ptr::NonNull::new(rd_options) else {
        return EcSpireOptions::DEFAULT;
    };
    EcSpireReloptionsView {
        rd_options: crate::am::common::reloptions::ReloptionsBlob::new(rd_options),
    }
    .to_options()
}

include!("tests.rs");
