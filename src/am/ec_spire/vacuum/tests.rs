#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePublishedEpochSnapshot,
        SpireValidatedEpochSnapshot,
    };
    use crate::am::ec_spire::storage::{
        SpireLeafPartitionObject, SpireLocalObjectStore, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment(vec_seq: u64, block_number: u32, offset_number: u16) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![vec_seq as u8],
        }
    }

    fn delete_assignment(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        let mut row = assignment(vec_seq, block_number, offset_number);
        row.flags = SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        row.payload_format = 0;
        row.gamma = 0.0;
        row.encoded_payload.clear();
        row
    }

    fn delta_insert_assignment(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        let mut row = assignment(vec_seq, block_number, offset_number);
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
        row
    }

    #[test]
    fn compaction_leaf_pid_match_rejects_malformed_header_pid() {
        let error = require_compaction_leaf_pid_match(42, 43).unwrap_err();

        assert_eq!(
            error,
            "ec_spire vacuum compaction leaf pid mismatch: manifest pid 42, object header pid 43"
        );
    }

    #[test]
    fn compaction_leaf_pid_match_returns_manifest_pid() {
        assert_eq!(require_compaction_leaf_pid_match(42, 42), Ok(42));
    }

    #[test]
    fn compaction_leaf_object_version_match_rejects_malformed_header_version() {
        let error = require_compaction_leaf_object_version_match(7, 8, 42).unwrap_err();

        assert_eq!(
            error,
            "ec_spire vacuum compaction leaf object_version mismatch for pid 42: manifest object_version 7, object header object_version 8"
        );
    }

    #[test]
    fn compaction_leaf_object_version_match_returns_manifest_version() {
        assert_eq!(
            require_compaction_leaf_object_version_match(7, 7, 42),
            Ok(7)
        );
    }

    #[test]
    fn miri_collect_visible_assignments_excludes_delete_delta_targets() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut boundary = assignment(4, 13, 1);
        boundary.flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        let leaf = SpireLeafPartitionObject::new(
            1,
            1,
            0,
            vec![assignment(1, 10, 1), assignment(2, 11, 1), boundary],
        )
        .unwrap();
        let leaf_placement = object_store.insert_leaf_object(7, &leaf).unwrap();
        let delta = SpireDeltaPartitionObject::new(
            2,
            1,
            1,
            vec![
                delete_assignment(1, 10, 1),
                delta_insert_assignment(3, 12, 1),
            ],
        )
        .unwrap();
        let delta_placement = object_store.insert_delta_object(7, &delta).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: leaf_placement.pid,
                    object_version: leaf_placement.object_version,
                    placement_tid: leaf_placement.object_tid,
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: delta_placement.pid,
                    object_version: delta_placement.object_version,
                    placement_tid: delta_placement.object_tid,
                },
            ],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![leaf_placement, delta_placement])
                .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let deleted = collect_delete_vec_ids_by_base_pid(
            &SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap(),
            &object_store,
        )
        .unwrap();
        assert_eq!(deleted.get(&1).unwrap().len(), 1);
        assert!(deleted.get(&1).unwrap().contains(&SpireVecId::local(1)));

        let visible = collect_visible_assignments(&snapshot, &object_store).unwrap();
        let visible_vec_ids = visible
            .iter()
            .map(|row| row.assignment.vec_id.clone())
            .collect::<Vec<_>>();

        assert_eq!(visible.len(), 2);
        assert!(visible_vec_ids.contains(&SpireVecId::local(2)));
        assert!(visible_vec_ids.contains(&SpireVecId::local(3)));
        assert!(!visible_vec_ids.contains(&SpireVecId::local(1)));
        assert!(!visible_vec_ids.contains(&SpireVecId::local(4)));
        assert_eq!(visible[0].base_pid, 1);
        assert_eq!(visible[1].base_pid, 1);
    }
}
