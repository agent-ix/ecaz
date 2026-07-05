# Review Request: Task 41 HNSW Shared Relation Guard Consolidation

## Summary

Task 41 relation-guard completeness slice.

The code commit `5a0649136cab9e44784dbdae38f699acb76bb709` removes the module-local `DebugAccessShareIndexRelation` from `src/am/ec_hnsw/shared.rs` and migrates its five debug-helper call sites to `IndexRelationGuard::access_share`:

- `debug_index_pages`
- `debug_planner_tuning_snapshot`
- `debug_index_metadata`
- `debug_update_index_metadata`
- `debug_vacuum_stats`

This directly addresses the 31179 and 31180 reviewer feedback that called out this HNSW-local guard as still unabsorbed by the shared relation primitive.

## Baseline Delta

- unsafe baseline entries: `4256 -> 4256`
- `src/am/ec_hnsw/shared.rs`: `106 -> 106`

This slice is baseline-neutral because the removed local guard already had SAFETY comments. The value is structural: one fewer module-local relation wrapper and one fewer copy of the pgrx-unwind close invariant.

See `artifacts/manifest.md` and `artifacts/validation.md`.

## Validation

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

## Review Focus

- Confirm `IndexRelationGuard::access_share` preserves the previous fixed AccessShareLock and error-on-null behavior.
- Confirm all raw `Relation` uses stay within the guard lifetime.
- Confirm this resolves the `DebugAccessShareIndexRelation` item from the remaining module-local relation guard list.
