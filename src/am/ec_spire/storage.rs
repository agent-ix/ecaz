//! PID-addressed partition-object storage codecs.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
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
include!("storage/top_graph.rs");
include!("storage/local_store.rs");
include!("storage/local_store_set.rs");
include!("storage/relation_store.rs");
include!("storage/helpers.rs");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpirePartitionHeaderFixture {
    pub kind: u8,
    pub pid: u64,
    pub object_version: u64,
    pub published_epoch_backref: u64,
    pub level: u16,
    pub parent_pid: u64,
    pub child_count: u32,
    pub assignment_count: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpireAssignmentRowFixture {
    pub flags: u16,
    pub vec_id: Vec<u8>,
    pub heap_tid: ItemPointer,
    pub payload_format: u8,
    pub gamma: f32,
    pub encoded_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpireLeafPartitionObjectFixture {
    pub header: SpirePartitionHeaderFixture,
    pub assignments: Vec<SpireAssignmentRowFixture>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpireRoutingPartitionObjectFixture {
    pub header: SpirePartitionHeaderFixture,
    pub dimensions: u16,
    pub centroid_ordinals: Vec<u32>,
    pub child_pids: Vec<u64>,
    pub centroids: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpireDeltaPartitionObjectFixture {
    pub header: SpirePartitionHeaderFixture,
    pub assignments: Vec<SpireAssignmentRowFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpireTopGraphNodeFixture {
    pub child_pid: u64,
    pub centroid_ordinal: u32,
    pub neighbors: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpireTopGraphPartitionObjectFixture {
    pub header: SpirePartitionHeaderFixture,
    pub root_pid: u64,
    pub dimensions: u16,
    pub graph_degree: u32,
    pub build_list_size: u32,
    pub alpha: f32,
    pub entry_node: u32,
    pub nodes: Vec<SpireTopGraphNodeFixture>,
}

fn spire_partition_header_fixture(
    header: SpirePartitionObjectHeader,
) -> SpirePartitionHeaderFixture {
    SpirePartitionHeaderFixture {
        kind: header.kind as u8,
        pid: header.pid,
        object_version: header.object_version,
        published_epoch_backref: header.published_epoch_backref,
        level: header.level,
        parent_pid: header.parent_pid,
        child_count: header.child_count,
        assignment_count: header.assignment_count,
        flags: header.flags,
    }
}

fn spire_assignment_row_fixture(row: &SpireLeafAssignmentRow) -> SpireAssignmentRowFixture {
    SpireAssignmentRowFixture {
        flags: row.flags,
        vec_id: row.vec_id.as_bytes().to_vec(),
        heap_tid: row.heap_tid,
        payload_format: row.payload_format,
        gamma: row.gamma,
        encoded_payload: row.encoded_payload.clone(),
    }
}

pub fn spire_decode_leaf_partition_object_fixture(
    input: &[u8],
) -> Result<SpireLeafPartitionObjectFixture, String> {
    let object = SpireLeafPartitionObject::decode(input)?;
    Ok(SpireLeafPartitionObjectFixture {
        header: spire_partition_header_fixture(object.header),
        assignments: object
            .assignments
            .iter()
            .map(spire_assignment_row_fixture)
            .collect(),
    })
}

pub fn spire_decode_routing_partition_object_fixture(
    input: &[u8],
) -> Result<SpireRoutingPartitionObjectFixture, String> {
    let object = SpireRoutingPartitionObject::decode(input)?;
    Ok(SpireRoutingPartitionObjectFixture {
        header: spire_partition_header_fixture(object.header),
        dimensions: object.dimensions,
        centroid_ordinals: object.centroid_ordinals,
        child_pids: object.child_pids,
        centroids: object.centroids,
    })
}

pub fn spire_decode_delta_partition_object_fixture(
    input: &[u8],
) -> Result<SpireDeltaPartitionObjectFixture, String> {
    let object = SpireDeltaPartitionObject::decode(input)?;
    Ok(SpireDeltaPartitionObjectFixture {
        header: spire_partition_header_fixture(object.header),
        assignments: object
            .assignments
            .iter()
            .map(spire_assignment_row_fixture)
            .collect(),
    })
}

pub fn spire_decode_top_graph_partition_object_fixture(
    input: &[u8],
) -> Result<SpireTopGraphPartitionObjectFixture, String> {
    let object = SpireTopGraphPartitionObject::decode(input)?;
    Ok(SpireTopGraphPartitionObjectFixture {
        header: spire_partition_header_fixture(object.header),
        root_pid: object.root_pid,
        dimensions: object.dimensions,
        graph_degree: object.graph_degree,
        build_list_size: object.build_list_size,
        alpha: object.alpha,
        entry_node: object.entry_node,
        nodes: object
            .nodes
            .into_iter()
            .map(|node| SpireTopGraphNodeFixture {
                child_pid: node.child_pid,
                centroid_ordinal: node.centroid_ordinal,
                neighbors: node.neighbors,
            })
            .collect(),
    })
}

include!("storage/tests.rs");
