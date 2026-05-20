use std::collections::HashSet;

use pgrx::pg_sys;

use crate::am::common::callback::pg_am_callback;

use super::assign::{
    build_boundary_insert_delta_assignment_placements_with_identity,
    build_primary_leaf_assignments_with_identity, SpireBoundaryLeafAssignmentIdentityInput,
    SpireLeafAssignmentIdentityInput, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::build::{
    self, encode_manifest_bundle_for_publish, object_manifest_from_placement_writes,
    root_control_state_for_publish, write_manifest_bundle_to_relation,
    write_placement_entries_to_relation, SpirePublishCoordinatorInput,
};
use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireLocalStoreConfig,
    SpirePlacementDirectory,
};
use super::storage::{
    SpireDeltaPartitionObject, SpireRelationObjectStore, SpireRelationObjectStoreSet,
    SpireRoutingChildEntry, SpireRoutingPartitionObject,
};
use super::{lock_publish_relation, options, page, scan};

pub(super) unsafe extern "C-unwind" fn ec_spire_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    index_info: *mut pg_sys::IndexInfo,
) -> bool {
    pg_am_callback!({
        crate::fault::maybe_fail_palloc("ec_spire aminsert entry");
        publish_insert_delta_epoch(
            index_relation,
            values,
            isnull,
            heap_tid,
            heap_relation,
            index_info,
        )
        .unwrap_or_else(|e| pgrx::error!("ec_spire aminsert failed: {e}"));
        true
    })
}

unsafe fn publish_insert_delta_epoch(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> Result<(), String> {
    // SAFETY: aminsert supplies a live SPIRE index relation; the guard only
    // captures and locks its OID while this publish path runs.
    let _guard = unsafe { lock_publish_relation(index_relation) };
    // SAFETY: the locked SPIRE index relation has a root/control page.
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    // SAFETY: relation options are read from the live index relation.
    let relation_options = unsafe { options::relation_options(index_relation) };
    // SAFETY: heap_relation and IndexInfo come from PostgreSQL's aminsert
    // callback and remain valid while deriving the indexed tuple layout.
    let tuple_layout = unsafe {
        build::resolve_indexed_tuple_layout(
            heap_relation,
            index_info,
            &relation_options,
            "aminsert",
        )
    };
    // SAFETY: heap_tid is the non-null item pointer supplied to aminsert.
    let heap_tid = unsafe { build::decode_heap_tid(heap_tid, "aminsert") };
    // SAFETY: values/isnull are aminsert tuple arrays and tuple_layout was
    // derived from the matching live heap/index metadata.
    let tuple = unsafe {
        build::build_spire_index_tuple(
            values,
            isnull,
            heap_tid,
            tuple_layout,
            relation_options.assignment_payload_format(),
            "aminsert",
        )
    };

    if root_control.active_epoch == 0 {
        // SAFETY: the publish lock is held and tuple/root_control were derived
        // from the same live index relation.
        return unsafe {
            publish_empty_insert_bootstrap_epoch(index_relation, root_control, tuple)
        };
    }

    // SAFETY: the live index relation and freshly read root control identify
    // the local-store config for the active SPIRE epoch.
    let local_store_config =
        unsafe { scan::load_relation_local_store_config(index_relation, root_control)? };
    // SAFETY: the live index relation and root control identify the active
    // manifest tuple locators for this publish operation.
    let (active_epoch_manifest, object_manifest, placement_directory) =
        unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
    let active_snapshot = super::meta::SpirePublishedEpochSnapshot::new(
        &active_epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let active_lookup = super::meta::SpireValidatedEpochSnapshot::from_snapshot(active_snapshot)?;
    // SAFETY: the live SPIRE index relation and active local-store config open
    // the relation-backed stores needed for delta object writes.
    let mut store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_config(
            index_relation,
            local_store_config.clone(),
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )?
    };
    let boundary_replica_count = u32::try_from(relation_options.boundary_replica_count)
        .map_err(|_| "ec_spire boundary_replica_count reloption must be non-negative".to_owned())?;
    let nprobe = boundary_replica_count
        .checked_add(1)
        .ok_or_else(|| "ec_spire insert boundary fanout overflow".to_owned())?;
    // Insert routing intentionally uses the recursive centroid router even when a
    // top graph is present; graph-aware insert routing is a separate maintenance
    // decision from graph-assisted scan routing.
    let routed = scan::collect_snapshot_routed_probe_leaf_rows(
        &active_snapshot,
        &store,
        &tuple.source_vector,
        nprobe,
    )?;
    let mut seen_leaf_pids = HashSet::new();
    let routed_leaf_pids = routed
        .iter()
        .filter_map(|route| {
            if seen_leaf_pids.insert(route.leaf_pid) {
                Some(route.leaf_pid)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let primary_leaf_pid = routed_leaf_pids
        .first()
        .copied()
        .ok_or_else(|| "ec_spire insert routed no leaf pids".to_owned())?;
    let replica_leaf_pids = routed_leaf_pids.iter().skip(1).copied().collect::<Vec<_>>();
    let new_epoch = root_control
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire insert epoch overflow".to_owned())?;
    let (published_at_micros, retain_until_micros) = build::current_epoch_publish_times()?;

    let mut pid_allocator = SpirePidAllocator::new(root_control.next_pid)?;
    let mut local_vec_id_allocator =
        SpireLocalVecIdAllocator::new(root_control.next_local_vec_seq)?;
    let assignment_placements = build_boundary_insert_delta_assignment_placements_with_identity(
        &mut local_vec_id_allocator,
        vec![SpireBoundaryLeafAssignmentIdentityInput {
            primary_pid: primary_leaf_pid,
            replica_pids: replica_leaf_pids,
            assignment: SpireLeafAssignmentIdentityInput {
                assignment: tuple.assignment,
                vec_id_source_identity: tuple.vec_id_source_identity,
            },
        }],
    )?;

    let mut placement_entries = placement_directory
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = new_epoch;
            entry
        })
        .collect::<Vec<_>>();
    for assignment_placement in assignment_placements {
        let base_lookup =
            active_lookup.require_lookup(assignment_placement.pid, "insert delta base leaf")?;
        let delta_pid = pid_allocator.allocate()?;
        let delta_object = SpireDeltaPartitionObject::new(
            delta_pid,
            new_epoch,
            assignment_placement.pid,
            vec![assignment_placement.row],
        )?;
        placement_entries.push(store.insert_delta_object_for_base_placement(
            new_epoch,
            base_lookup.placement,
            &delta_object,
        )?);
    }
    let placement_directory = SpirePlacementDirectory::from_entries(new_epoch, placement_entries)?;
    let placement_evidence =
        // SAFETY: publish lock is held and placement entries are validated for
        // appending to the live SPIRE index relation.
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        new_epoch,
        &placement_directory,
        &placement_evidence,
    )?;
    let epoch_manifest = SpireEpochManifest {
        epoch: new_epoch,
        state: SpireEpochState::Published,
        consistency_mode: active_epoch_manifest.consistency_mode,
        published_at_micros,
        retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let input = SpirePublishCoordinatorInput {
        epoch_manifest: &epoch_manifest,
        object_manifest: &object_manifest,
        placement_directory: &placement_directory,
        local_store_config,
        next_pid: pid_allocator.next_pid(),
        next_local_vec_seq: local_vec_id_allocator.next_local_vec_seq(),
    };
    // SAFETY: publish lock is held; replacement publish appends manifests and
    // updates the root/control page of the live index relation.
    unsafe {
        build::publish_replacement_epoch_to_relation(index_relation, active_epoch_manifest, input)
    }
}

unsafe fn publish_empty_insert_bootstrap_epoch(
    index_relation: pg_sys::Relation,
    root_control: super::meta::SpireRootControlState,
    tuple: build::SpireBuildTuple,
) -> Result<(), String> {
    let new_epoch = root_control
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire insert bootstrap epoch overflow".to_owned())?;
    let (published_at_micros, retain_until_micros) = build::current_epoch_publish_times()?;

    let mut pid_allocator = SpirePidAllocator::new(root_control.next_pid)?;
    let mut local_vec_id_allocator =
        SpireLocalVecIdAllocator::new(root_control.next_local_vec_seq)?;
    let root_pid = pid_allocator.allocate()?;
    let leaf_pid = pid_allocator.allocate()?;
    let assignments = build_primary_leaf_assignments_with_identity(
        &mut local_vec_id_allocator,
        vec![SpireLeafAssignmentIdentityInput {
            assignment: tuple.assignment,
            vec_id_source_identity: tuple.vec_id_source_identity,
        }],
    )?;

    let routing_object = SpireRoutingPartitionObject::root(
        root_pid,
        build::SPIRE_INITIAL_OBJECT_VERSION,
        tuple.dimensions,
        vec![SpireRoutingChildEntry {
            centroid_index: 0,
            child_pid: leaf_pid,
            centroid: tuple.source_vector,
        }],
    )?;

    // SAFETY: the live SPIRE index relation identifies the relation-backed
    // object store used for the bootstrap epoch.
    let store = unsafe { SpireRelationObjectStore::for_index_relation(index_relation)? };
    // SAFETY: index_relation remains live while reading its relcache OID.
    let index_oid = unsafe { (*index_relation).rd_id };
    // SAFETY: index_relation remains live while reading its relcache tablespace.
    let tablespace = unsafe { (*(*index_relation).rd_rel).reltablespace };
    let local_store_config =
        SpireLocalStoreConfig::embedded_single_store(index_oid.into(), tablespace.into())?;
    let placements = vec![
        store.insert_routing_object(new_epoch, &routing_object)?,
        store.insert_leaf_object_v2_from_rows(
            new_epoch,
            leaf_pid,
            build::SPIRE_INITIAL_OBJECT_VERSION,
            root_pid,
            &assignments,
        )?,
    ];
    let placement_directory = SpirePlacementDirectory::from_entries(new_epoch, placements)?;
    let placement_evidence =
        // SAFETY: publish lock is held and bootstrap placement entries are
        // validated for appending to the live SPIRE index relation.
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        new_epoch,
        &placement_directory,
        &placement_evidence,
    )?;
    let epoch_manifest = SpireEpochManifest {
        epoch: new_epoch,
        state: SpireEpochState::Published,
        consistency_mode: SpireConsistencyMode::Strict,
        published_at_micros,
        retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let input = SpirePublishCoordinatorInput {
        epoch_manifest: &epoch_manifest,
        object_manifest: &object_manifest,
        placement_directory: &placement_directory,
        local_store_config,
        next_pid: pid_allocator.next_pid(),
        next_local_vec_seq: local_vec_id_allocator.next_local_vec_seq(),
    };
    let manifests = encode_manifest_bundle_for_publish(input.clone())?;
    // SAFETY: publish lock is held and the encoded manifest bundle belongs to
    // this live SPIRE index relation.
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    // SAFETY: publish lock is held while initializing the root/control page.
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(())
}
