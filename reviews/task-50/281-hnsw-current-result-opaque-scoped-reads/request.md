# Task 50 Review Request: HNSW Current Result Opaque Scoped Reads

## Summary

This slice converts another HNSW debug cluster from direct
`unsafe { debug_scan_opaque(scan) }` reads to `debug_with_scan_opaque`:

- `debug_gettuple_current_result_state`
- `debug_current_result_comparison_score`
- `debug_current_result_approx_score`
- `debug_current_result_approx_rank`
- `debug_rescan_entry_candidate_state`
- `debug_rescan_successor_candidate_state`

Each helper now copies the fields it needs inside a scoped opaque closure
instead of creating an unbounded raw-pointer borrow at the call site.

## Unsafe Burndown

- Previous broad count from packet 280: `2164`
- Current broad count: `2156`
- Net: `-8`

## Validation

Artifacts are under `reviews/task-50/281-hnsw-current-result-opaque-scoped-reads/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped; syntax/format viability was checked by cargo parsing and `git diff --check`
- `current-result-opaque-grep.log`: touched helpers now use `debug_with_scan_opaque`; remaining direct opaque reads are outside this slice
- `unsafe-count.log`: `2156`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
