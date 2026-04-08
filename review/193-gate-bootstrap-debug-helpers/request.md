# Review Request: Gate Bootstrap Debug Helpers

## Summary

- mark the remaining bootstrap-named scan helpers as test/debug-only in `src/am/scan.rs`
- make the live A3 runtime boundary explicit now that graph prefetch and refresh are cursor-owned
- keep runtime behavior unchanged

## What changed

- added `#[cfg(any(test, feature = "pg_test"))]` to the remaining compatibility helpers that are no longer used by the live runtime path:
  - `consume_candidate_frontier_head(...)`
  - `refill_bootstrap_frontier_after_success(...)`
  - `seed_scan_result_state(...)`
  - `prefetch_next_graph_traversal_result(...)`
- left the helpers available for unit tests and `scan_debug` / pg-test surfaces

## Why

- After the previous A3 slices, the live graph-first scan path goes through `GraphTraversalCursor` for readiness, prefetch, emit, refresh, and exhaustion handling.
- The remaining bootstrap-named helpers are still useful for debug/test inspection, but they are no longer part of the runtime execution path.
- This slice makes that boundary explicit so the A3 runtime surface is easier to reason about before handing off to A4 recall work.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the remaining bootstrap helper surface is correctly classified as test/debug-only
- whether the runtime graph-first boundary now looks clean enough to treat A3 as effectively closed
- whether any remaining non-test bootstrap helper in `scan.rs` still deserves runtime ownership review before A4
