# Task 97 Packet 021: Status Through Packet 020

This status-only packet records the Task 97 task-file refresh after packet 020.

## Changes

- `plan/tasks/97-tq-qjl-block-kernel-family.md` now lists the forced qjl32
  NEON parity hook and points at
  `reviews/task-97/020-qjl32-neon-forced-parity-hook/` as the latest evidence.
- `plan/tasks/README.md` now points the Task 97 index row at packet 020.

## Current State

Task 97 remains in review. The latest evidence now includes:

- `reviews/task-97/018-qjl32-octet-batch/`
  - qjl32 AVX2 8-candidate octet scoring after full block32 chunks.
  - HNSW QJL under-octet bypass to avoid losing end-to-end local parity.
- `reviews/task-97/020-qjl32-neon-forced-parity-hook/`
  - forced NEON unit-test hook for the future Graviton 4 NEON parity
    requirement.

Remaining gates are unchanged:

- Packet 020 reviewer disposition.
- Approved Graviton 4 runtime evidence: `Isa::Sve2`, measured vector length,
  real NEON parity execution, and direct `[block-kernel-counters]` rows.
- Final closeout matrix.

## Validation

- `git diff --check`: passed

No CI or AWS was run.

## Review Request

Please review this as a status-only packet confirming the task files now point
at packet 020 as the latest Task 97 evidence without changing the remaining
approval-gated closeout requirements.
