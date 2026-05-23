# Review Request: HNSW Debug Heap Fetch Profile Helper Safe Surface

## Summary

This checkpoint centralizes the heap-backed debug profiling loop in
`src/am/ec_hnsw/scan_debug.rs`.

The change moves `index_rescan`, `index_getnext_slot`, `slot_getattr`, and
`ExecClearTuple` for `debug_profile_ordered_scan_with_heap_fetch` behind one
named helper, `debug_run_heap_fetch_profile_loop`. The helper owns the
scan/slot/order-by invariant for the profiling loop and removes the repeated
caller-side PostgreSQL slot unsafe blocks.

## Code Commit

- `0137baa69fec4db72a7685720282631051115b61` - `Centralize HNSW debug heap fetch profiling`

## Unsafe Count

- Previous packet baseline after packet 296: `2073`
- After this checkpoint: `2070`
- Net change: `-3`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1396 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/297-hnsw-debug-heap-fetch-profile-helper-safe/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/297-hnsw-debug-heap-fetch-profile-helper-safe unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/297-hnsw-debug-heap-fetch-profile-helper-safe/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
