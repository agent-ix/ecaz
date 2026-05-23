# Review Request: SPIRE Relation Plan Boundaries

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `36770634cb733b8a9f10ad1cf42c0ab51395d7ec`

## Summary

This slice consolidates adjacent SPIRE local-store relation-plan unsafe boundaries in `src/am/ec_spire/storage/relation_plan.rs`.

- `spire_aux_store_reloptions()` now performs text allocation, datum wrapping, array construction, null checks, and final Datum conversion inside one documented PostgreSQL allocation boundary.
- The auxiliary heap creation path now releases the copied tuple descriptor in the same boundary as `heap_create_with_catalog`.
- Dependency recording and the following `CommandCounterIncrement()` now share one catalog-write boundary.
- No safe raw-pointer helper signatures were added.

Unsafe count movement:

- `src/am/ec_spire/storage/relation_plan.rs`: 10 -> 5 direct `unsafe {` blocks.
- `src`: 1174 -> 1169 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/storage/relation_plan.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1169 current unsafe rows`.

Artifacts are in `reviews/task-50/377-spire-relation-plan-boundaries/artifacts/`.
