//! Leaf PID and vector-identity assignment helpers.

use super::storage::{
    SpireLeafAssignmentRow, SpireVecId, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
    SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
    SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
};
use crate::storage::page::ItemPointer;

pub(super) const SPIRE_FIRST_PID: u64 = 1;
pub(super) const SPIRE_FIRST_LOCAL_VEC_SEQ: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePidAllocator {
    next_pid: u64,
}

impl Default for SpirePidAllocator {
    fn default() -> Self {
        Self {
            next_pid: SPIRE_FIRST_PID,
        }
    }
}

impl SpirePidAllocator {
    pub(super) fn new(next_pid: u64) -> Result<Self, String> {
        if next_pid == 0 {
            return Err("ec_spire pid sequence 0 is invalid".to_owned());
        }
        Ok(Self { next_pid })
    }

    pub(super) fn next_pid(&self) -> u64 {
        self.next_pid
    }

    pub(super) fn allocate(&mut self) -> Result<u64, String> {
        let pid = self.next_pid;
        let next = pid
            .checked_add(1)
            .ok_or_else(|| "ec_spire pid sequence exhausted".to_owned())?;
        self.next_pid = next;
        Ok(pid)
    }

    pub(super) fn observe(&mut self, pid: u64) -> Result<(), String> {
        if pid == 0 {
            return Err("ec_spire observed pid 0 is invalid".to_owned());
        }
        let next = pid
            .checked_add(1)
            .ok_or_else(|| "ec_spire observed pid sequence is exhausted".to_owned())?;
        if next > self.next_pid {
            self.next_pid = next;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireLocalVecIdAllocator {
    next_local_vec_seq: u64,
}

impl Default for SpireLocalVecIdAllocator {
    fn default() -> Self {
        Self {
            next_local_vec_seq: SPIRE_FIRST_LOCAL_VEC_SEQ,
        }
    }
}

impl SpireLocalVecIdAllocator {
    pub(super) fn new(next_local_vec_seq: u64) -> Result<Self, String> {
        if next_local_vec_seq == 0 {
            return Err("ec_spire local vec_id sequence 0 is invalid".to_owned());
        }
        Ok(Self { next_local_vec_seq })
    }

    pub(super) fn next_local_vec_seq(&self) -> u64 {
        self.next_local_vec_seq
    }

    pub(super) fn allocate(&mut self) -> Result<SpireVecId, String> {
        let local_vec_seq = self.next_local_vec_seq;
        let next = local_vec_seq
            .checked_add(1)
            .ok_or_else(|| "ec_spire local vec_id sequence exhausted".to_owned())?;
        self.next_local_vec_seq = next;
        Ok(SpireVecId::local(local_vec_seq))
    }

    pub(super) fn observe(&mut self, vec_id: &SpireVecId) -> Result<(), String> {
        let Some(local_vec_seq) = vec_id.local_sequence() else {
            return Ok(());
        };
        if local_vec_seq == 0 {
            return Err("ec_spire observed local vec_id sequence 0 is invalid".to_owned());
        }
        let next = local_vec_seq
            .checked_add(1)
            .ok_or_else(|| "ec_spire observed local vec_id sequence is exhausted".to_owned())?;
        if next > self.next_local_vec_seq {
            self.next_local_vec_seq = next;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafAssignmentInput {
    pub(super) heap_tid: ItemPointer,
    pub(super) payload_format: u8,
    pub(super) gamma: f32,
    pub(super) encoded_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeleteDeltaInput {
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
}

pub(super) fn build_primary_leaf_assignments(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_allocated_assignment_rows(allocator, inputs, SPIRE_ASSIGNMENT_FLAG_PRIMARY)
}

pub(super) fn build_insert_delta_assignments(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_allocated_assignment_rows(
        allocator,
        inputs,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
    )
}

pub(super) fn build_delete_delta_assignments(
    inputs: Vec<SpireDeleteDeltaInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    let mut rows = Vec::with_capacity(inputs.len());
    for input in inputs {
        if input.heap_tid == ItemPointer::INVALID {
            return Err("ec_spire delete delta input heap_tid must be valid".to_owned());
        }

        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: input.vec_id,
            heap_tid: input.heap_tid,
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };
        row.encode()?;
        rows.push(row);
    }
    Ok(rows)
}

fn build_allocated_assignment_rows(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentInput>,
    flags: u16,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    let mut rows = Vec::with_capacity(inputs.len());
    for input in inputs {
        if input.heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment input heap_tid must be valid".to_owned());
        }
        if !input.gamma.is_finite() {
            return Err("ec_spire assignment input gamma must be finite".to_owned());
        }
        u32::try_from(input.encoded_payload.len())
            .map_err(|_| "ec_spire assignment input payload length exceeds u32".to_owned())?;

        let row = SpireLeafAssignmentRow {
            flags,
            vec_id: allocator.allocate()?,
            heap_tid: input.heap_tid,
            payload_format: input.payload_format,
            gamma: input.gamma,
            encoded_payload: input.encoded_payload,
        };
        row.encode()?;
        rows.push(row);
    }
    Ok(rows)
}

pub(super) fn observe_assignment_vec_ids(
    allocator: &mut SpireLocalVecIdAllocator,
    rows: &[SpireLeafAssignmentRow],
) -> Result<(), String> {
    for row in rows {
        allocator.observe(&row.vec_id)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_delete_delta_assignments, build_insert_delta_assignments,
        build_primary_leaf_assignments, observe_assignment_vec_ids, SpireDeleteDeltaInput,
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator,
        SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireVecId, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
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
    fn allocator_reports_sequence_exhaustion_without_advancing() {
        let mut allocator = SpireLocalVecIdAllocator::new(u64::MAX).unwrap();

        assert!(allocator.allocate().is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
        assert!(allocator.observe(&SpireVecId::local(u64::MAX)).is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
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
