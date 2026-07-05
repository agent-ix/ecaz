# Task 50 Review Request: Reloptions Layout Helper

## Summary

This slice advances P7 reloptions cleanup by centralizing repeated AM-specific reloptions layout casts behind a borrow-tied storage helper.

Code commit: `1695e4dafc5f7becf701ee76d7cc1ca5bf752a70`

Changes:

- Added `relation_options_layout_ref<T>(&NonNull<varlena>) -> &T` in `src/storage/relation.rs`.
- Converted DiskANN, HNSW, IVF/RaBitQ, and SPIRE reloptions views to store a non-null reloptions blob handle.
- Removed the four AM-local `unsafe { &*rd_options.cast::<...>() }` blocks.
- Removed unnecessary reloptions view lifetimes; typed layout borrows are now tied to `&self`.

Unsafe count:

- Before: `1226`
- After: `1223`
- Delta: `-3`

Targeted scan result:

- No remaining `let reloptions = unsafe { &*rd_options.cast::<...>() }` reloptions layout casts in the four AM options modules.

## Validation

Artifacts are under `reviews/task-50/359-reloptions-layout-helper/artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: `git diff --check` passed.
- `unsafe-count.log`: `1223`.
- `raw-boundary-guard.log`: no matches.
- `reloptions-cast-scan.log`: no matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check.log`: ledger regenerated and covers all `1223` current unsafe rows.
