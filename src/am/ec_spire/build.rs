use std::collections::{HashMap, HashSet};
use std::ffi::{c_void, CStr};
use std::mem::size_of;
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox, PgTupleDesc};

use super::assign::{
    build_primary_leaf_assignments, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
    SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireLocalStoreConfig,
    SpireManifestEntry, SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry,
    SpireRootControlState, SpireValidatedEpochSnapshot, SPIRE_MIN_EPOCH_RETENTION_SECS,
};
use super::storage::{
    create_local_store_relations_for_build, local_store_config_from_relation_plan,
    plan_local_store_relations, SpireLeafAssignmentRow, SpireLeafPartitionObject,
    SpireLocalObjectStore, SpireLocalObjectStoreSet, SpireObjectReader, SpirePartitionObjectKind,
    SpireRelationObjectStore, SpireRelationObjectStoreSet, SpireRoutingChildEntry,
    SpireRoutingPartitionObject,
};
use super::{options, page};
use super::{quantizer, quantizer::SpireAssignmentPayloadFormat};
use crate::am::common::training as common_training;
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::ItemPointer;

pub(super) const SPIRE_DEFAULT_KMEANS_ITERATIONS: usize = 8;
const SPIRE_DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
pub(super) const SPIRE_INITIAL_EPOCH: u64 = 1;
pub(super) const SPIRE_INITIAL_OBJECT_VERSION: u64 = 1;
const MICROS_PER_SECOND: i64 = 1_000_000;

include!("build/types.rs");
include!("build/object_store.rs");
include!("build/publish.rs");
include!("build/routing_plan.rs");
include!("build/recursive.rs");
include!("build/training.rs");
include!("build/drafts.rs");
include!("build/tuples.rs");
include!("build/tests.rs");
