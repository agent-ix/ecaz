# Review Request: HNSW Debug Palloc And Order-By Score Guards

## Summary

This slice continues the HNSW debug unsafe burndown after packet 282.

Code commit: `6724b8bddfe6c154e9a961b7fd8708d9c77e6d72`

Changes:

- Added `DebugPallocScanKey`, a local RAII guard for the debug-only `ScanKeyData` allocation used by `debug_rescan_with_unused_key_buffer`.
- Removed the caller-local `unsafe { palloc0(...) }` and trailing `unsafe { pfree(...) }` pair from that debug helper.
- Added `debug_gettuple_orderby_score_slot` to centralize the one-off `xs_orderbynulls` / `xs_orderbyvals` null checks and dereferences used by `debug_gettuple_orderby_score`.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2148 -> 2145`.
- Replaced manual allocation cleanup at the call site with a drop guard.
- Collapsed repeated order-by slot pointer checks and dereferences into a single helper boundary.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/283-hnsw-debug-palloc-orderby-score-guards/artifacts/manifest.md`

