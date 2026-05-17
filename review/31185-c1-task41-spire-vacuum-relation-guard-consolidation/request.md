# Review Request: Task 41 SPIRE Vacuum Relation Guard Consolidation

## Summary

Task 41 relation-guard completeness slice.

The code commit `abb3b8a86dd98f2ace230cc653c4392ea129cf9d` removes the module-local `ShareUpdateExclusiveIndexRelation` from `src/am/ec_spire/vacuum/mod.rs` and migrates both debug vacuum helpers to `IndexRelationGuard::open` with `ShareUpdateExclusiveLock`:

- `debug_spire_vacuum_remove_heap_tids`
- `debug_spire_vacuum_bulkdelete_heap_tids`

This directly addresses the 31176 and 31180 reviewer feedback that called out this SPIRE vacuum-local guard as superseded by the shared relation primitive.

## Baseline Delta

- unsafe baseline entries: `4256 -> 4256`
- `src/am/ec_spire/vacuum/mod.rs`: `35 -> 35`

This slice is baseline-neutral because the removed local guard already had SAFETY comments. The value is structural: one fewer module-local relation wrapper and one fewer copy of the ShareUpdateExclusive relation-close invariant.

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

- Confirm `IndexRelationGuard::open(..., ShareUpdateExclusiveLock, caller)` preserves the previous lock mode and error-on-null behavior.
- Confirm all raw `Relation` uses stay within the guard lifetime.
- Confirm this resolves the `ShareUpdateExclusiveIndexRelation` item from the remaining module-local relation guard list.
