#[cfg(test)]
mod tests {
    use super::{
        build_boundary_insert_delta_assignment_placements,
        build_boundary_leaf_assignment_placements,
        build_boundary_leaf_assignment_placements_with_identity, build_delete_delta_assignments,
        build_insert_delta_assignments, build_insert_delta_assignments_with_identity,
        build_primary_leaf_assignments, build_primary_leaf_assignments_with_identity,
        observe_assignment_vec_ids, SpireBoundaryLeafAssignmentIdentityInput,
        SpireBoundaryLeafAssignmentInput, SpireDeleteDeltaInput, SpireLeafAssignmentIdentityInput,
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
        SpireVecIdSourceIdentity, SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID,
        SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES,
    };
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireVecId, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
        SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE, SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    #[test]
    fn allocator_starts_at_first_local_sequence() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let first = allocator.allocate().unwrap();
        let second = allocator.allocate().unwrap();

        assert_eq!(first.local_sequence(), Some(SPIRE_FIRST_LOCAL_VEC_SEQ));
        assert_eq!(second.local_sequence(), Some(SPIRE_FIRST_LOCAL_VEC_SEQ + 1));
        assert_eq!(
            allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ + 2
        );
    }

    #[test]
    fn pid_allocator_starts_at_first_pid() {
        let mut allocator = SpirePidAllocator::default();

        let first = allocator.allocate().unwrap();
        let second = allocator.allocate().unwrap();

        assert_eq!(first, SPIRE_FIRST_PID);
        assert_eq!(second, SPIRE_FIRST_PID + 1);
        assert_eq!(allocator.next_pid(), SPIRE_FIRST_PID + 2);
    }

    #[test]
    fn pid_allocator_rejects_zero_next_pid_and_observed_zero() {
        assert!(SpirePidAllocator::new(0).is_err());

        let mut allocator = SpirePidAllocator::default();
        assert!(allocator.observe(0).is_err());
        assert_eq!(allocator.next_pid(), SPIRE_FIRST_PID);
    }

    #[test]
    fn pid_allocator_observes_pids_without_rewinding() {
        let mut allocator = SpirePidAllocator::new(10).unwrap();

        allocator.observe(20).unwrap();
        assert_eq!(allocator.next_pid(), 21);

        allocator.observe(5).unwrap();
        assert_eq!(allocator.next_pid(), 21);
    }

    #[test]
    fn pid_allocator_reports_exhaustion_without_advancing() {
        let mut allocator = SpirePidAllocator::new(u64::MAX).unwrap();

        assert!(allocator.allocate().is_err());
        assert_eq!(allocator.next_pid(), u64::MAX);
        assert!(allocator.observe(u64::MAX).is_err());
        assert_eq!(allocator.next_pid(), u64::MAX);
    }

    #[test]
    fn pid_allocator_reports_near_exhaustion_status() {
        let allocator = SpirePidAllocator::new(u64::MAX - 5).unwrap();

        let diagnostics = allocator.exhaustion_diagnostics(10);

        assert_eq!(diagnostics.next_value, u64::MAX - 5);
        assert_eq!(diagnostics.remaining_allocations, 5);
        assert!(diagnostics.near_exhaustion);
        assert!(!allocator.exhaustion_diagnostics(4).near_exhaustion);
    }

    #[test]
    fn allocator_rejects_zero_next_sequence() {
        assert!(SpireLocalVecIdAllocator::new(0).is_err());
    }

    #[test]
    fn allocator_observes_local_ids_without_rewinding() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        allocator.observe(&SpireVecId::local(20)).unwrap();
        assert_eq!(allocator.next_local_vec_seq(), 21);

        allocator.observe(&SpireVecId::local(5)).unwrap();
        assert_eq!(allocator.next_local_vec_seq(), 21);
    }

    #[test]
    fn allocator_rejects_observed_local_zero() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        assert!(allocator.observe(&SpireVecId::local(0)).is_err());
        assert_eq!(allocator.next_local_vec_seq(), SPIRE_FIRST_LOCAL_VEC_SEQ);
    }

    #[test]
    fn allocator_ignores_global_ids() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        allocator
            .observe(&SpireVecId::global(&[1, 2, 3]).unwrap())
            .unwrap();

        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn allocator_uses_global_source_identity_without_advancing_local_sequence() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        let vec_id = allocator
            .allocate_for_source_identity(&SpireVecIdSourceIdentity::StableGlobalPayload(vec![
                1, 2, 3,
            ]))
            .unwrap();

        assert_eq!(vec_id, SpireVecId::global(&[1, 2, 3]).unwrap());
        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn allocator_uses_fixed_global_source_identity_without_advancing_local_sequence() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();
        let payload = [7_u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES];
        let source_identity = SpireVecIdSourceIdentity::stable_fixed_global_payload(payload);

        let vec_id = allocator
            .allocate_for_source_identity(&source_identity)
            .unwrap();

        assert_eq!(vec_id, SpireVecId::global(&payload).unwrap());
        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn fixed_global_source_identity_rejects_wrong_width() {
        let short = vec![1_u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES - 1];
        let long = vec![1_u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES + 1];

        let short_err =
            SpireVecIdSourceIdentity::stable_fixed_global_payload_from_slice(&short).unwrap_err();
        let long_err =
            SpireVecIdSourceIdentity::stable_fixed_global_payload_from_slice(&long).unwrap_err();

        assert!(short_err.contains("must be 16 bytes"));
        assert!(long_err.contains("must be 16 bytes"));
    }

    #[test]
    fn allocator_rejects_invalid_global_source_identity_without_advancing() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        assert!(allocator
            .allocate_for_source_identity(&SpireVecIdSourceIdentity::StableGlobalPayload(vec![]))
            .is_err());
        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn allocator_reports_sequence_exhaustion_without_advancing() {
        let mut allocator = SpireLocalVecIdAllocator::new(u64::MAX).unwrap();

        assert!(allocator.allocate().is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
        assert!(allocator.observe(&SpireVecId::local(u64::MAX)).is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
    }

    #[test]
    fn local_vec_id_allocator_reports_near_exhaustion_status() {
        let allocator = SpireLocalVecIdAllocator::new(u64::MAX - 2).unwrap();

        let diagnostics = allocator.exhaustion_diagnostics(2);

        assert_eq!(diagnostics.next_value, u64::MAX - 2);
        assert_eq!(diagnostics.remaining_allocations, 2);
        assert!(diagnostics.near_exhaustion);
        assert!(!allocator.exhaustion_diagnostics(1).near_exhaustion);
    }

    #[test]
    fn build_primary_leaf_assignments_allocates_rows_in_order() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let rows = build_primary_leaf_assignments(
            &mut allocator,
            vec![
                SpireLeafAssignmentInput {
                    heap_tid: tid(10, 1),
                    payload_format: 1,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2],
                },
                SpireLeafAssignmentInput {
                    heap_tid: tid(10, 2),
                    payload_format: 1,
                    gamma: 0.75,
                    encoded_payload: vec![3, 4],
                },
            ],
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        assert_eq!(rows[0].vec_id.local_sequence(), Some(1));
        assert_eq!(rows[1].vec_id.local_sequence(), Some(2));
        assert_eq!(rows[0].heap_tid, tid(10, 1));
        assert_eq!(rows[1].encoded_payload, vec![3, 4]);
        assert_eq!(allocator.next_local_vec_seq(), 3);
    }

    #[test]
    fn build_primary_leaf_assignments_with_identity_accepts_global_ids() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        let rows = build_primary_leaf_assignments_with_identity(
            &mut allocator,
            vec![
                SpireLeafAssignmentIdentityInput {
                    assignment: SpireLeafAssignmentInput {
                        heap_tid: tid(10, 1),
                        payload_format: 1,
                        gamma: 0.5,
                        encoded_payload: vec![1, 2],
                    },
                    vec_id_source_identity: SpireVecIdSourceIdentity::StableGlobalPayload(vec![
                        9, 8, 7,
                    ]),
                },
                SpireLeafAssignmentIdentityInput {
                    assignment: SpireLeafAssignmentInput {
                        heap_tid: tid(10, 2),
                        payload_format: 1,
                        gamma: 0.75,
                        encoded_payload: vec![3, 4],
                    },
                    vec_id_source_identity: SpireVecIdSourceIdentity::AllocateLocal,
                },
            ],
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].vec_id, SpireVecId::global(&[9, 8, 7]).unwrap());
        assert_eq!(rows[1].vec_id.local_sequence(), Some(10));
        assert_eq!(allocator.next_local_vec_seq(), 11);
    }

    #[test]
    fn build_primary_leaf_assignments_rejects_invalid_locator_and_gamma() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        assert!(build_primary_leaf_assignments(
            &mut allocator,
            vec![SpireLeafAssignmentInput {
                heap_tid: ItemPointer::INVALID,
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .is_err());
        assert_eq!(allocator.next_local_vec_seq(), SPIRE_FIRST_LOCAL_VEC_SEQ);

        assert!(build_primary_leaf_assignments(
            &mut allocator,
            vec![SpireLeafAssignmentInput {
                heap_tid: tid(10, 1),
                payload_format: 1,
                gamma: f32::NAN,
                encoded_payload: vec![1, 2],
            }],
        )
        .is_err());
        assert_eq!(allocator.next_local_vec_seq(), SPIRE_FIRST_LOCAL_VEC_SEQ);
    }

    #[test]
    fn build_insert_delta_assignments_allocates_delta_insert_rows() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let rows = build_insert_delta_assignments(
            &mut allocator,
            vec![
                SpireLeafAssignmentInput {
                    heap_tid: tid(20, 1),
                    payload_format: 2,
                    gamma: 0.25,
                    encoded_payload: vec![9, 8],
                },
                SpireLeafAssignmentInput {
                    heap_tid: tid(20, 2),
                    payload_format: 2,
                    gamma: 0.5,
                    encoded_payload: vec![7, 6],
                },
            ],
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(rows[0].vec_id.local_sequence(), Some(1));
        assert_eq!(rows[1].vec_id.local_sequence(), Some(2));
        assert_eq!(rows[0].heap_tid, tid(20, 1));
        assert_eq!(rows[0].payload_format, 2);
        assert_eq!(rows[0].gamma, 0.25);
        assert_eq!(rows[1].encoded_payload, vec![7, 6]);
        assert_eq!(allocator.next_local_vec_seq(), 3);
        SpireDeltaPartitionObject::new(30, 4, 17, rows).unwrap();
    }

    #[test]
    fn build_insert_delta_assignments_with_identity_can_store_global_vec_ids() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        let rows = build_insert_delta_assignments_with_identity(
            &mut allocator,
            vec![SpireLeafAssignmentIdentityInput {
                assignment: SpireLeafAssignmentInput {
                    heap_tid: tid(20, 1),
                    payload_format: 2,
                    gamma: 0.25,
                    encoded_payload: vec![9, 8],
                },
                vec_id_source_identity: SpireVecIdSourceIdentity::StableGlobalPayload(vec![4, 5]),
            }],
        )
        .unwrap();

        assert_eq!(
            rows[0].flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(rows[0].vec_id, SpireVecId::global(&[4, 5]).unwrap());
        assert_eq!(allocator.next_local_vec_seq(), 10);
        SpireDeltaPartitionObject::new(30, 4, 17, rows).unwrap();
    }

    #[test]
    fn build_boundary_leaf_assignment_placements_reuses_vec_id_for_replicas() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let rows = build_boundary_leaf_assignment_placements(
            &mut allocator,
            vec![SpireBoundaryLeafAssignmentInput {
                primary_pid: 11,
                replica_pids: vec![12, 13],
                assignment: SpireLeafAssignmentInput {
                    heap_tid: tid(10, 1),
                    payload_format: 1,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2, 3],
                },
            }],
        )
        .unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].pid, 11);
        assert_eq!(rows[0].row.flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        assert_eq!(rows[1].pid, 12);
        assert_eq!(rows[1].row.flags, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA);
        assert_eq!(rows[2].pid, 13);
        assert_eq!(rows[2].row.flags, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA);
        assert_eq!(rows[0].row.vec_id, rows[1].row.vec_id);
        assert_eq!(rows[0].row.vec_id, rows[2].row.vec_id);
        assert_eq!(
            allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ + 1
        );
    }

    #[test]
    fn build_boundary_leaf_assignment_placements_with_identity_reuses_global_id_for_replicas() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        let rows = build_boundary_leaf_assignment_placements_with_identity(
            &mut allocator,
            vec![SpireBoundaryLeafAssignmentIdentityInput {
                primary_pid: 11,
                replica_pids: vec![12, 13],
                assignment: SpireLeafAssignmentIdentityInput {
                    assignment: SpireLeafAssignmentInput {
                        heap_tid: tid(10, 1),
                        payload_format: 1,
                        gamma: 0.5,
                        encoded_payload: vec![1, 2, 3],
                    },
                    vec_id_source_identity: SpireVecIdSourceIdentity::StableGlobalPayload(vec![
                        7, 7, 7,
                    ]),
                },
            }],
        )
        .unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].row.vec_id, SpireVecId::global(&[7, 7, 7]).unwrap());
        assert_eq!(rows[0].row.vec_id, rows[1].row.vec_id);
        assert_eq!(rows[0].row.vec_id, rows[2].row.vec_id);
        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn build_boundary_insert_delta_assignment_placements_sets_delta_flags() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let rows = build_boundary_insert_delta_assignment_placements(
            &mut allocator,
            vec![SpireBoundaryLeafAssignmentInput {
                primary_pid: 11,
                replica_pids: vec![12],
                assignment: SpireLeafAssignmentInput {
                    heap_tid: tid(10, 1),
                    payload_format: 1,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2, 3],
                },
            }],
        )
        .unwrap();

        assert_eq!(
            rows[0].row.flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(
            rows[1].row.flags,
            SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(rows[0].row.vec_id, rows[1].row.vec_id);
    }

    #[test]
    fn build_delete_delta_assignments_uses_existing_vec_ids() {
        let rows = build_delete_delta_assignments(vec![SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(99),
            heap_tid: tid(21, 3),
        }])
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].flags,
            SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        );
        assert_eq!(rows[0].vec_id.local_sequence(), Some(99));
        assert_eq!(rows[0].heap_tid, tid(21, 3));
        assert_eq!(rows[0].payload_format, 0);
        assert_eq!(rows[0].gamma, 0.0);
        assert!(rows[0].encoded_payload.is_empty());
        SpireDeltaPartitionObject::new(31, 4, 17, rows).unwrap();
    }

    #[test]
    fn build_delta_assignments_reject_invalid_locators() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        assert!(build_insert_delta_assignments(
            &mut allocator,
            vec![SpireLeafAssignmentInput {
                heap_tid: ItemPointer::INVALID,
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .is_err());
        assert_eq!(allocator.next_local_vec_seq(), SPIRE_FIRST_LOCAL_VEC_SEQ);

        assert!(build_delete_delta_assignments(vec![SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(99),
            heap_tid: ItemPointer::INVALID,
        }])
        .is_err());

        assert!(build_delete_delta_assignments(vec![SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(0),
            heap_tid: tid(10, 1),
        }])
        .is_err());
    }

    #[test]
    fn observe_assignment_vec_ids_advances_allocator() {
        let mut allocator = SpireLocalVecIdAllocator::default();
        let rows = build_primary_leaf_assignments(
            &mut allocator,
            vec![
                SpireLeafAssignmentInput {
                    heap_tid: tid(10, 1),
                    payload_format: 1,
                    gamma: 0.5,
                    encoded_payload: vec![1, 2],
                },
                SpireLeafAssignmentInput {
                    heap_tid: tid(10, 2),
                    payload_format: 1,
                    gamma: 0.75,
                    encoded_payload: vec![3, 4],
                },
            ],
        )
        .unwrap();

        let mut rebuilt = SpireLocalVecIdAllocator::default();
        observe_assignment_vec_ids(&mut rebuilt, &rows).unwrap();

        assert_eq!(rebuilt.next_local_vec_seq(), 3);
    }
}
