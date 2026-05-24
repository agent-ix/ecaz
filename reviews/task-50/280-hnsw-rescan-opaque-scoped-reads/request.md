# Task 50 Review Request: HNSW Rescan Opaque Scoped Reads

## Summary

This slice converts three HNSW rescan debug helpers from direct
`unsafe { debug_scan_opaque(scan) }` reads to the existing scoped
`debug_with_scan_opaque` helper:

- `debug_rescan_query_dimensions`
- `debug_rescan_overwrites_query_dimensions`
- `debug_rescan_with_unused_key_buffer`

The raw scan-opaque dereference remains centralized in the closure helper, and
each caller now receives only a bounded borrow while copying the debug fields it
returns.

## Unsafe Burndown

- Previous broad count from packet 279: `2167`
- Current broad count: `2164`
- Net: `-3`

## Validation

Artifacts are under `reviews/task-50/280-hnsw-rescan-opaque-scoped-reads/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped; syntax/format viability was checked by cargo parsing and `git diff --check`
- `rescan-opaque-grep.log`: rescan query-dimension helpers now use `debug_with_scan_opaque`; remaining direct opaque reads are outside this slice
- `unsafe-count.log`: `2164`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
