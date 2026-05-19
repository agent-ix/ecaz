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
        SpirePlacementEntry, SpirePlacementState, SpirePlacementDirectory, SPIRE_LOCAL_NODE_ID,
        SPIRE_SINGLE_LOCAL_STORE_ID,
    };
    use super::page;
    use crate::careful_pg_guards::pg_sys;
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
        "/../../src/am/ec_spire/storage/relation_store.rs"
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
            SpirePartitionObjectKind, SpireRelationObjectStore, SpireRoutingChildEntry,
            SpireRoutingPartitionObject, SpireTopGraphNodeRecord, SpireTopGraphPartitionObject,
            SpireVecId, SpireVecIdKind, LEAF_V2_LOCAL_VEC_ID_STRIDE,
            SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
            SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, SPIRE_LOCAL_VEC_ID_DISCRIMINATOR,
            SPIRE_PAYLOAD_FORMAT_NONE, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            SPIRE_PAYLOAD_FORMAT_TURBOQUANT, SPIRE_VEC_ID_MAX_BYTES,
        };
        use crate::careful_pg_guards::pg_sys;
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
        fn relation_object_store_for_index_relation_rejects_null_and_invalid_oid() {
            // null relation pointer.
            let null_result = unsafe {
                SpireRelationObjectStore::for_index_relation(std::ptr::null_mut())
            };
            let null_err = match null_result {
                Err(e) => e,
                Ok(_) => panic!("null relation must be rejected"),
            };
            assert!(
                null_err.contains("needs a valid relation"),
                "unexpected error: {null_err}"
            );

            // Non-null relation with rd_id == InvalidOid.
            let mut relation_data = pg_sys::RelationData {
                rd_att: std::ptr::null_mut(),
                rd_id: pg_sys::InvalidOid,
            };
            let relation: pg_sys::Relation = &mut relation_data;
            let invalid_oid_result = unsafe {
                SpireRelationObjectStore::for_index_relation(relation)
            };
            let invalid_oid_err = match invalid_oid_result {
                Err(e) => e,
                Ok(_) => panic!("invalid oid must be rejected"),
            };
            assert!(
                invalid_oid_err.contains("relid is invalid"),
                "unexpected error: {invalid_oid_err}"
            );
        }

        #[test]
        fn relation_object_store_inserts_reject_epoch_zero() {
            // for_store_relation_id is safe — it just stores the pointer/id.
            let store = SpireRelationObjectStore::for_store_relation_id(
                std::ptr::null_mut(),
                0,
                12345,
            );

            let routing = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
            assert!(store.insert_routing_object(0, &routing).is_err());

            let delta = SpireDeltaPartitionObject::new(
                19,
                4,
                17,
                vec![SpireLeafAssignmentRow {
                    flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                    vec_id: SpireVecId::local(1),
                    heap_tid: crate::storage::page::ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    },
                    payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2],
                }],
            )
            .unwrap();
            assert!(store.insert_delta_object(0, &delta).is_err());

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
            assert!(store.insert_top_graph_object(0, &top_graph).is_err());

            // Leaf V2 from rows: also rejects epoch == 0 before encoding.
            assert!(store.insert_leaf_object_v2_from_rows(0, 17, 3, 5, &[]).is_err());
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

        // ----------------------------------------------------------------
        // Relation-store round trips through the backing-page emulator.
        // ----------------------------------------------------------------

        use super::super::page;
        use std::sync::Mutex;

        static EMULATOR_LOCK: Mutex<()> = Mutex::new(());

        fn synth_relation(oid: u32) -> pg_sys::RelationData {
            pg_sys::RelationData {
                rd_att: std::ptr::null_mut(),
                rd_id: oid,
            }
        }

        const ROUND_TRIP_LOCAL_STORE_ID: u32 = 0;

        fn make_store(rel: &mut pg_sys::RelationData) -> SpireRelationObjectStore {
            let store_relid = rel.rd_id;
            let store = SpireRelationObjectStore::for_store_relation_id(
                rel,
                ROUND_TRIP_LOCAL_STORE_ID,
                store_relid,
            );
            // SAFETY: relation is alive for the test; metadata block 0 must
            // exist before any object tuple insert.
            unsafe { page::initialize_aux_store_metadata_page(rel) };
            store
        }

        #[test]
        fn relation_store_insert_and_read_routing_object_round_trip() {
            let _serial = EMULATOR_LOCK.lock().unwrap();
            pg_sys::reset_counters();
            let mut rel = synth_relation(45001);
            let store = make_store(&mut rel);

            let routing =
                SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
            let placement = store.insert_routing_object(7, &routing).unwrap();
            assert_eq!(placement.epoch, 7);
            assert_eq!(placement.pid, 11);
            assert_eq!(placement.local_store_id, ROUND_TRIP_LOCAL_STORE_ID);

            // SAFETY: placement came from the same store; validated by the
            // store's own read path before tuple bytes are pinned for decode.
            let decoded = unsafe { store.read_routing_object(&placement) }.unwrap();
            assert_eq!(decoded.header.pid, 11);
            assert_eq!(decoded.header.object_version, 3);
            assert_eq!(decoded.children().count(), 2);

            // SAFETY: same placement; header dispatch must report routing kind.
            let header = unsafe { store.read_object_header(&placement) }.unwrap();
            assert_eq!(header.pid, 11);
            assert_eq!(header.kind, SpirePartitionObjectKind::Root);
        }

        #[test]
        fn relation_store_insert_and_read_leaf_v2_round_trip() {
            let _serial = EMULATOR_LOCK.lock().unwrap();
            pg_sys::reset_counters();
            let mut rel = synth_relation(45002);
            let store = make_store(&mut rel);

            let assignments = vec![leaf_v2_assignment(1, 8), leaf_v2_assignment(2, 8)];
            let placement = store
                .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &assignments)
                .unwrap();
            assert_eq!(placement.epoch, 7);
            assert_eq!(placement.pid, 17);

            // SAFETY: same placement; segment chain is walked under share locks.
            let decoded = unsafe { store.read_leaf_object_v2(&placement) }.unwrap();
            assert_eq!(decoded.meta.header.pid, 17);
            assert_eq!(decoded.meta.header.object_version, 3);
            assert_eq!(decoded.meta.header.parent_pid, 5);
            assert_eq!(decoded.meta.header.assignment_count, 2);
            assert_eq!(decoded.segments.len(), 1);

            // SAFETY: header dispatch over leaf V2 meta.
            let header = unsafe { store.read_object_header(&placement) }.unwrap();
            assert_eq!(header.pid, 17);
            assert_eq!(header.kind, SpirePartitionObjectKind::Leaf);
        }

        #[test]
        fn relation_store_insert_and_read_delta_object_round_trip() {
            let _serial = EMULATOR_LOCK.lock().unwrap();
            pg_sys::reset_counters();
            let mut rel = synth_relation(45003);
            let store = make_store(&mut rel);

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
            let placement = store.insert_delta_object(7, &delta).unwrap();
            assert_eq!(placement.epoch, 7);
            assert_eq!(placement.pid, 19);

            // SAFETY: placement came from the same store; single-tuple decode.
            let decoded = unsafe { store.read_delta_object(&placement) }.unwrap();
            assert_eq!(decoded.header.pid, 19);
            assert_eq!(decoded.header.parent_pid, 17);
            assert_eq!(decoded.assignments.len(), 1);

            // SAFETY: header dispatch over delta object.
            let header = unsafe { store.read_object_header(&placement) }.unwrap();
            assert_eq!(header.pid, 19);
            assert_eq!(header.kind, SpirePartitionObjectKind::Delta);
        }

        #[test]
        fn relation_store_insert_and_read_top_graph_round_trip() {
            let _serial = EMULATOR_LOCK.lock().unwrap();
            pg_sys::reset_counters();
            let mut rel = synth_relation(45004);
            let store = make_store(&mut rel);

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
            let placement = store.insert_top_graph_object(7, &top_graph).unwrap();
            assert_eq!(placement.epoch, 7);
            assert_eq!(placement.pid, 90);

            // SAFETY: same store; round-trip read path.
            let decoded = unsafe { store.read_top_graph_object(&placement) }.unwrap();
            assert_eq!(decoded.header.pid, 90);
            assert_eq!(decoded.nodes.len(), 1);

            // SAFETY: header dispatch over top-graph object.
            let header = unsafe { store.read_object_header(&placement) }.unwrap();
            assert_eq!(header.pid, 90);
            assert_eq!(header.kind, SpirePartitionObjectKind::TopGraph);
        }
    }
}

#[cfg(test)]
mod page_tests {
    //! Tests for `src/am/ec_spire/page.rs` via the careful shadow crate.
    //!
    //! Early-error paths return `Err` before any `pg_sys` page operation runs.
    //! Success-path round-trips exercise the Phase-1 backing-page emulator in
    //! `careful_pg_guards::pg_sys` (real `PageInit`, `PageAddItemExtended`,
    //! `PageGetItem`, etc., backed by per-relation `[u8; 8192]` pages).
    use super::page;
    use crate::careful_pg_guards::pg_sys;
    use crate::storage::page::{ItemPointer, FIRST_DATA_BLOCK_NUMBER, METADATA_BLOCK_NUMBER};
    use std::sync::Mutex;

    use super::meta::SpireRootControlState;

    // Page-emulator tests share thread-local registry state with the
    // pg_guards module's tests; serialize independently so cargo test's
    // thread pool reset is deterministic per emulator-aware test.
    static EMULATOR_LOCK: Mutex<()> = Mutex::new(());

    fn null_relation() -> pg_sys::Relation {
        std::ptr::null_mut()
    }

    fn synth_relation(oid: u32) -> pg_sys::RelationData {
        pg_sys::RelationData {
            rd_att: std::ptr::null_mut(),
            rd_id: oid,
        }
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

    #[test]
    fn initialize_and_read_root_control_round_trip_through_emulator() {
        let _serial = EMULATOR_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let mut rel = synth_relation(35001);
        let relation: pg_sys::Relation = &mut rel;

        let initial = SpireRootControlState::empty();
        // SAFETY: emulator-backed relation is alive for the call; the helper
        // allocates a fresh root/control block via P_NEW + RBM_ZERO_AND_LOCK
        // and writes the encoded root/control state into the special area.
        unsafe { page::initialize_root_control_page(relation, initial) };
        // SAFETY: same relation; share-lock path reads the special area back.
        let read_back = unsafe { page::read_root_control_page(relation) };
        assert_eq!(read_back, initial);
    }

    #[test]
    fn append_and_read_object_tuple_round_trip_through_emulator() {
        let _serial = EMULATOR_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let mut rel = synth_relation(35002);
        let relation: pg_sys::Relation = &mut rel;

        // SAFETY: emulator relation; metadata block 0 must exist before any
        // object tuple append.
        unsafe { page::initialize_root_control_page(relation, SpireRootControlState::empty()) };

        let payload = b"ec-spire round-trip payload";
        // SAFETY: relation has its metadata block; helper allocates a new
        // data block via P_NEW and appends the tuple under WAL.
        let tid = unsafe { page::append_object_tuple(relation, payload) }
            .expect("append should succeed on an initialized relation");
        assert!(tid.block_number >= FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(tid.offset_number, 1);

        // SAFETY: tid came from append above; read pins the data page under
        // a share lock and copies the tuple bytes out.
        let read_back = unsafe { page::read_object_tuple(relation, tid) }
            .expect("read should succeed for a valid tid");
        assert_eq!(read_back.as_slice(), payload);
    }

    #[test]
    fn scan_object_tuples_visits_every_appended_tuple_in_order() {
        let _serial = EMULATOR_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let mut rel = synth_relation(35003);
        let relation: pg_sys::Relation = &mut rel;

        // SAFETY: emulator relation; metadata + several object tuples appended
        // so the scan has multiple entries to visit.
        unsafe { page::initialize_root_control_page(relation, SpireRootControlState::empty()) };
        let payloads: [&[u8]; 3] = [b"first", b"second-payload", b"third value"];
        let mut tids = Vec::new();
        for payload in &payloads {
            // SAFETY: same emulator relation.
            let tid = unsafe { page::append_object_tuple(relation, payload) }.unwrap();
            tids.push(tid);
        }

        let mut visited: Vec<(ItemPointer, Vec<u8>)> = Vec::new();
        // SAFETY: same emulator relation; visitor copies bytes out under the
        // per-page share lock.
        unsafe {
            page::scan_object_tuples(relation, |tid, tuple| {
                visited.push((tid, tuple.to_vec()));
                Ok(())
            })
        }
        .expect("scan should succeed");

        assert_eq!(visited.len(), payloads.len());
        for ((expected_tid, expected_payload), (actual_tid, actual_payload)) in
            tids.iter().zip(payloads.iter()).zip(visited.iter())
        {
            // payload, tid each compared explicitly so failures point at the
            // wrong column.
            let (expected_tid, expected_payload) = (expected_tid, expected_payload);
            let (actual_tid, actual_payload) = (actual_tid, actual_payload);
            assert_eq!(actual_tid, expected_tid);
            assert_eq!(actual_payload.as_slice(), *expected_payload);
        }
    }

    #[test]
    fn rewrite_object_tuple_same_len_updates_payload_in_place() {
        let _serial = EMULATOR_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let mut rel = synth_relation(35004);
        let relation: pg_sys::Relation = &mut rel;

        // SAFETY: emulator relation; setup mirrors the prior round-trip test.
        unsafe { page::initialize_root_control_page(relation, SpireRootControlState::empty()) };
        let original = b"original-payload";
        let replacement = b"replaced-payload";
        // SAFETY: lengths chosen to match.
        let tid = unsafe { page::append_object_tuple(relation, original) }.unwrap();
        unsafe { page::rewrite_object_tuple_same_len(relation, tid, replacement) }
            .expect("same-length rewrite should succeed");
        let read_back = unsafe { page::read_object_tuple(relation, tid) }.unwrap();
        assert_eq!(read_back.as_slice(), replacement);

        // Length-mismatch rewrite must error and leave the tuple unchanged.
        let mismatched = b"too-short";
        let err = unsafe { page::rewrite_object_tuple_same_len(relation, tid, mismatched) }
            .expect_err("length mismatch must be rejected");
        assert!(err.contains("length changed"), "unexpected error: {err}");
        let read_after_failure = unsafe { page::read_object_tuple(relation, tid) }.unwrap();
        assert_eq!(read_after_failure.as_slice(), replacement);
    }

    #[test]
    fn delete_object_tuples_no_compact_removes_real_tuples_and_reports_bytes() {
        let _serial = EMULATOR_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let mut rel = synth_relation(35005);
        let relation: pg_sys::Relation = &mut rel;

        // SAFETY: emulator relation.
        unsafe { page::initialize_root_control_page(relation, SpireRootControlState::empty()) };
        let payloads: [&[u8]; 2] = [b"keep-me", b"delete-me-please"];
        let mut tids = Vec::new();
        for payload in &payloads {
            // SAFETY: same emulator relation.
            tids.push(unsafe { page::append_object_tuple(relation, payload) }.unwrap());
        }

        // SAFETY: delete the second tuple; first remains readable.
        let (count, bytes) =
            unsafe { page::delete_object_tuples_no_compact(relation, &tids[1..]) }
                .expect("delete should succeed");
        assert_eq!(count, 1);
        assert_eq!(bytes, payloads[1].len() as u64);

        // First tuple still readable.
        let still_there = unsafe { page::read_object_tuple(relation, tids[0]) }.unwrap();
        assert_eq!(still_there.as_slice(), payloads[0]);

        // Deleted tuple now reports an unused slot.
        let err = unsafe { page::read_object_tuple(relation, tids[1]) }
            .expect_err("deleted tuple must surface as unused");
        assert!(err.contains("unused slot"), "unexpected error: {err}");
    }
}
