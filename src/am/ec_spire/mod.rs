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

use pgrx::pg_sys;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::vacuum::debug_spire_vacuum_remove_heap_tids;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireActiveSnapshotDiagnostics {
    pub(crate) active_epoch: u64,
    pub(crate) next_pid: u64,
    pub(crate) next_local_vec_seq: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) object_count: u64,
    pub(crate) placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) internal_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) delta_assignment_count: u64,
    pub(crate) available_object_bytes: u64,
    pub(crate) routing_object_bytes: u64,
    pub(crate) leaf_object_bytes: u64,
    pub(crate) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexOptionsSnapshot {
    pub(crate) nlists: i32,
    pub(crate) active_leaf_count: u32,
    pub(crate) relation_nprobe: i32,
    pub(crate) session_nprobe: Option<i32>,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) relation_rerank_width: i32,
    pub(crate) session_rerank_width: Option<i32>,
    pub(crate) effective_rerank_width: i32,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) training_sample_rows: i32,
    pub(crate) seed: i32,
    pub(crate) pq_group_size: i32,
    pub(crate) storage_format: &'static str,
    pub(crate) assignment_payload_format: &'static str,
    pub(crate) assignment_payload_scannable: bool,
    pub(crate) assignment_payload_status: &'static str,
    pub(crate) assignment_payload_recommendation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexHealthSnapshot {
    pub(crate) active_epoch: u64,
    pub(crate) consistency_mode: &'static str,
    pub(crate) status: &'static str,
    pub(crate) healthy: bool,
    pub(crate) recommendation: &'static str,
    pub(crate) compaction_recommended: bool,
    pub(crate) object_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) delta_assignment_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexPlacementSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) placement_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) stale_placement_count: u64,
    pub(crate) unavailable_placement_count: u64,
    pub(crate) skipped_placement_count: u64,
    pub(crate) object_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) internal_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) delta_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) assignment_count: u64,
    pub(crate) placement_object_bytes: u64,
    pub(crate) available_object_bytes: u64,
    pub(crate) routing_object_bytes: u64,
    pub(crate) leaf_object_bytes: u64,
    pub(crate) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexScanPlacementSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) effective_rerank_width: u64,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) node_id: u32,
    pub(crate) local_store_id: u32,
    pub(crate) scanned_pid_count: u64,
    pub(crate) leaf_pid_count: u64,
    pub(crate) delta_pid_count: u64,
    pub(crate) candidate_row_count: u64,
    pub(crate) leaf_candidate_row_count: u64,
    pub(crate) delta_candidate_row_count: u64,
    pub(crate) delete_delta_row_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireIndexRootRoutingSnapshotRow {
    pub(crate) active_epoch: u64,
    pub(crate) root_pid: u64,
    pub(crate) root_object_version: u64,
    pub(crate) root_level: u16,
    pub(crate) root_child_count: u64,
    pub(crate) centroid_dimensions: u16,
    pub(crate) centroid_index: u32,
    pub(crate) child_pid: u64,
    pub(crate) child_kind: &'static str,
    pub(crate) child_object_version: u64,
    pub(crate) child_level: u16,
    pub(crate) child_parent_pid: u64,
    pub(crate) child_assignment_count: u64,
    pub(crate) child_node_id: u32,
    pub(crate) child_local_store_id: u32,
    pub(crate) child_store_relid: u32,
    pub(crate) child_placement_state: &'static str,
    pub(crate) child_object_bytes: u64,
}

impl SpireActiveSnapshotDiagnostics {
    fn empty(root_control: meta::SpireRootControlState) -> Self {
        Self {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: "none",
            object_count: 0,
            placement_count: 0,
            local_store_count: 0,
            available_placement_count: 0,
            stale_placement_count: 0,
            unavailable_placement_count: 0,
            skipped_placement_count: 0,
            root_object_count: 0,
            internal_object_count: 0,
            leaf_object_count: 0,
            delta_object_count: 0,
            routing_child_count: 0,
            leaf_assignment_count: 0,
            delta_assignment_count: 0,
            available_object_bytes: 0,
            routing_object_bytes: 0,
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        }
    }
}

fn health_snapshot_from_diagnostics(
    diagnostics: &SpireActiveSnapshotDiagnostics,
) -> SpireIndexHealthSnapshot {
    let has_no_active_epoch = diagnostics.active_epoch == 0;
    let (status, healthy, recommendation, compaction_recommended) = if has_no_active_epoch {
        (
            "empty",
            true,
            "build or insert rows to publish the first SPIRE epoch",
            false,
        )
    } else if diagnostics.unavailable_placement_count > 0 {
        (
            "unavailable_placements",
            false,
            "restore unavailable local placements before relying on this index",
            false,
        )
    } else if diagnostics.stale_placement_count > 0 {
        (
            "stale_placements",
            false,
            "publish a cleanup epoch to remove stale placements",
            false,
        )
    } else if diagnostics.skipped_placement_count > 0 {
        (
            "skipped_placements",
            false,
            "inspect skipped placements before enabling degraded reads",
            false,
        )
    } else if diagnostics.delta_object_count > 0 {
        (
            "maintenance_recommended",
            true,
            "run VACUUM to compact active delta objects into V2 base leaves",
            true,
        )
    } else if diagnostics.consistency_mode == "degraded" {
        (
            "degraded_consistency",
            true,
            "verify degraded-read policy before relying on strict local semantics",
            false,
        )
    } else {
        ("ok", true, "none", false)
    };

    SpireIndexHealthSnapshot {
        active_epoch: diagnostics.active_epoch,
        consistency_mode: diagnostics.consistency_mode,
        status,
        healthy,
        recommendation,
        compaction_recommended,
        object_count: diagnostics.object_count,
        leaf_assignment_count: diagnostics.leaf_assignment_count,
        delta_assignment_count: diagnostics.delta_assignment_count,
        delta_object_count: diagnostics.delta_object_count,
        available_placement_count: diagnostics.available_placement_count,
        stale_placement_count: diagnostics.stale_placement_count,
        unavailable_placement_count: diagnostics.unavailable_placement_count,
        skipped_placement_count: diagnostics.skipped_placement_count,
    }
}

fn assignment_payload_format_name(format: quantizer::SpireAssignmentPayloadFormat) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

fn assignment_payload_scannability(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> (bool, &'static str, &'static str) {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant
        | quantizer::SpireAssignmentPayloadFormat::RaBitQ => (
            true,
            "supported",
            "format can be used for current SPIRE scans",
        ),
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => (
            false,
            "deferred_model_metadata",
            "persist grouped-PQ model metadata before using pq_fastscan with SPIRE",
        ),
    }
}

fn consistency_mode_name(mode: meta::SpireConsistencyMode) -> &'static str {
    match mode {
        meta::SpireConsistencyMode::Strict => "strict",
        meta::SpireConsistencyMode::Degraded => "degraded",
    }
}

fn placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

fn partition_object_kind_name(kind: storage::SpirePartitionObjectKind) -> &'static str {
    match kind {
        storage::SpirePartitionObjectKind::Root => "root",
        storage::SpirePartitionObjectKind::Internal => "internal",
        storage::SpirePartitionObjectKind::Leaf => "leaf",
        storage::SpirePartitionObjectKind::Delta => "delta",
    }
}

pub(crate) unsafe fn active_snapshot_diagnostics(
    index_relation: pg_sys::Relation,
) -> SpireActiveSnapshotDiagnostics {
    let result = (|| -> Result<SpireActiveSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(SpireActiveSnapshotDiagnostics::empty(root_control));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireActiveSnapshotDiagnostics {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: consistency_mode_name(diagnostics.consistency_mode),
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            stale_placement_count: diagnostics.stale_placement_count as u64,
            unavailable_placement_count: diagnostics.unavailable_placement_count as u64,
            skipped_placement_count: diagnostics.skipped_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            internal_object_count: diagnostics.internal_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            delta_object_count: diagnostics.delta_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            delta_assignment_count: diagnostics.delta_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
            routing_object_bytes: diagnostics.routing_object_bytes,
            leaf_object_bytes: diagnostics.leaf_object_bytes,
            delta_object_bytes: diagnostics.delta_object_bytes,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_options_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexOptionsSnapshot {
    let result = (|| -> Result<SpireIndexOptionsSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let active_leaf_count = if root_control.active_epoch == 0 {
            0
        } else {
            let (epoch_manifest, object_manifest, placement_directory) =
                unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
            let snapshot = meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?;
            let object_store =
                unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
            scan::count_snapshot_single_level_leaf_pids(&snapshot, &object_store)?
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);
        let assignment_payload_format = relation_options.assignment_payload_format();
        let (
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        ) = assignment_payload_scannability(assignment_payload_format);

        Ok(SpireIndexOptionsSnapshot {
            nlists: relation_options.nlists,
            active_leaf_count,
            relation_nprobe: relation_options.nprobe,
            session_nprobe: nprobe
                .session_nprobe
                .map(|value| i32::try_from(value).expect("session nprobe should fit in i32")),
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            relation_rerank_width: relation_options.rerank_width,
            session_rerank_width: rerank_width.session_rerank_width,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            training_sample_rows: relation_options.training_sample_rows,
            seed: relation_options.seed,
            pq_group_size: relation_options.pq_group_size,
            storage_format: relation_options.storage_format.reloption_name(),
            assignment_payload_format: assignment_payload_format_name(assignment_payload_format),
            assignment_payload_scannable,
            assignment_payload_status,
            assignment_payload_recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_health_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHealthSnapshot {
    let diagnostics = unsafe { active_snapshot_diagnostics(index_relation) };
    health_snapshot_from_diagnostics(&diagnostics)
}

pub(crate) unsafe fn index_placement_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexPlacementSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let rows = diagnostics::collect_store_placement_diagnostics(&snapshot, &object_store)?
            .into_iter()
            .map(|row| SpireIndexPlacementSnapshotRow {
                active_epoch: row.epoch,
                node_id: row.node_id,
                local_store_id: row.local_store_id,
                placement_count: row.placement_count as u64,
                available_placement_count: row.available_placement_count as u64,
                stale_placement_count: row.stale_placement_count as u64,
                unavailable_placement_count: row.unavailable_placement_count as u64,
                skipped_placement_count: row.skipped_placement_count as u64,
                object_count: row.object_count as u64,
                root_object_count: row.root_object_count as u64,
                internal_object_count: row.internal_object_count as u64,
                leaf_object_count: row.leaf_object_count as u64,
                delta_object_count: row.delta_object_count as u64,
                routing_child_count: row.routing_child_count as u64,
                assignment_count: row.assignment_count as u64,
                placement_object_bytes: row.placement_object_bytes,
                available_object_bytes: row.available_object_bytes,
                routing_object_bytes: row.routing_object_bytes,
                leaf_object_bytes: row.leaf_object_bytes,
                delta_object_bytes: row.delta_object_bytes,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_placement_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanPlacementSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = scan::collect_single_level_scan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let rows = diagnostics
            .stores
            .into_iter()
            .map(|store| SpireIndexScanPlacementSnapshotRow {
                active_epoch: store.epoch,
                effective_nprobe: diagnostics.scan_plan.nprobe,
                effective_nprobe_source: diagnostics.scan_plan.nprobe_source,
                effective_rerank_width: diagnostics.scan_plan.rerank_width as u64,
                effective_rerank_width_source: diagnostics.scan_plan.rerank_width_source,
                node_id: store.node_id,
                local_store_id: store.local_store_id,
                scanned_pid_count: store.scanned_pid_count as u64,
                leaf_pid_count: store.leaf_pid_count as u64,
                delta_pid_count: store.delta_pid_count as u64,
                candidate_row_count: store.candidate_row_count as u64,
                leaf_candidate_row_count: store.leaf_candidate_row_count as u64,
                delta_candidate_row_count: store.delta_candidate_row_count as u64,
                delete_delta_row_count: store.delete_delta_row_count as u64,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_root_routing_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRootRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let mut root = None;
        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "root routing snapshot")?;
            let header = object_store.read_object_header(lookup.placement)?;
            if header.kind != storage::SpirePartitionObjectKind::Root {
                continue;
            }
            if root.is_some() {
                return Err("ec_spire root routing snapshot found multiple root objects".to_owned());
            }
            root = Some((
                manifest_entry.pid,
                manifest_entry.object_version,
                object_store.read_routing_object(lookup.placement)?,
            ));
        }

        let Some((root_pid, root_object_version, root_object)) = root else {
            return Err("ec_spire root routing snapshot found no active root object".to_owned());
        };
        let root_child_count = root_object.child_count() as u64;
        let rows = root_object
            .children()
            .map(|child| {
                let child_lookup =
                    snapshot.require_lookup(child.child_pid, "root routing child")?;
                let child_header = object_store.read_object_header(child_lookup.placement)?;
                Ok(SpireIndexRootRoutingSnapshotRow {
                    active_epoch: snapshot.epoch_manifest().epoch,
                    root_pid,
                    root_object_version,
                    root_level: root_object.header.level,
                    root_child_count,
                    centroid_dimensions: root_object.dimensions,
                    centroid_index: child.centroid_index,
                    child_pid: child.child_pid,
                    child_kind: partition_object_kind_name(child_header.kind),
                    child_object_version: child_header.object_version,
                    child_level: child_header.level,
                    child_parent_pid: child_header.parent_pid,
                    child_assignment_count: u64::from(child_header.assignment_count),
                    child_node_id: child_lookup.placement.node_id,
                    child_local_store_id: child_lookup.placement.local_store_id,
                    child_store_relid: child_lookup.placement.store_relid,
                    child_placement_state: placement_state_name(child_lookup.placement.state),
                    child_object_bytes: u64::from(child_lookup.placement.object_bytes),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
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

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_root_control(index_oid: pg_sys::Oid) -> (u64, u64, u64) {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    (
        root_control.active_epoch,
        root_control.next_pid,
        root_control.next_local_vec_seq,
    )
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDebugSnapshotDiagnostics {
    pub(crate) epoch: u64,
    pub(crate) object_count: u64,
    pub(crate) placement_count: u64,
    pub(crate) local_store_count: u64,
    pub(crate) available_placement_count: u64,
    pub(crate) root_object_count: u64,
    pub(crate) leaf_object_count: u64,
    pub(crate) routing_child_count: u64,
    pub(crate) leaf_assignment_count: u64,
    pub(crate) available_object_bytes: u64,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_spire_active_snapshot_diagnostics(
    index_oid: pg_sys::Oid,
) -> SpireDebugSnapshotDiagnostics {
    let lockmode = pg_sys::AccessShareLock as pg_sys::LOCKMODE;
    let index_relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
    let result = (|| -> Result<SpireDebugSnapshotDiagnostics, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        let diagnostics = diagnostics::collect_snapshot_diagnostics(&snapshot, &object_store)?;

        Ok(SpireDebugSnapshotDiagnostics {
            epoch: diagnostics.epoch,
            object_count: diagnostics.object_count as u64,
            placement_count: diagnostics.placement_count as u64,
            local_store_count: diagnostics.local_store_count as u64,
            available_placement_count: diagnostics.available_placement_count as u64,
            root_object_count: diagnostics.root_object_count as u64,
            leaf_object_count: diagnostics.leaf_object_count as u64,
            routing_child_count: diagnostics.routing_child_count as u64,
            leaf_assignment_count: diagnostics.leaf_assignment_count as u64,
            available_object_bytes: diagnostics.available_object_bytes,
        })
    })();
    unsafe { pg_sys::index_close(index_relation, lockmode) };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}
