# Review Request: Task 41 SPIRE Maintenance Heap Relation Guard Consolidation

## Summary

Task 41 relation-guard completeness slice.

The code commit `86b21a12468bc357c711ab573daca950f27379fb` removes the module-local `SpireHeapRelationGuard` from `src/am/ec_spire/coordinator/lifecycle.rs`.

The SPIRE coordinator maintenance split path now uses a helper that resolves the heap relation OID from the index relation and returns the shared `HeapRelationGuard`. The maintenance call sites now use `heap_relation.as_ptr()` instead of the local guard's `relation()` method.

This addresses the `SpireHeapRelationGuard` item from the 31180 reviewer feedback.

## Baseline Delta

- unsafe baseline entries: `4256 -> 4254`
- `src/am/ec_spire/coordinator/lifecycle.rs`: dropped the two local table open/close SAFETY sites

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

- Confirm the heap OID resolution and failure messages match the prior local guard behavior.
- Confirm the shared `HeapRelationGuard` lifetime still covers vector attribute resolution, tuple-slot allocation, and split-replacement input building.
- Confirm this resolves the `SpireHeapRelationGuard` item from the remaining module-local relation guard list.
