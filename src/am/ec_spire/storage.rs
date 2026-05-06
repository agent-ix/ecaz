//! PID-addressed partition-object storage codecs.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    mem::size_of,
    ptr,
};

use pgrx::pg_sys;

use super::meta::{
    SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireLocalStoreState,
    SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState, SPIRE_LOCAL_NODE_ID,
    SPIRE_SINGLE_LOCAL_STORE_ID,
};
use super::page;
use crate::storage::page::{
    element_or_neighbor_tuple_fits, raw_tuple_storage_bytes, usable_page_bytes, DataPageChain,
    ItemPointer, DEFAULT_PAGE_SIZE, ITEM_POINTER_BYTES,
};

include!("storage/vec_id.rs");
include!("storage/relation_plan.rs");
include!("storage/header.rs");
include!("storage/assignment.rs");
include!("storage/leaf_v1.rs");
include!("storage/leaf_v2_parts.rs");
include!("storage/leaf_v2.rs");
include!("storage/routing_delta.rs");
include!("storage/local_store.rs");
include!("storage/local_store_set.rs");
include!("storage/relation_store.rs");
include!("storage/helpers.rs");
include!("storage/tests.rs");
