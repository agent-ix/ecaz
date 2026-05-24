# Task 50 Review Request: HNSW Debug Heap Scan Scoped Opaque Reads

## Summary

This slice extends the packet 278 RAII scan-state cleanup. `DebugHeapBackedScan`
now exposes:

- `as_ptr()`
- `with_opaque(...)`

The heap-backed profile helpers use the owner-scoped `with_opaque` closure
instead of pulling `TqScanOpaque` through `unsafe { debug_scan_opaque(scan) }`
at each read. The closure keeps the borrowed opaque lifetime bounded to the
read and leaves scan cleanup owned by `DebugHeapBackedScan`.

## Unsafe Burndown

- Previous broad count from packet 278: `2171`
- Current broad count: `2167`
- Net: `-4`

## Validation

Artifacts are under `reviews/task-50/279-hnsw-debug-heap-scan-scoped-opaque/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped; syntax/format viability was checked by cargo parsing and `git diff --check`
- `heap-scan-opaque-unsafe-grep.log`: heap-backed profile helpers now use `scan_state.with_opaque`; remaining matches are outside this slice
- `unsafe-count.log`: `2167`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
