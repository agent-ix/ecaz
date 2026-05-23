# Review Request: HNSW Debug Scan Pointer Helper Safe Surface

## Summary

This checkpoint centralizes several HNSW debug scan pointer reads in
`src/am/ec_hnsw/scan_debug.rs`.

The change adds a shared helper for scan-owned TID set snapshots, moves oracle
score-part pointer dereferences behind `debug_oracle_score_parts`, and makes
`debug_scan_heap_tid` a safe helper boundary. Callers no longer carry the raw
pointer dereference sites for these debug inspections.

## Code Commit

- `39af8f8cdc75c03aa87f1886323bb9eb5b7eb362` - `Centralize HNSW debug scan pointer reads`

## Unsafe Count

- Previous packet baseline after packet 293: `2086`
- After this checkpoint: `2081`
- Net change: `-5`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
