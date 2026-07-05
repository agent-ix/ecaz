# Review Request: SPIRE Live Index Guard Wrapper

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/394-spire-live-index-guard-wrapper`
- code commit: `aae781d4715a10cf56a6bcede804ea2a3ac96695`
- scope:
  - `src/am/ec_spire/coordinator/snapshots.rs`
  - `src/am/ec_spire/custom_scan/dml.rs`
  - `src/am/ec_spire/custom_scan/explain.rs`
  - `src/am/ec_spire/custom_scan/planner.rs`

## Summary

This slice adds `live_index_relation_from_guard`, a safe constructor for
`SpireLiveIndexRelation` when the caller already holds an `IndexRelationGuard`.

Three SPIRE CustomScan call sites now use the guard-backed constructor instead
of rebuilding a live-index relation from `index_relation.as_ptr()` inside
caller-side unsafe blocks:

- production DML output loading;
- CustomScan EXPLAIN context;
- planner eligibility over candidate SPIRE indexes.

The raw-pointer constructor remains `unsafe`; this only removes call-site unsafe
where the RAII guard already owns the relation lifetime.

## Unsafe Count

- before this slice: `1123`
- after this slice: `1121`
- movement: three caller-side unsafe blocks removed, one centralized guard
  boundary added.

## Plan Coverage

- Program: P2, PostgreSQL Handle Views
- Wave/tranche: Wave 2 SPIRE production fanout, CustomScan planner/executor
  relation wrapper reuse.
- Disposition: repeated caller unsafe for guard-owned relation pointers is
  absorbed into the named `SpireLiveIndexRelation` guard constructor.

## Validation

Artifacts are packet-local under
`reviews/task-50/394-spire-live-index-guard-wrapper/artifacts/`.

- `rustfmt-check.log`: passed; only existing stable-toolchain warnings for
  unstable rustfmt settings.
- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML
  re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: no matches.
- `src-unsafe-count.log`: `1121`.
- `unsafe-ledger-generate.log`: generated `1121` current `src` ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers `1121` current unsafe rows.
