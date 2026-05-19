pub mod assign {
    pub(super) const SPIRE_FIRST_PID: u64 = 1;
    pub(super) const SPIRE_FIRST_LOCAL_VEC_SEQ: u64 = 1;
}

#[path = "../../../src/am/ec_spire/meta.rs"]
pub mod meta;

pub mod storage {
    use std::{
        collections::{BTreeMap, BTreeSet, HashMap, HashSet},
        mem::size_of,
        ptr,
    };

    use super::meta::{
        SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireLocalStoreState,
        SpirePlacementEntry, SpirePlacementState, SPIRE_LOCAL_NODE_ID,
        SPIRE_SINGLE_LOCAL_STORE_ID,
    };
    use crate::storage::page::{
        element_or_neighbor_tuple_fits, raw_tuple_storage_bytes, usable_page_bytes, DataPageChain,
        ItemPointer, DEFAULT_PAGE_SIZE, ITEM_POINTER_BYTES,
    };

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum SpirePartitionObjectKind {
        Root = 1,
        Internal = 2,
        Leaf = 3,
        Delta = 4,
        TopGraph = 5,
    }

    impl SpirePartitionObjectKind {
        fn decode(value: u8) -> Result<Self, String> {
            match value {
                1 => Ok(Self::Root),
                2 => Ok(Self::Internal),
                3 => Ok(Self::Leaf),
                4 => Ok(Self::Delta),
                5 => Ok(Self::TopGraph),
                other => Err(format!("ec_spire invalid partition object kind: {other}")),
            }
        }
    }

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/vec_id.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/header.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/assignment.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/leaf_v1.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/leaf_v2_parts.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/leaf_v2.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/routing_delta.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/top_graph.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/local_store.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/local_store_set.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/helpers.rs"
    ));

    fn spire_local_store_relation_name(
        index_relid: u32,
        local_store_id: u32,
    ) -> Result<String, String> {
        if index_relid == 0 {
            return Err("ec_spire local store relation name needs a valid index relid".to_owned());
        }

        Ok(format!("ec_spire_store_{index_relid}_{local_store_id}"))
    }

    #[cfg(test)]
    mod tests {
        use super::super::meta::{
            SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpirePlacementState,
        };
        use super::{
            decode_leaf_v2_local_vec_id, is_delete_delta_assignment,
            is_visible_primary_assignment, is_visible_primary_assignment_ref,
            is_visible_scored_assignment, spire_local_store_relation_name, SpireDeltaPartitionObject,
            SpireLeafAssignmentRow, SpireLeafPartitionObject, SpireLeafPartitionObjectV2Meta,
            SpireLeafPartitionObjectV2Segment, SpireLocalObjectStore, SpireLocalObjectStoreSet,
            SpireObjectReader, SpirePartitionObjectHeader, SpirePartitionObjectKind,
            SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireTopGraphNodeRecord,
            SpireTopGraphPartitionObject, SpireVecId, SpireVecIdKind, LEAF_V2_LOCAL_VEC_ID_STRIDE,
            SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
            SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, SPIRE_LOCAL_VEC_ID_DISCRIMINATOR,
            SPIRE_PAYLOAD_FORMAT_NONE, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            SPIRE_PAYLOAD_FORMAT_TURBOQUANT, SPIRE_VEC_ID_MAX_BYTES,
        };
        use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

        fn routing_children() -> Vec<SpireRoutingChildEntry> {
            vec![
                SpireRoutingChildEntry {
                    centroid_index: 0,
                    child_pid: 17,
                    centroid: vec![1.0, 0.0],
                },
                SpireRoutingChildEntry {
                    centroid_index: 1,
                    child_pid: 18,
                    centroid: vec![-1.0, 0.0],
                },
            ]
        }

        fn leaf_v2_assignment(local_vec_seq: u64, payload_len: usize) -> SpireLeafAssignmentRow {
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(local_vec_seq),
                heap_tid: ItemPointer {
                    block_number: 100 + local_vec_seq as u32,
                    offset_number: local_vec_seq as u16,
                },
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: local_vec_seq as f32 / 10.0,
                encoded_payload: vec![local_vec_seq as u8; payload_len],
            }
        }

        fn leaf_v2_global_assignment(
            global_payload: &[u8],
            heap_block_number: u32,
            heap_offset_number: u16,
            payload_len: usize,
        ) -> SpireLeafAssignmentRow {
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::global(global_payload).unwrap(),
                heap_tid: ItemPointer {
                    block_number: heap_block_number,
                    offset_number: heap_offset_number,
                },
                payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: heap_offset_number as f32 / 10.0,
                encoded_payload: vec![heap_offset_number as u8; payload_len],
            }
        }

        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/vec_and_routing.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/top_graph.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/assignment.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/leaf.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/delta.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src/am/ec_spire/storage/tests/local_store.rs"
        ));

        #[test]
        fn local_object_store_set_routes_by_pid_and_reads_back_objects() {
            let config = SpireLocalStoreConfig::from_stores(
                1,
                vec![
                    SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap(),
                    SpireLocalStoreDescriptor::available(1, 10_001, 0).unwrap(),
                ],
            )
            .unwrap();
            let expected_store = config.store_for_pid(17).unwrap().local_store_id;
            let assignments = vec![leaf_v2_assignment(1, 8), leaf_v2_assignment(2, 8)];
            let mut store_set = SpireLocalObjectStoreSet::from_config(config, 512).unwrap();

            let placement = store_set
                .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &assignments)
                .unwrap();
            let decoded = store_set.read_leaf_object_v2(&placement).unwrap();

            assert_eq!(placement.local_store_id, expected_store);
            assert_eq!(decoded.meta.header.pid, 17);
            assert_eq!(decoded.meta.header.assignment_count, 2);
        }

        #[test]
        fn local_object_store_set_rejects_unconfigured_placements() {
            let config = SpireLocalStoreConfig::from_stores(
                1,
                vec![SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap()],
            )
            .unwrap();
            let mut store_set = SpireLocalObjectStoreSet::from_config(config, 512).unwrap();
            let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
            let mut placement = store_set.insert_routing_object(7, &object).unwrap();

            placement.local_store_id = 99;
            assert!(store_set.read_routing_object(&placement).is_err());
        }
    }
}
