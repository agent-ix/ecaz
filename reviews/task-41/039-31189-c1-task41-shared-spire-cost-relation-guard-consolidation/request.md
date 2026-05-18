# Task 41 Review Request: Shared/SPiRE Cost Relation Guard Consolidation

## Scope

This checkpoint removes the remaining planner-cost local `index_open` /
`index_close` pairs from:

- `src/am/common/cost.rs`
- `src/am/ec_spire/cost/mod.rs`

Both cost estimate callbacks now use the shared `IndexRelationGuard` with
`NoLock`, matching the planner relation-locking semantics already used by
the DiskANN/IVF cost slice in packet 31188.

Code commit: `4e1e50541313ab6e415f46c677b3ae9974554fd6`

## Safety Invariant

PostgreSQL still owns the relation cache entry returned by `index_open`.
`IndexRelationGuard` owns the matching `index_close` with the same
`NoLock` lockmode, and its lifetime encloses the call to
`compute_amcostestimate`.

This means pgrx error unwinds inside the guarded scope close the relation by
construction instead of depending on a manually paired close after the cost
calculation.

## Baseline Impact

Unsafe comment baseline remained stable:

- before: `4254`
- after: `4254`

The count is neutral because these files still contain other unsafe planner
callback and relation-inspection operations, but this removes four manual
open/close unsafe call sites from the cost-estimate path.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm `IndexRelationGuard::open(... NoLock ...)` is the right shared
  abstraction for planner cost callbacks.
- Confirm the guard lifetime remains wide enough for all relation metadata
  reads performed by `compute_amcostestimate`.
