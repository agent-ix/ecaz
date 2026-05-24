# Review Request: HNSW Debug Scan Opaque Borrow Inline

## Summary

This checkpoint responds to the blocking feedback in
`reviews/task-50/298-hnsw-debug-scan-opaque-access-safe/feedback/2026-05-21-01-reviewer.md`.

The reviewer was correct: packet 298 made `debug_scan_opaque` and
`debug_scan_opaque_mut` safe helpers even though they accepted a raw
`IndexScanDesc` and returned borrowed references with an unbounded caller-chosen
lifetime. That regressed the round-1 anti-pattern B rule.

This patch applies the reviewer's Option B:

- deletes the two single-caller helpers;
- inlines the raw opaque dereferences into `debug_with_scan_opaque` and
  `debug_with_scan_opaque_mut`;
- keeps the lifetime bound by the `FnOnce` closure wrappers instead of exposing
  a safe raw-pointer-to-reference helper.

## Code Commit

- `026237fa062e9713d204018b1cfbc8860e3b5442` - `Inline HNSW debug scan opaque borrows`

## Unsafe Count

- Previous packet baseline after packet 303: `2061`
- After this checkpoint: `2061`
- Net change: `0`
- `src/am/ec_hnsw/scan_debug.rs` by-file match count remains `24`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/304-hnsw-debug-scan-opaque-borrow-inline/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/304-hnsw-debug-scan-opaque-borrow-inline unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/304-hnsw-debug-scan-opaque-borrow-inline/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

The cargo commands pass. The logs include the known pre-existing SPIRE
unused-import warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
