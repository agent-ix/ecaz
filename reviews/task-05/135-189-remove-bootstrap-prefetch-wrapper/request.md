# Review Request: Remove Bootstrap Prefetch Wrapper

## Summary

- remove the stale bootstrap-named prefetch wrapper from the live graph-first scan surface
- point the debug/test materialization helper at the real graph-prefetch boundary instead
- keep behavior unchanged while making the remaining runtime boundary more honestly graph-first

## What changed

- made `prefetch_next_graph_traversal_result(...)` the explicit shared graph-prefetch boundary
- removed `materialize_next_bootstrap_frontier_result(...)`
- updated `src/am/scan_debug.rs` to call `prefetch_next_graph_traversal_result(...)` directly
- kept the rest of the graph prefetch/materialization behavior unchanged

## Why

- The previous slice already moved the live runtime path off the bootstrap-named wrapper.
- Leaving that wrapper behind kept an obsolete bootstrap/control-flow name in the graph-first scan surface even though the real boundary is now graph cursor prefetch.
- This is a small but real A3 cleanup: the debug/test surface now follows the same graph-prefetch boundary the runtime uses, which trims one more stale bootstrap layer out of `scan.rs`.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether removing the bootstrap-named wrapper is the right follow-through after collapsing the graph prefetch materialization boundary
- whether any remaining bootstrap naming in the live graph-first runtime path is still intentional
- whether the next useful A3 cut is now the search/frontier selection shell rather than more naming or wrapper cleanup
