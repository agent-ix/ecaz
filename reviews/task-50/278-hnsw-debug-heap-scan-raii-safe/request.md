# Task 50 Review Request: HNSW Debug Heap Scan RAII Safe API

## Summary

This slice makes `debug_begin_heap_backed_scan` safe and removes the separate
unsafe `debug_end_heap_backed_scan` helper. `DebugHeapBackedScan` already owns
the `IndexScanGuard`, active snapshot, index relation guard, and heap relation
guard; normal `drop(scan_state)` now expresses the cleanup contract directly.

The change removes unsafe begin/end wrappers from all heap-backed HNSW debug
scan callers without changing scan lifetime ownership. The `IndexScanGuard`
field is declared before the dependency guards, so it is dropped first.

## Unsafe Burndown

- Previous broad count from packet 277: `2187`
- Current broad count: `2171`
- Net: `-16`

## Validation

Artifacts are under `reviews/task-50/278-hnsw-debug-heap-scan-raii-safe/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped; syntax/format viability was checked by cargo parsing and `git diff --check`
- `heap-scan-raii-unsafe-grep.log`: no unsafe heap-scan begin/end signature or call wrappers remain
- `unsafe-count.log`: `2171`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
