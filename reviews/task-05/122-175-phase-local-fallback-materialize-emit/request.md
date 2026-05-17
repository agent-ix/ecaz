# Review Request: Phase-Local Fallback Materialize and Emit

## Summary

- make the linear fallback path own its materialize-plus-emit cycle in `src/am/scan.rs`
- keep graph traversal and fallback on distinct result-materialization boundaries
- relax one stale pg lifecycle assertion so staged A3 scans may either prefill the next graph result or clear current state on exhaustion

## What changed

- renamed the generic result-state seeding helper to `seed_scan_result_state(...)`
- added phase-local wrappers:
  - `materialize_graph_traversal_result(...)`
  - `materialize_linear_fallback_result(...)`
- added `emit_materialized_linear_fallback_result(...)` so the fallback path owns:
  - result-state materialization
  - first tuple emission
  - fallback-specific post-emit teardown
- updated the pg lifecycle contract in `src/lib.rs` to allow this staged A3 behavior after the last duplicate drain:
  - either another graph result is prefetched
  - or the graph lane exhausts and clears `current_result`

## Why

- Recent A3 work made graph traversal a prefetched ordered cursor.
- The fallback path was still partly using the generic shared result-materialization shell inline.
- This slice makes the live runtime boundary more explicit:
  - graph traversal seeds prefetched ordered results
  - fallback materializes on demand and immediately emits through its own phase-local helper
- The pg assertion update keeps the test focused on lifecycle correctness instead of assuming graph recall/continuation guarantees that A3 has not promised yet.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the fallback materialize-plus-emit helper is the right phase boundary in `src/am/scan.rs`
- whether the renamed shared seeding helper keeps graph and fallback responsibilities clearer without hiding state changes
- whether the pg lifecycle contract now matches the intended staged A3 behavior without weakening a guarantee we still want to enforce
