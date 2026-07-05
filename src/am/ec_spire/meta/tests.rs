#[cfg(test)]
mod tests {
    use super::{
        plan_epoch_cleanup, spire_pid_hash, SpireConsistencyMode, SpireEpochManifest,
        SpireEpochState, SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireLocalStoreState,
        SpireManifestEntry, SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry,
        SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
        SpireValidatedEpochSnapshot, SPIRE_DEFAULT_LOCAL_STORE_GENERATION,
        SPIRE_FAILED_EPOCH_RETENTION_SECS, SPIRE_LOCAL_NODE_ID, SPIRE_MAX_RETAINED_RETIRED_EPOCHS,
        SPIRE_MIN_EPOCH_RETENTION_SECS, SPIRE_SINGLE_LOCAL_STORE_ID,
    };
    use crate::am::ec_spire::assign::{SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID};
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn published_epoch(epoch: u64, consistency_mode: SpireConsistencyMode) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Published,
            consistency_mode,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        }
    }

    fn retired_epoch(
        epoch: u64,
        retain_until_micros: i64,
        active_query_count: u64,
    ) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros,
            active_query_count,
        }
    }

    fn failed_epoch(epoch: u64, retain_until_micros: i64) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros,
            active_query_count: 0,
        }
    }

    fn object_manifest(epoch: u64, pid: u64, object_version: u64) -> SpireObjectManifest {
        SpireObjectManifest::from_entries(
            epoch,
            vec![SpireManifestEntry {
                epoch,
                pid,
                object_version,
                placement_tid: tid(55, 4),
            }],
        )
        .unwrap()
    }

    fn placement_directory(
        epoch: u64,
        pid: u64,
        object_version: u64,
        state: SpirePlacementState,
    ) -> SpirePlacementDirectory {
        let mut placement = SpirePlacementEntry::local_single_store(
            epoch,
            pid,
            12345,
            object_version,
            tid(44, 2),
            4096,
        );
        placement.state = state;
        SpirePlacementDirectory::from_entries(epoch, vec![placement]).unwrap()
    }

    #[test]
    fn retention_defaults_match_phase0_design() {
        assert_eq!(SPIRE_MIN_EPOCH_RETENTION_SECS, 600);
        assert_eq!(SPIRE_FAILED_EPOCH_RETENTION_SECS, 3600);
        assert_eq!(SPIRE_MAX_RETAINED_RETIRED_EPOCHS, 2);
    }

    #[test]
    fn root_control_empty_state_round_trips() {
        let state = SpireRootControlState::empty();

        assert_eq!(state.active_epoch, 0);
        assert_eq!(state.next_pid, SPIRE_FIRST_PID);
        assert_eq!(state.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ);
        assert_eq!(state.epoch_manifest_tid, ItemPointer::INVALID);
        assert_eq!(
            SpireRootControlState::decode(&state.encode().unwrap()).unwrap(),
            state
        );
    }

    #[test]
    fn root_control_published_state_round_trips() {
        let state =
            SpireRootControlState::published(7, 12, 100, tid(50, 1), tid(50, 2), tid(50, 3))
                .unwrap();

        assert_eq!(
            SpireRootControlState::decode(&state.encode().unwrap()).unwrap(),
            state
        );
    }

    #[test]
    fn root_control_rejects_invalid_cursors_and_manifest_refs() {
        assert!(
            SpireRootControlState::published(7, 0, 100, tid(50, 1), tid(50, 2), tid(50, 3))
                .is_err()
        );
        assert!(
            SpireRootControlState::published(7, 12, 0, tid(50, 1), tid(50, 2), tid(50, 3)).is_err()
        );
        assert!(SpireRootControlState::published(
            7,
            12,
            100,
            ItemPointer::INVALID,
            tid(50, 2),
            tid(50, 3),
        )
        .is_err());

        let mut empty = SpireRootControlState::empty();
        empty.epoch_manifest_tid = tid(50, 1);
        assert!(empty.encode().is_err());
    }

    #[test]
    fn root_control_rejects_corrupt_header() {
        let state = SpireRootControlState::empty();
        let mut encoded = state.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireRootControlState::decode(&encoded).is_err());

        encoded = state.encode().unwrap();
        encoded[6] = 1;
        assert!(SpireRootControlState::decode(&encoded).is_err());
    }

    #[test]
    fn embedded_single_store_config_preserves_current_store_shape() {
        let config = SpireLocalStoreConfig::embedded_single_store(12345, 0)
            .expect("default tablespace oid 0 should be allowed");

        assert_eq!(config.generation, SPIRE_DEFAULT_LOCAL_STORE_GENERATION);
        assert_eq!(config.stores.len(), 1);
        assert_eq!(config.stores[0].local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
        assert_eq!(config.stores[0].store_relid, 12345);
        assert_eq!(config.stores[0].tablespace_oid, 0);
        assert_eq!(config.stores[0].state, SpireLocalStoreState::Available);

        let decoded = SpireLocalStoreConfig::decode(&config.encode().unwrap()).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn local_store_config_allows_repeated_tablespaces_for_baselines() {
        let config = SpireLocalStoreConfig::from_stores(
            2,
            vec![
                SpireLocalStoreDescriptor::available(1, 12346, 987).unwrap(),
                SpireLocalStoreDescriptor::available(0, 12345, 987).unwrap(),
            ],
        )
        .expect("repeated tablespace oid should be accepted");

        assert_eq!(config.stores[0].local_store_id, 0);
        assert_eq!(config.stores[1].local_store_id, 1);
        assert_eq!(
            config.stores[0].tablespace_oid,
            config.stores[1].tablespace_oid
        );

        let decoded = SpireLocalStoreConfig::decode(&config.encode().unwrap()).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn local_store_config_rejects_empty_duplicate_or_invalid_store_relid() {
        assert!(SpireLocalStoreConfig::from_stores(1, Vec::new()).is_err());
        assert!(SpireLocalStoreDescriptor::available(0, 0, 42).is_err());
        assert!(SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 12345, 42).unwrap(),
                SpireLocalStoreDescriptor::available(0, 12346, 43).unwrap(),
            ],
        )
        .is_err());
    }

    #[test]
    fn local_store_config_validates_placements_against_active_store_set() {
        let store = SpireLocalStoreDescriptor::available(2, 12347, 987).unwrap();
        let config = SpireLocalStoreConfig::from_stores(4, vec![store]).unwrap();
        let placement =
            SpirePlacementEntry::local_store_available(7, 11, &store, 3, tid(44, 2), 4096);

        config.validate_placement(&placement).unwrap();

        let mut wrong_store_id = placement;
        wrong_store_id.local_store_id = 3;
        assert!(config.validate_placement(&wrong_store_id).is_err());

        let mut wrong_relid = placement;
        wrong_relid.store_relid = 99999;
        assert!(config.validate_placement(&wrong_relid).is_err());

        let unavailable_config = SpireLocalStoreConfig::from_stores(
            4,
            vec![SpireLocalStoreDescriptor {
                state: SpireLocalStoreState::Unavailable,
                ..store
            }],
        )
        .unwrap();
        assert!(unavailable_config.validate_placement(&placement).is_err());
    }

    #[test]
    fn spire_pid_hash_has_stable_cross_platform_values() {
        assert_eq!(spire_pid_hash(1), 0x5692_161d_100b_05e5);
        assert_eq!(spire_pid_hash(2), 0xdbd2_3897_3a2b_148a);
        assert_eq!(spire_pid_hash(11), 0x3462_d848_f53a_bb6d);
        assert_eq!(spire_pid_hash(123_456_789), 0xf21c_87d4_233f_fd60);
    }

    #[test]
    fn local_store_config_places_pid_by_stable_hash_mod_store_count() {
        let config = SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 12345, 900).unwrap(),
                SpireLocalStoreDescriptor::available(1, 12346, 901).unwrap(),
                SpireLocalStoreDescriptor::available(2, 12347, 902).unwrap(),
                SpireLocalStoreDescriptor::available(3, 12348, 903).unwrap(),
            ],
        )
        .unwrap();

        assert_eq!(config.store_for_pid(1).unwrap().local_store_id, 1);
        assert_eq!(config.store_for_pid(2).unwrap().local_store_id, 2);
        assert_eq!(config.store_for_pid(3).unwrap().local_store_id, 0);
        assert_eq!(config.store_for_pid(11).unwrap().local_store_id, 1);
        assert!(config.store_for_pid(0).is_err());
    }

    #[test]
    fn local_single_store_placement_uses_default_ids() {
        let entry = SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096);

        assert_eq!(entry.node_id, SPIRE_LOCAL_NODE_ID);
        assert_eq!(entry.local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
        assert_eq!(entry.state, SpirePlacementState::Available);
    }

    #[test]
    fn local_single_store_state_constructors_make_state_explicit() {
        let available =
            SpirePlacementEntry::local_single_store_available(7, 11, 12345, 3, tid(44, 2), 4096);
        let stale =
            SpirePlacementEntry::local_single_store_stale(7, 11, 12345, 3, tid(44, 2), 4096);
        let unavailable =
            SpirePlacementEntry::local_single_store_unavailable(7, 11, 12345, 3, tid(44, 2), 4096);
        let skipped =
            SpirePlacementEntry::local_single_store_skipped(7, 11, 12345, 3, tid(44, 2), 4096);

        assert_eq!(available.state, SpirePlacementState::Available);
        assert_eq!(stale.state, SpirePlacementState::Stale);
        assert_eq!(unavailable.state, SpirePlacementState::Unavailable);
        assert_eq!(skipped.state, SpirePlacementState::Skipped);
        assert_eq!(stale.node_id, SPIRE_LOCAL_NODE_ID);
        assert_eq!(stale.local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
    }

    #[test]
    fn placement_entry_round_trips() {
        let entry = SpirePlacementEntry {
            epoch: 7,
            pid: 11,
            node_id: 0,
            local_store_id: 2,
            store_relid: 12345,
            object_version: 3,
            object_tid: tid(44, 2),
            object_bytes: 8192,
            state: SpirePlacementState::Stale,
        };

        assert_eq!(
            SpirePlacementEntry::decode(&entry.encode().unwrap()).unwrap(),
            entry
        );
    }

    #[test]
    fn placement_entry_rejects_invalid_identity_and_locator() {
        let mut entry = SpirePlacementEntry::local_single_store(0, 11, 12345, 3, tid(44, 2), 4096);
        assert!(entry.encode().is_err());

        entry.epoch = 7;
        entry.pid = 0;
        assert!(entry.encode().is_err());

        entry.pid = 11;
        entry.store_relid = 0;
        assert!(entry.encode().is_err());

        entry.store_relid = 12345;
        entry.object_version = 0;
        assert!(entry.encode().is_err());

        entry.object_version = 3;
        entry.object_tid = ItemPointer::INVALID;
        assert!(entry.encode().is_err());

        entry.object_tid = tid(44, 2);
        entry.object_bytes = 0;
        assert!(entry.encode().is_err());
    }

    #[test]
    fn placement_entry_rejects_unknown_state_and_format() {
        let entry = SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096);
        let mut encoded = entry.encode().unwrap();

        encoded[2] = 99;
        assert!(SpirePlacementEntry::decode(&encoded).is_err());

        encoded[2] = SpirePlacementState::Available as u8;
        encoded[0] = 2;
        assert!(SpirePlacementEntry::decode(&encoded).is_err());
    }

    #[test]
    fn placement_directory_sorts_and_round_trips_entries() {
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 21, 12345, 4, tid(45, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
            ],
        )
        .unwrap();

        let decoded = SpirePlacementDirectory::decode(&directory.encode().unwrap()).unwrap();

        assert_eq!(decoded, directory);
        assert_eq!(decoded.entries[0].pid, 11);
        assert_eq!(decoded.entries[1].pid, 21);
        assert_eq!(decoded.get(21).unwrap().object_version, 4);
        assert!(decoded.get(99).is_none());
    }

    #[test]
    fn placement_directory_rejects_duplicate_pid_and_epoch_mismatch() {
        assert!(SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 4, tid(45, 2), 4096),
            ],
        )
        .is_err());

        assert!(SpirePlacementDirectory::from_entries(
            7,
            vec![SpirePlacementEntry::local_single_store(
                8,
                11,
                12345,
                3,
                tid(44, 2),
                4096,
            )],
        )
        .is_err());
    }

    #[test]
    fn placement_directory_rejects_corrupt_header_and_length() {
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![SpirePlacementEntry::local_single_store(
                7,
                11,
                12345,
                3,
                tid(44, 2),
                4096,
            )],
        )
        .unwrap();
        let mut encoded = directory.encode().unwrap();

        encoded[0] = 0;
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());

        encoded = directory.encode().unwrap();
        encoded[6] = 1;
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());

        encoded = directory.encode().unwrap();
        encoded.push(99);
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_round_trips() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 2,
        };

        assert_eq!(
            SpireEpochManifest::decode(&manifest.encode().unwrap()).unwrap(),
            manifest
        );
        assert_eq!(
            manifest.encode().unwrap().len(),
            SpireEpochManifest::encoded_len()
        );
    }

    #[test]
    fn epoch_manifest_allows_building_without_publish_timestamp() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Building,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };

        assert_eq!(
            SpireEpochManifest::decode(&manifest.encode().unwrap()).unwrap(),
            manifest
        );
    }

    #[test]
    fn epoch_manifest_rejects_invalid_state_and_consistency_mode() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let mut encoded = manifest.encode().unwrap();

        encoded[6] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());

        encoded[6] = SpireEpochState::Failed as u8;
        encoded[7] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_rejects_invalid_magic_and_format() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let mut encoded = manifest.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireEpochManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded[4] = 2;
        assert!(SpireEpochManifest::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_rejects_invalid_publish_timing() {
        let mut manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        assert!(manifest.encode().is_err());

        manifest.state = SpireEpochState::Retired;
        manifest.published_at_micros = 2000;
        manifest.retain_until_micros = 1000;
        assert!(manifest.encode().is_err());
    }

    #[test]
    fn cleanup_eligibility_keeps_building_and_published_epochs() {
        let building = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Building,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let published = published_epoch(8, SpireConsistencyMode::Strict);

        assert!(!building.cleanup_eligible_at(i64::MAX));
        assert!(!published.cleanup_eligible_at(i64::MAX));
    }

    #[test]
    fn cleanup_eligibility_keeps_retired_epochs_until_retention_and_queries_clear() {
        let mut retired = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 1,
        };

        assert!(!retired.cleanup_eligible_at(1999));
        assert!(!retired.cleanup_eligible_at(2000));

        retired.active_query_count = 0;
        assert!(!retired.cleanup_eligible_at(1999));
        assert!(retired.cleanup_eligible_at(2000));
    }

    #[test]
    fn cleanup_eligibility_uses_failed_epoch_retain_until() {
        let failed = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 2000,
            active_query_count: 99,
        };

        assert!(!failed.cleanup_eligible_at(1999));
        assert!(failed.cleanup_eligible_at(2000));
    }

    #[test]
    fn cleanup_plan_keeps_active_epoch_and_newest_retired_epochs() {
        let manifests = vec![
            published_epoch(10, SpireConsistencyMode::Strict),
            retired_epoch(9, 1000, 0),
            retired_epoch(8, 1000, 0),
            retired_epoch(7, 1000, 0),
            failed_epoch(6, 1000),
        ];

        let plan = plan_epoch_cleanup(&manifests, 10, 2000).unwrap();

        assert_eq!(plan.retained_retired_epochs, vec![8, 9]);
        assert_eq!(plan.cleanup_epochs, vec![6, 7]);
    }

    #[test]
    fn cleanup_plan_waits_for_retention_and_active_queries() {
        let manifests = vec![
            retired_epoch(9, 3000, 0),
            retired_epoch(8, 1000, 1),
            retired_epoch(7, 1000, 0),
            failed_epoch(6, 3000),
        ];

        let plan = plan_epoch_cleanup(&manifests, 0, 2000).unwrap();

        assert_eq!(plan.retained_retired_epochs, vec![8, 9]);
        assert_eq!(plan.cleanup_epochs, vec![7]);
    }

    #[test]
    fn cleanup_plan_rejects_duplicate_epochs() {
        let manifests = vec![retired_epoch(7, 1000, 0), failed_epoch(7, 1000)];

        assert!(plan_epoch_cleanup(&manifests, 0, 2000).is_err());
    }

    #[test]
    fn manifest_entry_round_trips() {
        let entry = SpireManifestEntry {
            epoch: 7,
            pid: 11,
            object_version: 3,
            placement_tid: tid(55, 4),
        };

        assert_eq!(
            SpireManifestEntry::decode(&entry.encode().unwrap()).unwrap(),
            entry
        );
    }

    #[test]
    fn manifest_entry_rejects_invalid_identity_locator_and_reserved_bytes() {
        let mut entry = SpireManifestEntry {
            epoch: 0,
            pid: 11,
            object_version: 3,
            placement_tid: tid(55, 4),
        };
        assert!(entry.encode().is_err());

        entry.epoch = 7;
        entry.pid = 0;
        assert!(entry.encode().is_err());

        entry.pid = 11;
        entry.object_version = 0;
        assert!(entry.encode().is_err());

        entry.object_version = 3;
        entry.placement_tid = ItemPointer::INVALID;
        assert!(entry.encode().is_err());

        entry.placement_tid = tid(55, 4);
        let mut encoded = entry.encode().unwrap();
        encoded[2] = 1;
        assert!(SpireManifestEntry::decode(&encoded).is_err());
    }

    #[test]
    fn object_manifest_sorts_and_round_trips_entries() {
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 21,
                    object_version: 4,
                    placement_tid: tid(45, 2),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(44, 2),
                },
            ],
        )
        .unwrap();

        let decoded = SpireObjectManifest::decode(&manifest.encode().unwrap()).unwrap();

        assert_eq!(decoded, manifest);
        assert_eq!(decoded.entries[0].pid, 11);
        assert_eq!(decoded.entries[1].pid, 21);
        assert_eq!(decoded.get(21).unwrap().object_version, 4);
        assert!(decoded.get(99).is_none());
    }

    #[test]
    fn object_manifest_rejects_duplicate_pid_and_epoch_mismatch() {
        assert!(SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(44, 2),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 4,
                    placement_tid: tid(45, 2),
                },
            ],
        )
        .is_err());

        assert!(SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 8,
                pid: 11,
                object_version: 3,
                placement_tid: tid(44, 2),
            }],
        )
        .is_err());
    }

    #[test]
    fn object_manifest_rejects_corrupt_header_and_length() {
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 7,
                pid: 11,
                object_version: 3,
                placement_tid: tid(44, 2),
            }],
        )
        .unwrap();
        let mut encoded = manifest.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireObjectManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded[6] = 1;
        assert!(SpireObjectManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded.push(99);
        assert!(SpireObjectManifest::decode(&encoded).is_err());
    }

    #[test]
    fn published_epoch_snapshot_accepts_strict_available_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        let snapshot = SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).unwrap();

        assert_eq!(snapshot.epoch_manifest.epoch, 7);
        assert_eq!(snapshot.object_manifest.get(11).unwrap().object_version, 3);
        assert_eq!(
            snapshot.placement_directory.get(11).unwrap().state,
            SpirePlacementState::Available
        );
    }

    #[test]
    fn validated_epoch_snapshot_builds_pid_lookup_cache() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(55, 4),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 4,
                    placement_tid: tid(56, 4),
                },
            ],
        )
        .unwrap();
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 12, 12345, 4, tid(45, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 2048),
            ],
        )
        .unwrap();

        let snapshot = SpireValidatedEpochSnapshot::new(&epoch, &manifest, &directory).unwrap();
        let lookup = snapshot.require_lookup(12, "test").unwrap();

        assert_eq!(snapshot.snapshot().epoch_manifest.epoch, 7);
        assert_eq!(lookup.manifest_entry.object_version, 4);
        assert_eq!(lookup.placement.object_tid, tid(45, 2));
        assert!(snapshot.lookup(99).is_none());
    }

    #[test]
    fn published_epoch_snapshot_rejects_non_published_epoch() {
        let mut epoch = published_epoch(7, SpireConsistencyMode::Strict);
        epoch.state = SpireEpochState::Building;
        epoch.published_at_micros = 0;
        let manifest = object_manifest(7, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_err());
    }

    #[test]
    fn published_epoch_snapshot_rejects_epoch_mismatch() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let wrong_manifest = object_manifest(8, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &wrong_manifest, &directory).is_err());

        let manifest = object_manifest(7, 11, 3);
        let wrong_directory = placement_directory(8, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_directory).is_err());
    }

    #[test]
    fn published_epoch_snapshot_rejects_missing_or_version_mismatched_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);
        let wrong_pid_directory = placement_directory(7, 12, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_pid_directory).is_err());

        let wrong_version_directory = placement_directory(7, 11, 4, SpirePlacementState::Available);

        assert!(
            SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_version_directory).is_err()
        );

        let orphan_placement_directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 12, 12345, 4, tid(45, 2), 4096),
            ],
        )
        .unwrap();

        assert!(
            SpirePublishedEpochSnapshot::new(&epoch, &manifest, &orphan_placement_directory)
                .is_err()
        );
    }

    #[test]
    fn published_epoch_snapshot_rejects_non_available_placement_in_strict_mode() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);

        for state in [
            SpirePlacementState::Stale,
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let directory = placement_directory(7, 11, 3, state);
            assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_err());
        }
    }

    #[test]
    fn published_epoch_snapshot_degraded_mode_allows_unavailable_or_skipped_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Degraded);
        let manifest = object_manifest(7, 11, 3);

        for state in [
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let directory = placement_directory(7, 11, 3, state);
            assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_ok());
        }

        let stale_directory = placement_directory(7, 11, 3, SpirePlacementState::Stale);
        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &stale_directory).is_err());
    }
}
