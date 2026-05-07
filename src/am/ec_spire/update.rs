//! Epoch-published insert/delete, split, merge, and cleanup mechanics live here.

use std::collections::{HashMap, HashSet};

use super::assign::{
    build_delete_delta_assignments, build_insert_delta_assignments, SpireDeleteDeltaInput,
    SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::build::{
    encode_manifest_bundle_for_publish, encode_publish_bundle_for_publish,
    object_manifest_from_placement_writes, publish_replacement_epoch_to_relation,
    root_control_state_for_publish, write_placement_entries_to_relation,
    SpireEncodedManifestBundle, SpireEncodedPublishBundle, SpirePublishCoordinatorInput,
    SpirePublishPlacementWriteEvidence, SpirePublishedManifestLocators,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireLocalStoreConfig,
    SpireManifestEntry, SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry,
    SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
    SpireValidatedEpochSnapshot,
};
use super::page;
use super::scan::{
    collect_validated_snapshot_visible_primary_rows, load_indexed_source_vector_from_heap_row,
    load_relation_local_store_config, SpireLeafScanRow,
};
use super::storage::{
    is_delete_delta_assignment, is_visible_primary_assignment, SpireDeltaPartitionObject,
    SpireLeafAssignmentRow, SpireLocalObjectStore, SpireObjectReader, SpirePartitionObjectKind,
    SpireRelationObjectStore, SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
    SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
};
use super::SpireIndexLeafSnapshotRow;
use crate::am::common::training as common_training;
use crate::am::ec_hnsw::source;
use crate::storage::page::ItemPointer;

include!("update/types.rs");
include!("update/scheduler.rs");
include!("update/materialization.rs");
include!("update/routing.rs");
include!("update/leaf_rows.rs");
include!("update/publish.rs");
include!("update/helpers.rs");
include!("update/delta.rs");
include!("update/tests.rs");
