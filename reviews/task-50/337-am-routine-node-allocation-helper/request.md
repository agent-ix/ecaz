# Review Request: AM Routine Node Allocation Helper

Task: 50 unsafe burndown

Commit under review:

- `d2f494f8` - `Centralize AM routine node allocation`

## Summary

This packet centralizes the PostgreSQL `IndexAmRoutine` node allocation boundary used by the AM handler builders.

- Adds `am::common::routine::alloc_index_am_routine`, which owns the `PgBox::<IndexAmRoutine>::alloc_node(T_IndexAmRoutine)` contract.
- Updates SPIRE, IVF, HNSW, and DiskANN routine builders to call the shared helper instead of allocating the PostgreSQL node directly.
- Keeps the existing AM callback assignment behavior unchanged.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1339` to `1336`.
- Direct `IndexAmRoutine` `alloc_node` usage is now centralized in `src/am/common/routine.rs`.
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1336` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `alloc_index_am_routine` is the right shared boundary for the PostgreSQL node allocation invariant.
- Confirm the four AM routine builders only changed allocation plumbing and did not alter callback fields.
