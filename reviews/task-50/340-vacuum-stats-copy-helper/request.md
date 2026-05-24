# Review Request: Vacuum Stats Copy Helper

Task: 50 unsafe burndown

Commit under review:

- `65ea0caf` - `Centralize vacuum stats copying`

## Summary

This packet centralizes immediate copying of callback-returned `IndexBulkDeleteResult` stats.

- Adds `am::common::vacuum::copy_index_bulk_delete_result`, which takes a checked `NonNull<IndexBulkDeleteResult>` and returns an owned-by-value stats copy.
- Updates IVF, HNSW, and SPIRE debug vacuum helpers to use the shared copier instead of dereferencing `stats` directly.
- Keeps raw pointer conversion at the callback boundary; safe helper input is `NonNull`, not a raw PostgreSQL pointer.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1324` to `1320`.
- The targeted `unsafe { *stats }` / `&*stats` copy pattern no longer appears in the AM vacuum/debug paths scanned by the packet artifact.
- The broadened boundary-signature guard still has one remaining hit:
  - `src/am/ec_hnsw/options.rs`

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1320` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm `NonNull<IndexBulkDeleteResult>` avoids the raw-pointer safe API antipattern.
- Confirm each caller still copies stats before relation guards and PostgreSQL-owned callback state leave scope.
