//! ec_spire access-method scaffold.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod assign;
mod build;
mod cost;
mod custom_scan;
mod diagnostics;
mod dml_frontdoor;
mod insert;
mod meta;
mod options;
mod page;
mod quantizer;
mod routine;
mod scan;
mod storage;
mod update;
mod vacuum;

use pgrx::{pg_sys, Spi};

use self::storage::SpireObjectReader;

pub(crate) use self::cost::index_cost_snapshot;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::custom_scan::custom_scan_store_tuple_payload_json_for_test;
pub(crate) use self::custom_scan::{
    custom_scan_index_eligibility_row, custom_scan_status_row, register_custom_scan,
};
pub(crate) use self::dml_frontdoor::{
    classify_dml_frontdoor_query, SpireDmlFrontdoorCustomScanMode, SpireDmlFrontdoorPkValuePlan,
    SpireDmlFrontdoorQueryContext,
};
pub(crate) use self::dml_frontdoor::{
    dml_frontdoor_bigint_pk_value_bytes, dml_frontdoor_hook_status_row,
    dml_frontdoor_pk_argument_from_replacement_decision,
    dml_frontdoor_primitive_plan_const_pk_value_bytes,
    dml_frontdoor_primitive_plan_from_replacement_decision,
    dml_frontdoor_relation_context_catalog_row, dml_frontdoor_relation_context_row,
    dml_frontdoor_replacement_decision_catalog_row, dml_frontdoor_target_relation_oid,
    register_dml_frontdoor_planner_hook,
};
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::vacuum::{
    debug_spire_vacuum_bulkdelete_heap_tids, debug_spire_vacuum_remove_heap_tids,
};

pub(super) const EC_SPIRE_DEFAULT_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MIN_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MAX_NLISTS: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RECURSIVE_FANOUT: i32 = 0;
pub(super) const EC_SPIRE_MIN_RECURSIVE_FANOUT: i32 = 0;
pub(super) const EC_SPIRE_MAX_RECURSIVE_FANOUT: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT: i32 = 1;
pub(super) const EC_SPIRE_MIN_LOCAL_STORE_COUNT: i32 = 1;
pub(super) const EC_SPIRE_MAX_LOCAL_STORE_COUNT: i32 = 16;
pub(super) const EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT: i32 = 0;
pub(super) const EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT: i32 = 0;
pub(super) const EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT: i32 = 8;
pub(super) const EC_SPIRE_DEFAULT_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MIN_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MIN_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_MAX_CANDIDATE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MIN_MAX_CANDIDATE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MAX_MAX_CANDIDATE_ROWS: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_SEED: i32 = 42;
pub(super) const EC_SPIRE_MIN_SEED: i32 = 0;
pub(super) const EC_SPIRE_MAX_SEED: i32 = i32::MAX;

const SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER: u64 = 4;
const SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS: u64 = 32;
const SPIRE_LEAF_MERGE_AVERAGE_DIVISOR: u64 = 4;
pub(super) const EC_SPIRE_DEFAULT_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_PQ_GROUP_SIZE: i32 = 32;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_ENABLED: i32 = 0;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_ENABLED: i32 = 0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_ENABLED: i32 = 1;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_DEGREE: i32 = 32;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_DEGREE: i32 = 1;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_DEGREE: i32 = 1024;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 100;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 1;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_BUILD_LIST_SIZE: i32 = 100_000;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_ALPHA: f32 = 1.2;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_ALPHA: f32 = 1.0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_ALPHA: f32 = 10.0;
pub(super) const EC_SPIRE_DEFAULT_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_TOP_GRAPH_SEARCH_LIST_SIZE: i32 = 1_000_000;

pub(super) const SPIRE_PUBLISH_LOCK_MODE: pg_sys::LOCKMODE =
    pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE;

include!("root/lifecycle.rs");
include!("root/types.rs");
include!("root/remote_candidates.rs");
include!("root/diagnostics.rs");
include!("root/hierarchy_shape.rs");
include!("root/snapshots.rs");
include!("root/maintenance.rs");
include!("root/hierarchy_snapshots.rs");
include!("root/debug.rs");
include!("root/tests.rs");
