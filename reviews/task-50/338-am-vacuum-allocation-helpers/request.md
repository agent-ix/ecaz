# Review Request: AM Vacuum Allocation Helpers

Task: 50 unsafe burndown

Commits under review:

- `d637f447` - `Centralize AM vacuum allocation helpers`
- `6b1025cf` - `Wrap AM vacuum stats allocation`

## Summary

This packet centralizes repeated PostgreSQL vacuum allocation contracts across the AM implementations.

- Adds `am::common::vacuum::alloc_index_bulk_delete_result` for zeroed `IndexBulkDeleteResult` allocation.
- Adds `am::common::vacuum::alloc_index_vacuum_info` for debug/test `IndexVacuumInfo` allocation.
- Uses the helpers in SPIRE, IVF, HNSW, and DiskANN vacuum/debug paths.
- Keeps the bulk-delete stats helper from exposing a raw PostgreSQL pointer in its safe public signature by returning an allocation wrapper and converting at the existing callback boundary.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1336` to `1327`.
- Direct AM `IndexBulkDeleteResult` / `IndexVacuumInfo` `alloc0` use is now centralized in `src/am/common/vacuum.rs`.
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~2..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1327` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the allocation wrapper avoids the safe-API raw pointer antipattern.
- Confirm each caller still initializes callback-required fields before invoking AM vacuum callbacks.
