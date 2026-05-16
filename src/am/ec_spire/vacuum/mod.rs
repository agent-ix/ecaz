use std::collections::{HashMap, HashSet};
use std::ffi::c_void;

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::assign::{
    build_delete_delta_assignments, SpireDeleteDeltaInput, SpireLocalVecIdAllocator,
    SpirePidAllocator,
};
use super::build::{
    self, object_manifest_from_placement_writes, write_placement_entries_to_relation,
    SpirePublishCoordinatorInput,
};
use super::meta::{SpireEpochManifest, SpireEpochState, SpirePlacementDirectory};
use super::storage::{
    is_delete_delta_assignment, is_visible_primary_assignment, SpireDeltaPartitionObject,
    SpireLeafAssignmentRow, SpireObjectReader, SpirePartitionObjectKind,
    SpireRelationObjectStoreSet, SpireVecId, SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
};
use super::{lock_publish_relation, page, scan};
use crate::storage::page::ItemPointer;

type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

#[derive(Debug, Clone, PartialEq)]
struct VacuumVisibleAssignment {
    base_pid: u64,
    assignment: SpireLeafAssignmentRow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VacuumDeleteResult {
    removed_assignments: u64,
    live_assignments: u64,
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_spire ambulkdelete requires vacuum info")
            }
            let index_relation = (*info).index;
            let Some(callback) = callback else {
                let live_count = collect_live_assignment_count(index_relation)
                    .unwrap_or_else(|e| pgrx::error!("ec_spire vacuum stats failed: {e}"));
                return finish_vacuum_stats(index_relation, stats, live_count, 0);
            };

            let result = run_bulkdelete(index_relation, callback, callback_state)
                .unwrap_or_else(|e| pgrx::error!("ec_spire ambulkdelete failed: {e}"));
            finish_vacuum_stats(
                index_relation,
                stats,
                result.live_assignments,
                result.removed_assignments,
            )
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_spire amvacuumcleanup requires vacuum info")
            }
            let index_relation = (*info).index;
            let live_count = run_vacuum_cleanup(index_relation)
                .unwrap_or_else(|e| pgrx::error!("ec_spire vacuum cleanup stats failed: {e}"));
            finish_vacuum_stats(index_relation, stats, live_count, 0)
        })
    }
}

unsafe fn run_vacuum_cleanup(index_relation: pg_sys::Relation) -> Result<u64, String> {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return Ok(0);
    }
    unsafe { publish_compacted_delta_epoch_if_needed(index_relation, root_control)? };
    collect_live_assignment_count(index_relation)
}

unsafe fn run_bulkdelete(
    index_relation: pg_sys::Relation,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<VacuumDeleteResult, String> {
    let _guard = unsafe { lock_publish_relation(index_relation) };
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return Ok(VacuumDeleteResult {
            removed_assignments: 0,
            live_assignments: 0,
        });
    }

    let (active_epoch_manifest, object_manifest, placement_directory) =
        unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
    let active_snapshot = super::meta::SpirePublishedEpochSnapshot::new(
        &active_epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_placements(
            index_relation,
            &placement_directory,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )?
    };
    let visible = collect_visible_assignments(&active_snapshot, &store)?;
    let mut deletes_by_base_pid: HashMap<u64, Vec<SpireDeleteDeltaInput>> = HashMap::new();
    for assignment in &visible {
        if unsafe { heap_tid_is_dead(assignment.assignment.heap_tid, callback, callback_state) } {
            deletes_by_base_pid
                .entry(assignment.base_pid)
                .or_default()
                .push(SpireDeleteDeltaInput {
                    vec_id: assignment.assignment.vec_id.clone(),
                    heap_tid: assignment.assignment.heap_tid,
                });
        }
    }

    let removed_assignments =
        deletes_by_base_pid
            .values()
            .map(Vec::len)
            .try_fold(0_u64, |acc, len| {
                acc.checked_add(
                    u64::try_from(len)
                        .map_err(|_| "ec_spire vacuum delete count exceeds u64".to_owned())?,
                )
                .ok_or_else(|| "ec_spire vacuum delete count overflow".to_owned())
            })?;
    let live_assignments = u64::try_from(visible.len())
        .map_err(|_| "ec_spire vacuum live assignment count exceeds u64".to_owned())?
        .saturating_sub(removed_assignments);
    if removed_assignments == 0 {
        return Ok(VacuumDeleteResult {
            removed_assignments,
            live_assignments,
        });
    }

    publish_delete_delta_epoch(
        index_relation,
        root_control,
        active_epoch_manifest,
        placement_directory,
        deletes_by_base_pid,
    )?;
    Ok(VacuumDeleteResult {
        removed_assignments,
        live_assignments,
    })
}

fn collect_live_assignment_count(index_relation: pg_sys::Relation) -> Result<u64, String> {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return Ok(0);
    }
    let (epoch_manifest, object_manifest, placement_directory) =
        unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
    let snapshot = super::meta::SpirePublishedEpochSnapshot::new(
        &epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_placements(
            index_relation,
            &placement_directory,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )?
    };
    let visible = collect_visible_assignments(&snapshot, &store)?;
    u64::try_from(visible.len())
        .map_err(|_| "ec_spire vacuum live assignment count exceeds u64".to_owned())
}

fn collect_visible_assignments(
    snapshot: &super::meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<VacuumVisibleAssignment>, String> {
    let snapshot = super::meta::SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let deleted_by_base_pid = collect_delete_vec_ids_by_base_pid(&snapshot, object_store)?;
    let mut visible = Vec::new();

    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "vacuum visible assignment")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Leaf => {
                let deleted = deleted_by_base_pid.get(&manifest_entry.pid);
                for assignment in read_leaf_assignments(object_store, placement)? {
                    if !is_visible_primary_assignment(&assignment) {
                        continue;
                    }
                    if deleted.is_some_and(|deleted| deleted.contains(&assignment.vec_id)) {
                        continue;
                    }
                    visible.push(VacuumVisibleAssignment {
                        base_pid: manifest_entry.pid,
                        assignment,
                    });
                }
            }
            SpirePartitionObjectKind::Delta => {
                let deleted = deleted_by_base_pid.get(&header.parent_pid);
                let delta = object_store.read_delta_object(placement)?;
                for assignment in delta.assignments {
                    if !is_visible_primary_assignment(&assignment) {
                        continue;
                    }
                    if deleted.is_some_and(|deleted| deleted.contains(&assignment.vec_id)) {
                        continue;
                    }
                    visible.push(VacuumVisibleAssignment {
                        base_pid: header.parent_pid,
                        assignment,
                    });
                }
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::TopGraph => {}
        }
    }

    Ok(visible)
}

unsafe fn publish_compacted_delta_epoch_if_needed(
    index_relation: pg_sys::Relation,
    root_control: super::meta::SpireRootControlState,
) -> Result<bool, String> {
    let (active_epoch_manifest, object_manifest, placement_directory) =
        unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
    let local_store_config =
        unsafe { scan::load_relation_local_store_config(index_relation, root_control)? };
    let active_snapshot = super::meta::SpirePublishedEpochSnapshot::new(
        &active_epoch_manifest,
        &object_manifest,
        &placement_directory,
    )?;
    let snapshot = super::meta::SpireValidatedEpochSnapshot::from_snapshot(active_snapshot)?;
    let mut store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_config(
            index_relation,
            local_store_config.clone(),
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )?
    };

    let mut affected_base_pids = HashSet::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "vacuum compaction delta")?;
        let header = store.read_object_header(lookup.placement)?;
        if header.kind == SpirePartitionObjectKind::Delta {
            affected_base_pids.insert(header.parent_pid);
        }
    }
    if affected_base_pids.is_empty() {
        return Ok(false);
    }

    let mut compact_rows_by_base_pid: HashMap<u64, Vec<SpireLeafAssignmentRow>> = HashMap::new();
    for visible in collect_visible_assignments(&active_snapshot, &store)? {
        if !affected_base_pids.contains(&visible.base_pid) {
            continue;
        }
        let mut assignment = visible.assignment;
        assignment.flags &= !SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
        compact_rows_by_base_pid
            .entry(visible.base_pid)
            .or_default()
            .push(assignment);
    }

    let new_epoch = root_control
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire vacuum compaction epoch overflow".to_owned())?;
    let (published_at_micros, retain_until_micros) =
        unsafe { build::current_epoch_publish_times()? };
    let pid_allocator = SpirePidAllocator::new(root_control.next_pid)?;
    let local_vec_id_allocator = SpireLocalVecIdAllocator::new(root_control.next_local_vec_seq)?;

    let mut placement_entries = Vec::with_capacity(placement_directory.entries.len());
    let mut compacted_base_pids = HashSet::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "vacuum compaction object")?;
        let placement = lookup.placement;
        let header = store.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Delta => {}
            SpirePartitionObjectKind::Leaf if affected_base_pids.contains(&manifest_entry.pid) => {
                let leaf_pid = require_compaction_leaf_pid_match(manifest_entry.pid, header.pid)?;
                let leaf_object_version = require_compaction_leaf_object_version_match(
                    manifest_entry.object_version,
                    header.object_version,
                    leaf_pid,
                )?;
                let rows = compact_rows_by_base_pid
                    .remove(&leaf_pid)
                    .unwrap_or_default();
                // Compaction normalizes rewritten base leaves into the V2 segment format.
                let object_version = leaf_object_version.checked_add(1).ok_or_else(|| {
                    format!(
                        "ec_spire vacuum compaction object version overflow for pid {}",
                        leaf_pid
                    )
                })?;
                placement_entries.push(store.insert_leaf_object_v2_from_rows(
                    new_epoch,
                    leaf_pid,
                    object_version,
                    header.parent_pid,
                    &rows,
                )?);
                compacted_base_pids.insert(leaf_pid);
            }
            SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::Leaf
            | SpirePartitionObjectKind::TopGraph => {
                // TODO(phase6): invalidate or rebuild top graphs when compaction
                // starts rewriting routing centroids rather than only leaf rows.
                let mut carried = placement.clone();
                carried.epoch = new_epoch;
                placement_entries.push(carried);
            }
        }
    }

    if compacted_base_pids != affected_base_pids {
        let missing = affected_base_pids
            .difference(&compacted_base_pids)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "ec_spire vacuum compaction delta parent pids do not all reference active leaves: {missing:?}"
        ));
    }
    if !compact_rows_by_base_pid.is_empty() {
        let leftover = compact_rows_by_base_pid.keys().copied().collect::<Vec<_>>();
        return Err(format!(
            "ec_spire vacuum compaction had leftover rows for base pids: {leftover:?}"
        ));
    }

    let placement_directory = SpirePlacementDirectory::from_entries(new_epoch, placement_entries)?;
    let placement_evidence =
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
    unsafe {
        build::publish_replacement_epoch_to_relation(index_relation, active_epoch_manifest, input)?;
    }
    Ok(true)
}

fn require_compaction_leaf_pid_match(manifest_pid: u64, header_pid: u64) -> Result<u64, String> {
    if manifest_pid != header_pid {
        return Err(format!(
            "ec_spire vacuum compaction leaf pid mismatch: manifest pid {manifest_pid}, object header pid {header_pid}"
        ));
    }
    Ok(manifest_pid)
}

fn require_compaction_leaf_object_version_match(
    manifest_object_version: u64,
    header_object_version: u64,
    leaf_pid: u64,
) -> Result<u64, String> {
    if manifest_object_version != header_object_version {
        return Err(format!(
            "ec_spire vacuum compaction leaf object_version mismatch for pid {leaf_pid}: manifest object_version {manifest_object_version}, object header object_version {header_object_version}"
        ));
    }
    Ok(manifest_object_version)
}

fn collect_delete_vec_ids_by_base_pid(
    snapshot: &super::meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<HashMap<u64, HashSet<SpireVecId>>, String> {
    let mut deleted_by_base_pid: HashMap<u64, HashSet<SpireVecId>> = HashMap::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "vacuum delete assignment")?;
        let placement = lookup.placement;
        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }
        let delta = object_store.read_delta_object(placement)?;
        for assignment in delta.assignments {
            if is_delete_delta_assignment(&assignment) {
                deleted_by_base_pid
                    .entry(header.parent_pid)
                    .or_default()
                    .insert(assignment.vec_id);
            }
        }
    }
    Ok(deleted_by_base_pid)
}

fn read_leaf_assignments(
    object_store: &impl SpireObjectReader,
    placement: &super::meta::SpirePlacementEntry,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    match object_store.read_leaf_object(placement) {
        Ok(object) => Ok(object.assignments),
        Err(v1_error) => object_store
            .read_leaf_object_v2(placement)
            .map_err(|v2_error| {
                format!(
                    "ec_spire vacuum could not read leaf pid {} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}",
                    placement.pid
                )
            })?
            .assignment_rows(),
    }
}

fn require_base_placement(
    placement_directory: &SpirePlacementDirectory,
    base_pid: u64,
) -> Result<&super::meta::SpirePlacementEntry, String> {
    placement_directory
        .entries
        .iter()
        .find(|entry| entry.pid == base_pid)
        .ok_or_else(|| format!("ec_spire vacuum missing base placement for pid {base_pid}"))
}

fn publish_delete_delta_epoch(
    index_relation: pg_sys::Relation,
    root_control: super::meta::SpireRootControlState,
    active_epoch_manifest: SpireEpochManifest,
    placement_directory: SpirePlacementDirectory,
    deletes_by_base_pid: HashMap<u64, Vec<SpireDeleteDeltaInput>>,
) -> Result<(), String> {
    let new_epoch = root_control
        .active_epoch
        .checked_add(1)
        .ok_or_else(|| "ec_spire vacuum epoch overflow".to_owned())?;
    let (published_at_micros, retain_until_micros) =
        unsafe { build::current_epoch_publish_times()? };
    let mut pid_allocator = SpirePidAllocator::new(root_control.next_pid)?;
    let local_vec_id_allocator = SpireLocalVecIdAllocator::new(root_control.next_local_vec_seq)?;
    let local_store_config =
        unsafe { scan::load_relation_local_store_config(index_relation, root_control)? };
    let mut store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_config(
            index_relation,
            local_store_config.clone(),
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )?
    };

    let mut placement_entries = placement_directory
        .entries
        .iter()
        .cloned()
        .map(|mut entry| {
            entry.epoch = new_epoch;
            entry
        })
        .collect::<Vec<_>>();
    let mut groups = deletes_by_base_pid.into_iter().collect::<Vec<_>>();
    groups.sort_by_key(|(base_pid, _)| *base_pid);
    for (base_pid, deletes) in groups {
        let delta_pid = pid_allocator.allocate()?;
        let assignments = build_delete_delta_assignments(deletes)?;
        let delta_object =
            SpireDeltaPartitionObject::new(delta_pid, new_epoch, base_pid, assignments)?;
        let base_placement = require_base_placement(&placement_directory, base_pid)?;
        placement_entries.push(store.insert_delta_object_for_base_placement(
            new_epoch,
            base_placement,
            &delta_object,
        )?);
    }

    let placement_directory = SpirePlacementDirectory::from_entries(new_epoch, placement_entries)?;
    let placement_evidence =
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
    unsafe {
        build::publish_replacement_epoch_to_relation(index_relation, active_epoch_manifest, input)?;
    }
    Ok(())
}

unsafe fn finish_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    live_assignments: u64,
    removed_assignments: u64,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = live_assignments as f64;
        (*stats).tuples_removed += removed_assignments as f64;
    }
    stats
}

unsafe fn heap_tid_is_dead(
    heap_tid: ItemPointer,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> bool {
    let mut tid = pg_sys::ItemPointerData::default();
    item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    unsafe { callback((&mut tid) as pg_sys::ItemPointer, callback_state) }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Default)]
struct DebugVacuumCallbackState {
    dead_tids: HashSet<ItemPointer>,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe extern "C-unwind" fn debug_vacuum_dead_tid_callback(
    itemptr: pg_sys::ItemPointer,
    state: *mut c_void,
) -> bool {
    let state = unsafe { &*(state.cast::<DebugVacuumCallbackState>()) };
    state
        .dead_tids
        .contains(&unsafe { super::build::decode_heap_tid(itemptr, "debug vacuum") })
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_vacuum_remove_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[ItemPointer],
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation = unsafe {
        pg_sys::index_open(
            index_oid,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
    let mut callback_state = DebugVacuumCallbackState {
        dead_tids: dead_tids.iter().copied().collect(),
    };

    let stats = unsafe {
        ec_spire_ambulkdelete(
            info_ptr,
            std::ptr::null_mut(),
            Some(debug_vacuum_dead_tid_callback),
            (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
        )
    };
    let stats = unsafe { ec_spire_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };
    unsafe {
        pg_sys::index_close(
            index_relation,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_vacuum_bulkdelete_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[ItemPointer],
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation = unsafe {
        pg_sys::index_open(
            index_oid,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
    let mut callback_state = DebugVacuumCallbackState {
        dead_tids: dead_tids.iter().copied().collect(),
    };

    let stats = unsafe {
        ec_spire_ambulkdelete(
            info_ptr,
            std::ptr::null_mut(),
            Some(debug_vacuum_dead_tid_callback),
            (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
        )
    };
    let result = unsafe { *stats };
    unsafe {
        pg_sys::index_close(
            index_relation,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    result
}

include!("tests.rs");
