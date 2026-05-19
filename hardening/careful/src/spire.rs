pub mod assign {
    pub(super) const SPIRE_FIRST_PID: u64 = 1;
    pub(super) const SPIRE_FIRST_LOCAL_VEC_SEQ: u64 = 1;
}

#[path = "../../../src/am/ec_spire/meta.rs"]
pub mod meta;

#[path = "../../../src/am/ec_spire/page.rs"]
pub mod page;

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

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/vec_id.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/storage/relation_plan.rs"
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

    #[cfg(test)]
    mod tests {
        use super::super::meta::{
            SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpirePlacementState,
        };
        use super::{
            decode_leaf_v2_local_vec_id, is_delete_delta_assignment,
            is_visible_primary_assignment, is_visible_primary_assignment_ref,
            is_visible_scored_assignment, local_store_config_from_relation_plan,
            plan_local_store_relations, spire_local_store_relation_name, SpireDeltaPartitionObject,
            SpireLeafAssignmentRow, SpireLeafPartitionObject, SpireLeafPartitionObjectV2Meta,
            SpireLeafPartitionObjectV2Segment, SpireLocalObjectStore, SpireLocalObjectStoreSet,
            SpireLocalStoreState, SpireObjectReader, SpirePartitionObjectHeader,
            SpirePartitionObjectKind, SpireRoutingChildEntry, SpireRoutingPartitionObject,
            SpireTopGraphNodeRecord, SpireTopGraphPartitionObject, SpireVecId, SpireVecIdKind,
            LEAF_V2_LOCAL_VEC_ID_STRIDE,
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
            "/../../src/am/ec_spire/storage/tests/local_store_plan.rs"
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
        fn local_object_store_set_round_trips_non_leaf_object_kinds() {
            // Pin the insert/read delegations through SpireObjectReader for
            // every non-leaf object kind. The leaf-V2 path is already pinned
            // by local_object_store_set_routes_by_pid_and_reads_back_objects;
            // this test pins routing / delta / top-graph / read_object_header
            // so a mis-routed store_for_placement is observable for each.
            let config = SpireLocalStoreConfig::from_stores(
                1,
                vec![
                    SpireLocalStoreDescriptor::available(0, 10_000, 0).unwrap(),
                    SpireLocalStoreDescriptor::available(1, 10_001, 0).unwrap(),
                ],
            )
            .unwrap();
            let mut store_set = SpireLocalObjectStoreSet::from_config(config, 1024).unwrap();

            // Routing.
            let routing =
                SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
            let routing_placement = store_set.insert_routing_object(7, &routing).unwrap();
            assert_eq!(
                store_set
                    .read_routing_object(&routing_placement)
                    .unwrap()
                    .header
                    .pid,
                11
            );
            assert_eq!(
                store_set
                    .read_object_header(&routing_placement)
                    .unwrap()
                    .kind,
                SpirePartitionObjectKind::Root,
            );

            // Delta.
            let delta = SpireDeltaPartitionObject::new(
                19,
                4,
                17,
                vec![SpireLeafAssignmentRow {
                    flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                    vec_id: SpireVecId::local(1),
                    heap_tid: ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    },
                    payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2, 3, 4],
                }],
            )
            .unwrap();
            let delta_placement = store_set.insert_delta_object(7, &delta).unwrap();
            assert_eq!(
                store_set
                    .read_delta_object(&delta_placement)
                    .unwrap()
                    .header
                    .pid,
                19
            );

            // Top-graph.
            let top_graph = SpireTopGraphPartitionObject::new(
                90,
                3,
                11,
                2,
                2,
                2,
                4,
                1.2,
                0,
                vec![SpireTopGraphNodeRecord {
                    child_pid: 21,
                    centroid_ordinal: 0,
                    neighbors: vec![],
                }],
            )
            .unwrap();
            let top_graph_placement = store_set.insert_top_graph_object(7, &top_graph).unwrap();
            assert_eq!(
                store_set
                    .read_top_graph_object(&top_graph_placement)
                    .unwrap()
                    .header
                    .pid,
                90
            );
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

#[cfg(test)]
mod page_tests {
    //! Tests for `src/am/ec_spire/page.rs` via the careful shadow crate.
    //!
    //! These exercise the early-error paths that return `Err` before any
    //! `pg_sys` page operation runs. The page-level stubs in
    //! `careful_pg_guards::pg_sys` (PageInit, PageAddItemExtended, ...)
    //! are no-ops; tests here deliberately do not depend on them, so the
    //! shadow scaffold can stay minimal.
    use super::page;
    use crate::careful_pg_guards::pg_sys;
    use crate::storage::page::{ItemPointer, FIRST_DATA_BLOCK_NUMBER, METADATA_BLOCK_NUMBER};

    fn null_relation() -> pg_sys::Relation {
        std::ptr::null_mut()
    }

    #[test]
    fn read_object_tuple_rejects_metadata_block_tid() {
        let tid = ItemPointer {
            block_number: METADATA_BLOCK_NUMBER,
            offset_number: 1,
        };
        // SAFETY: returns Err before touching the relation pointer.
        let err = unsafe { page::read_object_tuple(null_relation(), tid) }
            .expect_err("metadata block must be rejected");
        assert!(
            err.contains("cannot use metadata block"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn append_object_tuple_rejects_empty_payload() {
        // SAFETY: returns Err before touching the relation pointer.
        let err = unsafe { page::append_object_tuple(null_relation(), &[]) }
            .expect_err("empty payload must be rejected");
        assert!(
            err.contains("payload must not be empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn append_object_tuple_rejects_uninitialized_relation() {
        // Force RelationGetNumberOfBlocksInFork to report zero blocks so the
        // append helper bails out before requesting any page.
        pg_sys::set_relation_block_count(0);
        // SAFETY: shadow RelationGetNumberOfBlocksInFork returns 0; the
        // helper returns Err before touching the relation pointer.
        let err = unsafe { page::append_object_tuple(null_relation(), &[1, 2, 3]) }
            .expect_err("uninitialized relation must be rejected");
        assert!(
            err.contains("root/control block must be initialized"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn delete_object_tuples_no_compact_is_noop_for_empty_input() {
        // SAFETY: the helper iterates the empty slice and returns Ok early.
        let (count, bytes) = unsafe { page::delete_object_tuples_no_compact(null_relation(), &[]) }
            .expect("empty delete should succeed");
        assert_eq!(count, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn delete_object_tuples_no_compact_rejects_metadata_block_tid() {
        let tids = [ItemPointer {
            block_number: METADATA_BLOCK_NUMBER,
            offset_number: 1,
        }];
        // SAFETY: returns Err while validating the TID list, before opening
        // any buffer.
        let err = unsafe { page::delete_object_tuples_no_compact(null_relation(), &tids) }
            .expect_err("metadata block tid must be rejected");
        assert!(
            err.contains("cannot remove metadata block"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn first_data_block_is_immediately_after_metadata_block() {
        // Pin the on-disk-layout invariant the page.rs guards rely on.
        assert_eq!(METADATA_BLOCK_NUMBER, 0);
        assert_eq!(FIRST_DATA_BLOCK_NUMBER, 1);
    }
}
