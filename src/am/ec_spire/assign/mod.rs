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

include!("tests.rs");
