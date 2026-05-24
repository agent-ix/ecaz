# Review Request: Callback IndexInfo Borrow

Task: 50 unsafe burndown

Commit under review:

- `2f80af40` - `Centralize callback IndexInfo borrowing`

## Summary

This packet centralizes AM callback `IndexInfo` borrowing.

- Adds `am::common::pg_ptr::index_info`, which borrows a checked `NonNull<IndexInfo>`.
- Updates IVF, SPIRE, HNSW, and DiskANN build/index validation helpers to use the shared borrow helper.
- Removes repeated caller-side `&*index_info` / `index_info.as_ref()` unsafe blocks from those AM paths.

## Unsafe / Guardrail Impact

- Current `src` direct unsafe count drops from `1314` to `1311`.
- Direct `IndexInfo` raw-borrow hits from the targeted AM paths are centralized in `src/am/common/pg_ptr.rs`.
- The broadened boundary-signature guard has no hits.

See `artifacts/unsafe-counts-and-guard.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1311` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Note: the compile log still includes the existing unused SPIRE DML re-export warning in `src/am/mod.rs`.

## Reviewer Focus

- Confirm the `NonNull<IndexInfo>` helper is an acceptable typed boundary and does not expose raw pointer safe APIs.
- Confirm the four AM build/validation paths still reject null `IndexInfo` before borrowing.
