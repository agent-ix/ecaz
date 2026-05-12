#[cfg(test)]
mod tests {
    use super::{
        collect_quantized_routed_probe_candidates, collect_ranked_routed_probe_candidates,
        collect_delta_delete_vec_ids_for_loaded_routes,
        collect_reranked_quantized_routed_probe_candidates, collect_scan_routing_diagnostics,
        collect_scan_plan_selected_leaf_pids,
        collect_single_level_scan_plan_placement_diagnostics,
        collect_single_level_scan_plan_reranked_candidates, collect_snapshot_delta_rows,
        collect_snapshot_leaf_rows, collect_snapshot_routed_leaf_rows,
        collect_snapshot_routed_probe_leaf_rows, collect_snapshot_top_graph_routed_probe_leaf_rows,
        collect_snapshot_visible_primary_rows, collect_top_graph_scan_plan_reranked_candidates,
        count_snapshot_recursive_leaf_pids, count_snapshot_single_level_leaf_pids,
        ensure_local_heap_placement_directory_is_deliverable,
        group_leaf_and_delta_reads_by_local_store, heap_rerank_prefetch_block_numbers,
        load_delta_rows_for_routes, load_snapshot_routing_hierarchy, load_snapshot_top_graph_object,
        prefetch_store_object_read_groups, production_scan_result_stream_am_outputs,
        rank_routed_leaf_rows_by_ip,
        rerank_scored_candidates_by_ip, rerank_scored_candidates_by_ip_with_prefetch,
        route_recursive_routing_objects_to_leaf_pids,
        route_recursive_routing_objects_to_leaf_routes_with_budget,
        route_recursive_routing_objects_to_leaf_routes_with_policy, route_root_object_to_leaf_pids,
        route_routing_object_to_child_pids, route_routing_object_to_child_routes_with_policy,
        route_top_graph_object_to_child_pids, route_top_graph_object_to_leaf_routes,
        route_top_graph_to_child_pids, SpireDeltaObjectRoute, SpireLeafObjectReadRoute,
        SpireLeafScanRow, SpireNoopRoutedScanObserver, SpireRecursiveLeafRoute,
        SpireRoutedLeafScanRows, SpireScanCandidateCursor, SpireScanOpaque, SpireScanOutput,
        SpireScanOutputCursor, SpireScanPlacementDiagnosticsObserver, SpireScanQuery,
        SpireScoredScanCandidate, SpireStoreObjectReadGroup,
    };
    use crate::am::ec_spire::{
        SpireRemoteProductionScanAmDeliverySummaryRow,
        SpireRemoteProductionScanHeapResolutionSummaryRow, SpireRemoteProductionScanOutputRow,
        SpireRemoteProductionScanResultStream,
        SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY,
        SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY,
        SPIRE_REMOTE_FINAL_STATUS_REQUIRES_CUSTOM_SCAN_TUPLE_DELIVERY,
        SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
        SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_local_recursive_routing_epoch_draft, build_partitioned_single_level_leaf_epoch_draft,
        build_recursive_routing_hierarchy_draft, build_single_level_leaf_epoch_draft,
        build_spire_top_graph_draft_from_routing_object,
        spire_top_graph_partition_object_from_build_draft, SpirePartitionedSingleLevelBuildInput,
        SpireRecursiveRoutingBuildInput, SpireRecursiveRoutingChildInput,
        SpireRecursiveRoutingEpochInput, SpireSingleLevelBuildInput, SpireSingleLevelCentroidPlan,
        SpireTopGraphBuildParams,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireObjectManifest,
        SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot, SpireRootControlState, SpireValidatedEpochSnapshot,
    };
    use crate::am::ec_spire::options::{
        resolve_single_level_scan_plan_values, EcSpireOptions, SpireCandidateDedupeMode,
        SpireRecursiveNprobePolicy, SpireRecursiveRouteBudget, SpireSingleLevelScanPlan,
        SpireSourceIdentityProvider, SpireStorageFormat,
    };
    use crate::am::ec_spire::quantizer::{
        encode_assignment_input, SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer,
    };
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireLeafAssignmentRow, SpireLeafPartitionObject,
        SpireLocalObjectStore, SpireLocalObjectStoreSet, SpireObjectReader,
        SpirePartitionObjectHeader, SpirePartitionObjectKind, SpireRoutingChildEntry,
        SpireRoutingPartitionObject, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };
    use crate::am::ec_spire::update::{
        build_delta_epoch_draft_from_snapshot, SpireDeltaEpochInput,
    };
    use crate::storage::page::ItemPointer;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn production_scan_stream_for_am(
        am_delivery: SpireRemoteProductionScanAmDeliverySummaryRow,
        outputs: Vec<SpireRemoteProductionScanOutputRow>,
    ) -> SpireRemoteProductionScanResultStream {
        SpireRemoteProductionScanResultStream {
            summary: SpireRemoteProductionScanHeapResolutionSummaryRow {
                requested_epoch: am_delivery.requested_epoch,
                consistency_mode_source: "test",
                consistency_mode: "strict",
                effective_nprobe: 1,
                selected_pid_count: 1,
                local_pid_count: am_delivery.local_heap_tid_output_count,
                remote_pid_count: am_delivery.remote_origin_output_count,
                skipped_pid_count: 0,
                dispatch_count: am_delivery.remote_origin_output_count,
                compact_candidate_count: am_delivery.output_count,
                remote_heap_ready_dispatch_count: am_delivery.remote_origin_output_count,
                remote_heap_failed_dispatch_count: 0,
                remote_heap_candidate_count: am_delivery.remote_origin_output_count,
                local_heap_candidate_count: am_delivery.local_heap_tid_output_count,
                returned_candidate_count: am_delivery.output_count,
                result_source: "test",
                final_heap_fetch_status: SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY,
                next_blocker: am_delivery.next_blocker,
                status: am_delivery.status,
                recommendation: am_delivery.recommendation,
            },
            am_delivery,
            outputs,
        }
    }

    fn assignment_input(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        assignment_input_with_payload(block_number, offset_number, vec![1, 2, 3])
    }

    fn quantized_assignment_input(
        block_number: u32,
        offset_number: u16,
        payload_format: SpireAssignmentPayloadFormat,
        source_vector: &[f32],
    ) -> SpireLeafAssignmentInput {
        encode_assignment_input(
            payload_format,
            tid(block_number, offset_number),
            source_vector,
        )
        .unwrap()
    }

    fn assignment_input_with_payload(
        block_number: u32,
        offset_number: u16,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload,
        }
    }

    fn build_input(assignments: Vec<SpireLeafAssignmentInput>) -> SpireSingleLevelBuildInput {
        SpireSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_tid: tid(60, 1),
            assignments,
        }
    }

    fn partitioned_build_input(
        assignments: Vec<SpireLeafAssignmentInput>,
        assignment_indexes: Vec<u32>,
    ) -> SpirePartitionedSingleLevelBuildInput {
        SpirePartitionedSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            root_placement_tid: tid(60, 3),
            placement_tids: vec![tid(60, 1), tid(60, 2)],
            assignments,
            centroid_plan: SpireSingleLevelCentroidPlan {
                dimensions: 2,
                centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
                assignment_indexes,
            },
        }
    }

    fn delta_input(
        insert_assignments: Vec<SpireLeafAssignmentInput>,
        delete_assignments: Vec<SpireDeleteDeltaInput>,
    ) -> SpireDeltaEpochInput {
        SpireDeltaEpochInput {
            epoch: 8,
            object_version: 3,
            published_at_micros: 2000,
            retain_until_micros: 3000,
            consistency_mode: SpireConsistencyMode::Strict,
            base_pid: SPIRE_FIRST_PID,
            placement_tid: tid(80, 1),
            insert_assignments,
            delete_assignments,
        }
    }

    fn delete_delta_input(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireDeleteDeltaInput {
        SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
        }
    }

    fn assignment_row(flags: u16, offset_number: u16) -> SpireLeafAssignmentRow {
        assignment_row_with_payload(
            flags,
            u64::from(offset_number),
            10,
            offset_number,
            vec![1, 2, 3],
        )
    }

    fn assignment_row_with_payload(
        flags: u16,
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload,
        }
    }

    fn delete_assignment_row(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        }
    }

    fn scored_candidate(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
        score: f32,
    ) -> SpireScoredScanCandidate {
        SpireScoredScanCandidate {
            epoch: 1,
            pid: SPIRE_FIRST_PID + vec_seq,
            object_version: 1,
            row_index: u32::from(offset_number),
            assignment_flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            score,
        }
    }

    fn routing_child(
        centroid_index: u32,
        child_pid: u64,
        centroid: Vec<f32>,
    ) -> SpireRoutingChildEntry {
        SpireRoutingChildEntry {
            centroid_index,
            child_pid,
            centroid,
        }
    }

    fn snapshot_for_placement<'a>(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> SpirePublishedEpochSnapshot<'a> {
        SpirePublishedEpochSnapshot::new(epoch_manifest, object_manifest, placement_directory)
            .unwrap()
    }

    fn manifest_entry_for(placement: &SpirePlacementEntry) -> SpireManifestEntry {
        SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
    }


    include!("tests/snapshot_rows.rs");
    include!("tests/routed_rows.rs");
    include!("tests/diagnostics.rs");
    include!("tests/candidates.rs");
    include!("tests/routing.rs");
    include!("tests/recursive_candidates.rs");
    include!("tests/runtime_state.rs");
}
