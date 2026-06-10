# Task 97 Packet 023: Status Through Packet 022

This status-only packet records the Task 97 task-file refresh after packet 022
and a runbook command-shape correction.

## Changes

- `plan/tasks/97-tq-qjl-block-kernel-family.md` now points at
  `reviews/task-97/022-graviton4-closeout-runbook/` as the latest evidence.
- `plan/tasks/README.md` now points the Task 97 index row at packet 022.
- Packet 022's Graviton 4 runbook now includes the required
  `--profile <approved-graviton4-profile>` placeholder for both
  `ecaz cloud up` and `ecaz cloud install`.

## Current State

Task 97 remains in review. The latest evidence now includes:

- `reviews/task-97/020-qjl32-neon-forced-parity-hook/`
  - forced NEON unit-test hook for the future Graviton 4 NEON parity
    requirement.
- `reviews/task-97/022-graviton4-closeout-runbook/`
  - closeout runbook for the future approved Graviton 4 execution pass.

Remaining gates are unchanged:

- Packet 022 reviewer disposition.
- Approved Graviton 4 runtime evidence: `Isa::Sve2`, measured vector length,
  real NEON parity execution, and direct `[block-kernel-counters]` rows.
- Final closeout matrix.

## Validation

- `git diff --check`: passed

No CI or AWS was run. No local tests were needed for this status-only packet.

## Review Request

Please review this as a status-only packet confirming that the Task 97 task
files now point at packet 022 and that the runbook's future `ecaz cloud`
commands include the required profile placeholder.
