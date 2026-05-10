//! Leaf PID and vector-identity assignment helpers.

use super::storage::{
    SpireLeafAssignmentRow, SpireVecId, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
    SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE, SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
    SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
};
use crate::storage::page::ItemPointer;

pub(super) const SPIRE_FIRST_PID: u64 = 1;
pub(super) const SPIRE_FIRST_LOCAL_VEC_SEQ: u64 = 1;
pub(super) const SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireAllocatorExhaustionDiagnostics {
    pub(super) next_value: u64,
    pub(super) remaining_allocations: u64,
    pub(super) near_exhaustion: bool,
}

impl SpireAllocatorExhaustionDiagnostics {
    fn from_next_value(next_value: u64, warn_within: u64) -> Self {
        let remaining_allocations = u64::MAX.saturating_sub(next_value);
        Self {
            next_value,
            remaining_allocations,
            near_exhaustion: remaining_allocations <= warn_within,
        }
    }
}

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

    pub(super) fn exhaustion_diagnostics(
        &self,
        warn_within: u64,
    ) -> SpireAllocatorExhaustionDiagnostics {
        SpireAllocatorExhaustionDiagnostics::from_next_value(self.next_pid, warn_within)
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

    pub(super) fn exhaustion_diagnostics(
        &self,
        warn_within: u64,
    ) -> SpireAllocatorExhaustionDiagnostics {
        SpireAllocatorExhaustionDiagnostics::from_next_value(self.next_local_vec_seq, warn_within)
    }

    pub(super) fn allocate(&mut self) -> Result<SpireVecId, String> {
        let local_vec_seq = self.next_local_vec_seq;
        let next = local_vec_seq
            .checked_add(1)
            .ok_or_else(|| "ec_spire local vec_id sequence exhausted".to_owned())?;
        self.next_local_vec_seq = next;
        Ok(SpireVecId::local(local_vec_seq))
    }

    pub(super) fn allocate_for_source_identity(
        &mut self,
        source_identity: &SpireVecIdSourceIdentity,
    ) -> Result<SpireVecId, String> {
        match source_identity {
            SpireVecIdSourceIdentity::AllocateLocal => self.allocate(),
            SpireVecIdSourceIdentity::StableFixedGlobalPayload(payload) => {
                SpireVecId::global(payload)
            }
            SpireVecIdSourceIdentity::StableGlobalPayload(payload) => SpireVecId::global(payload),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SpireVecIdSourceIdentity {
    AllocateLocal,
    StableFixedGlobalPayload([u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES]),
    StableGlobalPayload(Vec<u8>),
}

impl Default for SpireVecIdSourceIdentity {
    fn default() -> Self {
        Self::AllocateLocal
    }
}

impl SpireVecIdSourceIdentity {
    pub(super) fn stable_fixed_global_payload(
        payload: [u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES],
    ) -> Self {
        Self::StableFixedGlobalPayload(payload)
    }

    pub(super) fn stable_fixed_global_payload_from_slice(payload: &[u8]) -> Result<Self, String> {
        let payload_len = payload.len();
        let payload =
            <[u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES]>::try_from(payload).map_err(
                |_| {
                    format!(
                        "ec_spire stable global source identity payload length {payload_len} must be {SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES} bytes"
                    )
                },
            )?;
        Ok(Self::stable_fixed_global_payload(payload))
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
pub(super) struct SpireLeafAssignmentIdentityInput {
    pub(super) assignment: SpireLeafAssignmentInput,
    pub(super) vec_id_source_identity: SpireVecIdSourceIdentity,
}

impl SpireLeafAssignmentIdentityInput {
    pub(super) fn allocate_local(assignment: SpireLeafAssignmentInput) -> Self {
        Self {
            assignment,
            vec_id_source_identity: SpireVecIdSourceIdentity::AllocateLocal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeleteDeltaInput {
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireBoundaryLeafAssignmentInput {
    pub(super) primary_pid: u64,
    pub(super) replica_pids: Vec<u64>,
    pub(super) assignment: SpireLeafAssignmentInput,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireBoundaryLeafAssignmentIdentityInput {
    pub(super) primary_pid: u64,
    pub(super) replica_pids: Vec<u64>,
    pub(super) assignment: SpireLeafAssignmentIdentityInput,
}

impl SpireBoundaryLeafAssignmentIdentityInput {
    fn allocate_local(input: SpireBoundaryLeafAssignmentInput) -> Self {
        Self {
            primary_pid: input.primary_pid,
            replica_pids: input.replica_pids,
            assignment: SpireLeafAssignmentIdentityInput::allocate_local(input.assignment),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafAssignmentPlacement {
    pub(super) pid: u64,
    pub(super) row: SpireLeafAssignmentRow,
}

pub(super) fn build_primary_leaf_assignments(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_primary_leaf_assignments_with_identity(
        allocator,
        inputs
            .into_iter()
            .map(SpireLeafAssignmentIdentityInput::allocate_local)
            .collect(),
    )
}

pub(super) fn build_primary_leaf_assignments_with_identity(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentIdentityInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_allocated_assignment_rows(allocator, inputs, SPIRE_ASSIGNMENT_FLAG_PRIMARY)
}

pub(super) fn build_insert_delta_assignments(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_insert_delta_assignments_with_identity(
        allocator,
        inputs
            .into_iter()
            .map(SpireLeafAssignmentIdentityInput::allocate_local)
            .collect(),
    )
}

pub(super) fn build_insert_delta_assignments_with_identity(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireLeafAssignmentIdentityInput>,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    build_allocated_assignment_rows(
        allocator,
        inputs,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
    )
}

pub(super) fn build_boundary_leaf_assignment_placements(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireBoundaryLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentPlacement>, String> {
    build_boundary_leaf_assignment_placements_with_identity(
        allocator,
        inputs
            .into_iter()
            .map(SpireBoundaryLeafAssignmentIdentityInput::allocate_local)
            .collect(),
    )
}

pub(super) fn build_boundary_leaf_assignment_placements_with_identity(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireBoundaryLeafAssignmentIdentityInput>,
) -> Result<Vec<SpireLeafAssignmentPlacement>, String> {
    build_boundary_assignment_placements(
        allocator,
        inputs,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
    )
}

pub(super) fn build_boundary_insert_delta_assignment_placements(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireBoundaryLeafAssignmentInput>,
) -> Result<Vec<SpireLeafAssignmentPlacement>, String> {
    build_boundary_insert_delta_assignment_placements_with_identity(
        allocator,
        inputs
            .into_iter()
            .map(SpireBoundaryLeafAssignmentIdentityInput::allocate_local)
            .collect(),
    )
}

pub(super) fn build_boundary_insert_delta_assignment_placements_with_identity(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireBoundaryLeafAssignmentIdentityInput>,
) -> Result<Vec<SpireLeafAssignmentPlacement>, String> {
    build_boundary_assignment_placements(
        allocator,
        inputs,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
    )
}

fn build_boundary_assignment_placements(
    allocator: &mut SpireLocalVecIdAllocator,
    inputs: Vec<SpireBoundaryLeafAssignmentIdentityInput>,
    primary_flags: u16,
    replica_flags: u16,
) -> Result<Vec<SpireLeafAssignmentPlacement>, String> {
    let mut rows = Vec::new();
    for input in inputs {
        if input.primary_pid == 0 {
            return Err("ec_spire boundary assignment primary pid 0 is invalid".to_owned());
        }
        if input.replica_pids.contains(&input.primary_pid) {
            return Err(
                "ec_spire boundary assignment replica pids must not include primary pid".to_owned(),
            );
        }
        let mut seen_replica_pids = std::collections::HashSet::new();
        for replica_pid in &input.replica_pids {
            if *replica_pid == 0 {
                return Err("ec_spire boundary assignment replica pid 0 is invalid".to_owned());
            }
            if !seen_replica_pids.insert(*replica_pid) {
                return Err("ec_spire boundary assignment replica pids must be unique".to_owned());
            }
        }
        validate_assignment_input(&input.assignment.assignment)?;

        let vec_id =
            allocator.allocate_for_source_identity(&input.assignment.vec_id_source_identity)?;
        rows.push(SpireLeafAssignmentPlacement {
            pid: input.primary_pid,
            row: build_assignment_row(
                input.assignment.assignment.clone(),
                vec_id.clone(),
                primary_flags,
            )?,
        });
        for replica_pid in input.replica_pids {
            rows.push(SpireLeafAssignmentPlacement {
                pid: replica_pid,
                row: build_assignment_row(
                    input.assignment.assignment.clone(),
                    vec_id.clone(),
                    replica_flags,
                )?,
            });
        }
    }
    Ok(rows)
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
    inputs: Vec<SpireLeafAssignmentIdentityInput>,
    flags: u16,
) -> Result<Vec<SpireLeafAssignmentRow>, String> {
    let mut rows = Vec::with_capacity(inputs.len());
    for input in inputs {
        validate_assignment_input(&input.assignment)?;
        let vec_id = allocator.allocate_for_source_identity(&input.vec_id_source_identity)?;
        rows.push(build_assignment_row(input.assignment, vec_id, flags)?);
    }
    Ok(rows)
}

fn build_assignment_row(
    input: SpireLeafAssignmentInput,
    vec_id: SpireVecId,
    flags: u16,
) -> Result<SpireLeafAssignmentRow, String> {
    validate_assignment_input(&input)?;

    let row = SpireLeafAssignmentRow {
        flags,
        vec_id,
        heap_tid: input.heap_tid,
        payload_format: input.payload_format,
        gamma: input.gamma,
        encoded_payload: input.encoded_payload,
    };
    row.encode()?;
    Ok(row)
}

fn validate_assignment_input(input: &SpireLeafAssignmentInput) -> Result<(), String> {
    if input.heap_tid == ItemPointer::INVALID {
        return Err("ec_spire assignment input heap_tid must be valid".to_owned());
    }
    if !input.gamma.is_finite() {
        return Err("ec_spire assignment input gamma must be finite".to_owned());
    }
    u32::try_from(input.encoded_payload.len())
        .map_err(|_| "ec_spire assignment input payload length exceeds u32".to_owned())?;
    Ok(())
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
