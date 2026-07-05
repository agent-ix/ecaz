# Review Request: HNSW Debug Scan Opaque Access Safe Surface

## Summary

This checkpoint makes the HNSW debug scan opaque accessors safe helper
boundaries in `src/am/ec_hnsw/scan_debug.rs`.

The raw scan opaque dereference still lives in `debug_scan_opaque` and
`debug_scan_opaque_mut`, but those helpers are no longer `unsafe fn`, and the
scoped closure helpers no longer need caller-side unsafe blocks. This preserves
the existing scoped-borrow contract while removing direct caller unsafe.

## Code Commit

- `bd4ee510ffe61fcd343a6d452fcc2ba686dbc9bc` - `Make HNSW debug scan opaque access safe`

## Unsafe Count

- Previous packet baseline after packet 297: `2070`
- After this checkpoint: `2066`
- Net change: `-4`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1394 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/298-hnsw-debug-scan-opaque-access-safe/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/298-hnsw-debug-scan-opaque-access-safe unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/298-hnsw-debug-scan-opaque-access-safe/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
