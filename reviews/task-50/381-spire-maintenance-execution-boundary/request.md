# Review Request: SPIRE Maintenance Execution Boundary

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `c51bf73758157f45690d6731973e4090fe21caf4`

## Summary

This slice consolidates the SPIRE scheduled replacement maintenance publish execution boundary in `src/am/ec_spire/coordinator/maintenance.rs`.

- The selected scheduled-maintenance input build and `publish_relation_selected_scheduled_replacement_epoch()` call now run inside one documented boundary.
- The boundary is scoped to the publish-lock-held section where the selected plan, execution input, and replacement object/manifest writes all derive from the same live index relation and epoch snapshot.
- No safe raw-pointer helper signatures were added.

Unsafe count movement:

- `src/am/ec_spire/coordinator/maintenance.rs`: 4 -> 3 direct `unsafe {` blocks.
- `src`: 1159 -> 1158 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/coordinator/maintenance.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1158 current unsafe rows`.

Artifacts are in `reviews/task-50/381-spire-maintenance-execution-boundary/artifacts/`.
