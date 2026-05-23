---
task: 50
packet: reviews/task-50/231-spire-customscan-expression-list-view-boundary
head_sha: c9250b493ae623971d2c6207ea1f0675008efe45
timestamp: 2026-05-21T05:02:02-07:00
lane: SPIRE custom scan executor unsafe burndown
storage_format: n/a
rerank_mode: n/a
surface: CustomScan expression-list view
---

# Manifest

## Code Checkpoint

- Commit: `c9250b493ae623971d2c6207ea1f0675008efe45`
- Summary: moved DML CustomScan expression-list length and bounds handling onto the concrete `CustomScanExprList` view.
- Removed generic unsafe helper boundaries:
  - `custom_scan_list_len`
  - `custom_scan_list_nth_node`
- Source unsafe count:
  - Previous packet count: `2513`
  - This packet count: `2508`
  - Delta: `-5`

## Validation Artifacts

- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs src/am/ec_spire/custom_scan/cost_helpers.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed with no output.
- `artifacts/src-unsafe-count.log`
  - Command: `rg -n 'unsafe' src | wc -l`
  - Result: `2508`.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-custom-scan-pg18-pg-test-no-run.log`
  - Command: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- The change keeps the raw PostgreSQL List invariant attached to `CustomScanExprList::from_custom_scan`, rather than exposing generic helpers over arbitrary list pointers.
