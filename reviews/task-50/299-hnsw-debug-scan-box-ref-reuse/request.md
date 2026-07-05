# Review Request: HNSW Debug Scan Box Ref Reuse

## Summary

This checkpoint reuses the HNSW scan-owned pointer helper from debug code.

`scan_box_ref` is now visible within the HNSW module, and
`src/am/ec_hnsw/scan_debug.rs` uses it for scan-owned TID sets, prepared query
lengths, cached quantizer access, and prepared query access. This removes
duplicated raw pointer dereferences from debug helpers and centralizes that
contract in `src/am/ec_hnsw/scan.rs`.

## Code Commit

- `906f83c51f7bc1060edf911b914f4650c619383f` - `Reuse HNSW scan-owned pointer helper in debug`

## Unsafe Count

- Previous packet baseline after packet 298: `2066`
- After this checkpoint: `2063`
- Net change: `-3`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1391 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/299-hnsw-debug-scan-box-ref-reuse/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/299-hnsw-debug-scan-box-ref-reuse unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/299-hnsw-debug-scan-box-ref-reuse/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
