# Review Request: SPIRE Vacuum Live Count Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/390-spire-vacuum-live-count-boundary`
- code commit: `efc3da03a83cfdb9c18f86e7770a1102ce5c94d0`
- scope: `src/am/ec_spire/vacuum/mod.rs`

## Summary

This slice changes `collect_live_assignment_count` from an unsafe raw-relation helper to a safe internal helper that requires `SpireVacuumIndexRelation`.

The cleanup path and no-callback bulkdelete stats path now construct/use the typed vacuum relation wrapper before collecting visible assignments. This removes the direct caller unsafe block in `run_vacuum_cleanup` and keeps live-count collection behind the existing vacuum relation boundary.

## Unsafe Count

- before this slice: `1135`
- after this slice: `1134`

## Validation

Artifacts are packet-local under `reviews/task-50/390-spire-vacuum-live-count-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-vacuum.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1134`.
- `unsafe-ledger-generate.log`: generated 1134 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1134 current unsafe rows.
