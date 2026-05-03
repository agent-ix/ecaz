//! ec_spire access-method scaffold.

mod assign;
mod build;
mod cost;
mod diagnostics;
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

#[cfg(any(test, feature = "pg_test"))]
use pgrx::pg_sys;

pub(super) const EC_SPIRE_DEFAULT_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MIN_NLISTS: i32 = 0;
pub(super) const EC_SPIRE_MAX_NLISTS: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MIN_NPROBE: i32 = 0;
pub(super) const EC_SPIRE_MAX_NPROBE: i32 = 1_000_000;
pub(super) const EC_SPIRE_DEFAULT_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MIN_RERANK_WIDTH: i32 = 0;
pub(super) const EC_SPIRE_MAX_RERANK_WIDTH: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS: i32 = 10_000_000;
pub(super) const EC_SPIRE_DEFAULT_SEED: i32 = 42;
pub(super) const EC_SPIRE_MIN_SEED: i32 = 0;
pub(super) const EC_SPIRE_MAX_SEED: i32 = i32::MAX;
pub(super) const EC_SPIRE_DEFAULT_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MIN_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_SPIRE_MAX_PQ_GROUP_SIZE: i32 = 32;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_spire {callback} is not implemented yet")
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_object_tuple_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u64, u32, u64, u64, u32, u64) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u32, u16, u64, u32, u64, u64, u32, u64), String> {
        let store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let object = storage::SpireRoutingPartitionObject::root(
            10,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 11,
                centroid: vec![1.0, 0.0],
            }],
        )?;

        let placement = unsafe { store.insert_routing_object(1, &object)? };
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let decoded = unsafe { store.read_routing_object(&placement)? };
        let child = decoded
            .children()
            .next()
            .ok_or_else(|| "ec_spire debug routing object lost its child".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            root_control.active_epoch,
            placement.store_relid,
            decoded.header.pid,
            decoded.header.object_version,
            decoded.header.child_count,
            child.child_pid,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_relation_leaf_v2_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u32, u16, u32, u32, u64, u32) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u32, u16, u32, u32, u64, u32), String> {
        let store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let assignments = vec![
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(1),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 42,
                    offset_number: 1,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3, 4],
            },
            storage::SpireLeafAssignmentRow {
                flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: storage::SpireVecId::local(2),
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: 43,
                    offset_number: 2,
                },
                payload_format: storage::SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
                gamma: 0.75,
                encoded_payload: vec![5, 6, 7, 8],
            },
        ];

        let placement =
            unsafe { store.insert_leaf_object_v2_from_rows(1, 20, 1, 10, &assignments)? };
        let leaf = unsafe { store.read_leaf_object_v2(&placement)? };
        let rows = leaf.assignment_rows()?;
        let first_row = rows
            .first()
            .ok_or_else(|| "ec_spire debug leaf V2 lost its first row".to_owned())?;

        Ok((
            placement.object_tid.block_number,
            placement.object_tid.offset_number,
            leaf.meta.header.assignment_count,
            leaf.meta.segment_count,
            first_row
                .vec_id
                .local_sequence()
                .ok_or_else(|| "ec_spire debug leaf V2 first row lost local vec_id".to_owned())?,
            first_row.heap_tid.block_number,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_empty_manifest_publish_roundtrip(
    index_oid: pg_sys::Oid,
) -> (u64, u64, u64, u32, u16, u32, u16, u32, u16) {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<(u64, u64, u64, u32, u16, u32, u16, u32, u16), String> {
        let epoch_manifest = meta::SpireEpochManifest {
            epoch: 1,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 600_000_001,
            active_query_count: 0,
        };
        let object_manifest = meta::SpireObjectManifest::from_entries(1, Vec::new())?;
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, Vec::new())?;
        let input = build::SpirePublishCoordinatorInput {
            epoch_manifest: &epoch_manifest,
            object_manifest: &object_manifest,
            placement_directory: &placement_directory,
            next_pid: assign::SPIRE_FIRST_PID,
            next_local_vec_seq: assign::SPIRE_FIRST_LOCAL_VEC_SEQ,
        };
        let manifests = build::encode_manifest_bundle_for_publish(input)?;
        let locators =
            unsafe { build::write_manifest_bundle_to_relation(index_relation, &manifests)? };
        let root_control = build::root_control_state_for_publish(input, locators)?;
        unsafe { page::initialize_root_control_page(index_relation, root_control) };
        let persisted = unsafe { page::read_root_control_page(index_relation) };

        Ok((
            persisted.active_epoch,
            persisted.next_pid,
            persisted.next_local_vec_seq,
            persisted.epoch_manifest_tid.block_number,
            persisted.epoch_manifest_tid.offset_number,
            persisted.object_manifest_tid.block_number,
            persisted.object_manifest_tid.offset_number,
            persisted.placement_directory_tid.block_number,
            persisted.placement_directory_tid.offset_number,
        ))
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}
