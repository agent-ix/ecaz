#[cfg(test)]
mod tests {
    use super::retired_epoch_manifest_from;
    use super::{
        build_local_recursive_routing_epoch_draft, build_partitioned_single_level_leaf_epoch_draft,
        build_single_level_leaf_epoch_draft, encode_publish_bundle_for_publish,
        object_manifest_from_placement_writes, object_write_evidence_from_placement_directory,
        placement_write_evidence_from_object_manifest, resolve_training_sample_count,
        train_single_level_centroid_plan, SpireBuildState, SpireBuildTuple, SpireIndexedVectorKind,
        SpirePartitionedSingleLevelBuildInput, SpirePublishPlacementWriteEvidence,
        SpirePublishStage, SpirePublishWritingObjects, SpireRecursiveBuildCoordinatorInput,
        SpireRecursiveLeafObjectInput, SpireRecursiveRoutingBuildInput,
        SpireRecursiveRoutingChildInput, SpireRecursiveRoutingEpochInput,
        SpireRecursiveRoutingEpochObjectInput, SpireSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan, SpireSingleLevelRouteMap,
    };
    use super::{SpirePublishedManifestLocators, SpireSingleLevelBuildDraft};
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
        SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochState, SpirePublishedEpochSnapshot,
    };
    use crate::am::ec_spire::meta::{
        SpireEpochManifest, SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireObjectManifest,
        SpirePlacementDirectory, SpireRootControlState,
    };
    use crate::am::ec_spire::quantizer::{self, SpireAssignmentPayloadFormat};
    use crate::am::ec_spire::storage::{
        SpireLeafAssignmentRow, SpireLocalObjectStore, SpireLocalObjectStoreSet, SpireObjectReader,
        SpirePartitionObjectKind, SpireVecId, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_input(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn options(training_sample_rows: i32) -> super::options::EcSpireOptions {
        super::options::EcSpireOptions {
            nlists: 2,
            recursive_fanout: 0,
            local_store_count: 1,
            nprobe: 0,
            rerank_width: 0,
            training_sample_rows,
            seed: 7,
            pq_group_size: 0,
            storage_format: super::options::SpireStorageFormat::TurboQuant,
            local_store_tablespaces: None,
        }
    }

    fn build_tuple(offset_number: u16, source_vector: Vec<f32>) -> SpireBuildTuple {
        let heap_tid = tid(10, offset_number);
        let assignment = quantizer::encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            heap_tid,
            &source_vector,
        )
        .unwrap();
        SpireBuildTuple {
            heap_tid,
            dimensions: source_vector.len() as u16,
            assignment,
            source_vector,
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
        centroid_plan: SpireSingleLevelCentroidPlan,
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
            centroid_plan,
        }
    }

    fn recursive_child(pid: u64, centroid: Vec<f32>) -> SpireRecursiveRoutingChildInput {
        SpireRecursiveRoutingChildInput {
            child_pid: pid,
            child_level: 0,
            centroid,
            source_count: 1,
        }
    }

    fn primary_row(vec_seq: u64, block_number: u32, offset_number: u16) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn build_valid_draft() -> (
        SpireSingleLevelBuildDraft,
        SpirePidAllocator,
        SpireLocalVecIdAllocator,
        SpireLocalObjectStore,
    ) {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1), assignment_input(10, 2)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();

        (draft, pid_allocator, local_vec_id_allocator, object_store)
    }

    fn manifest_locators() -> SpirePublishedManifestLocators {
        SpirePublishedManifestLocators {
            epoch_manifest_tid: tid(70, 1),
            object_manifest_tid: tid(70, 2),
            placement_directory_tid: tid(70, 3),
            local_store_config_tid: tid(70, 4),
        }
    }

    include!("tests/centroid_state.rs");
    include!("tests/recursive.rs");
    include!("tests/publish.rs");
    include!("tests/single_level.rs");
}
