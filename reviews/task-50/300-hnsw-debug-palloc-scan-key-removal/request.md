# Review Request: HNSW Debug Palloc Scan Key Removal

## Summary

This checkpoint removes the HNSW debug-only `DebugPallocScanKey` palloc/pfree
guard. The only caller passed the scratch scan key buffer to `index_rescan`
with `nkeys = 0`, so the code now uses a stack `ScanKeyData::default()` instead
of allocating a PostgreSQL scan key array that the callee does not consume.

This removes a small RAII helper and two raw allocation/free call sites from
`src/am/ec_hnsw/scan_debug.rs`.

## Code Commit

- `8130c68cb90bc1ed6fecbaf3f49dedf5c46675d0` - `Remove HNSW debug palloc scan key guard`

## Unsafe Count

- Previous packet baseline after packet 299: `2063`
- After this checkpoint: `2061`
- Net change: `-2`
- `src/am/ec_hnsw/scan_debug.rs` by-file match count: `24`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/300-hnsw-debug-palloc-scan-key-removal/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/300-hnsw-debug-palloc-scan-key-removal unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/300-hnsw-debug-palloc-scan-key-removal/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
