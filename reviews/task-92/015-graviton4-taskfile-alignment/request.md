# Task 92 Packet 015: Graviton 4 Task-File Alignment

## Summary

This checkpoint aligns the Task 92 task definition with the already-reviewed
ADR-076 and block-kernel development documentation for the AWS Graviton 4
target.

The task file still described the runtime ISA helper as
`enum Isa { Scalar, Neon, Sve, Avx2 }` and said Graviton 4 measurements may be
reported as SVE-256 when measured. That phrasing was stale after Packet 002
locked the target as AWS Graviton 4 / Neoverse V2 / SVE2 / measured vector
length.

## What Changed

- Updated `plan/tasks/92-cross-quant-block-kernel-infrastructure.md` so the
  task definition now:
  - includes `Sve2` in the `Isa` enum shape;
  - names AWS Graviton 4 as Neoverse V2 / SVE2;
  - says Graviton 4 packets target `Sve2` when available;
  - uses `sve2-128` as the current target-host measured-label example;
  - forbids width-specific claims from host-class inference alone.

## Validation

- `rg -n "Graviton 4 measurements may be reported|Graviton 4.*SVE-256|Graviton 4.*sve-256|enum Isa \\{ Scalar, Neon, Sve, Avx2 \\}" plan/tasks spec/adr docs crates src reviews/task-92 --glob '!reviews/task-92/*/feedback/*.md'`
  - only matched Packet 002's review-focus sentence asking reviewers to confirm
    the old implication was removed.
- `git diff --check`
  - passed.

## Review Focus

- Confirm the Task 92 task definition now matches ADR-076 and
  `docs/block-kernel-development.md` on Graviton 4: SVE2 dispatch, measured
  vector length, current target-host label `sve2-128`, and no Graviton 3 /
  SVE-256 assumption.
