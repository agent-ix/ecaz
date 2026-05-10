use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use super::build::{
    rank_centroid_routes_by_ip, SpireCentroidRouteInput, SpireRankedCentroidRoute,
    SpireTopGraphBuildDraft,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireLocalStoreConfig, SpireObjectManifest,
    SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState, SpirePublishedEpochSnapshot,
    SpireRootControlState, SpireValidatedEpochSnapshot,
};
use super::options::{
    relation_options, resolve_single_level_scan_plan, EcSpireOptions, SpireCandidateDedupeMode,
    SpireRecursiveNprobePolicy, SpireRecursiveRouteBudget, SpireSingleLevelScanPlan,
    SpireTopGraphOptionPlan,
};
use super::page;
use super::quantizer::{SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer};
use super::storage::{
    is_delete_delta_assignment, is_visible_primary_assignment, is_visible_scored_assignment,
    is_visible_scored_assignment_flags, SpireLeafAssignmentRow, SpireLeafObjectColumns,
    SpireLeafPartitionObject, SpireObjectReader, SpirePartitionObjectKind,
    SpireRoutingPartitionObject, SpireTopGraphPartitionObject, SpireVecId,
    SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
};
use crate::am::ec_hnsw::source;
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::ItemPointer;
use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox, PgMemoryContexts};

include!("scan/types.rs");
include!("scan/snapshot.rs");
include!("scan/candidates.rs");
include!("scan/routing.rs");
include!("scan/leaf_rows.rs");
include!("scan/relation.rs");
include!("scan/callbacks.rs");
include!("scan/tests.rs");
