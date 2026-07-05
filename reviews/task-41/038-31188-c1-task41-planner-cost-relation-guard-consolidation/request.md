# Review Request: Task 41 Planner Cost Relation Guard Consolidation

## Summary

Task 41 relation-guard completeness slice.

The code commit `59b040a5634adfc73298d1e3d36b0f38ebd95a4b` removes duplicate `OpenedCostIndexRelation` wrappers from:

- `src/am/ec_diskann/cost.rs`
- `src/am/ec_ivf/cost.rs`

Both planner cost callbacks now use `IndexRelationGuard::open(index_oid, NoLock, caller)` directly. This addresses the two `OpenedCostIndexRelation` items from the 31180 reviewer feedback.

## Baseline Delta

- unsafe baseline entries: `4254 -> 4254`

This slice is baseline-neutral because both local cost wrappers already had SAFETY comments. The value is structural: fewer module-local PostgreSQL relation wrappers and a shared audit point for the NoLock planner relation lifetime.

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

- Confirm `IndexRelationGuard::open(..., NoLock, caller)` preserves the prior NoLock planner relation lifetime.
- Confirm both cost callbacks still hold the relation guard across the full `compute_amcostestimate` call.
- Confirm this resolves the two cost-estimator `OpenedCostIndexRelation` items from the remaining module-local relation guard list.
