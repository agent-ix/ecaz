# Task 35 Packet 039: IVF Scan Debug Tail Safety

## Code Under Review

- Commit: `bfe19421b59a461829853bd039526b0ee8896630`
- Scope: `src/am/ec_ivf/scan.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice finishes the remaining unsafe-comment debt in `src/am/ec_ivf/scan.rs`.
It documents the selected-probe-plan directory loader, EXPLAIN counter access,
and the `pg_test` debug helpers that open guarded relations, begin/end debug
scans, inspect scan-private state, read metadata, traverse directory entries,
and fetch debug gettuple output values.

Key safety boundaries documented:

- live IVF index relation and metadata directory-chain assumptions in
  `build_selected_probe_plan` and `load_directory_entries`
- PostgreSQL `IndexScanState`, `IndexScanDesc`, and AM-private opaque access in
  `explain_counters_from_index_scan_state`
- guarded heap/index relation and snapshot lifetimes in debug heap-backed scans
- debug AM begin/rescan/end scan ownership and order
- prepared-query and PQ FastScan model pointer reads from scan-owned opaque
  state
- executor-owned heap TID and order-by slot array access in debug gettuple
  output collection
- metadata and directory summary/entry reads under guarded index relations

## Baseline Accounting

- Global unsafe-comment baseline: `2763 -> 2720`
- `src/am/ec_ivf/scan.rs`: `43 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2763` global entries and `43 src/am/ec_ivf/scan.rs`.
- `artifacts/ivf-scan-baseline-before.log`: pre-slice IVF scan baseline entry
  list.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2720` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2720` global entries and no remaining `src/am/ec_ivf/scan.rs` entry.
- `artifacts/ivf-scan-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`:
  `cargo check --all-targets --no-default-features --features pg18,bench`
  completed successfully with the known unrelated warnings in
  `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `artifacts/final-diff.patch`: final review diff for the slice.
