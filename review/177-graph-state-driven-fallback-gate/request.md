# Review Request: Graph-State-Driven Fallback Gate

## Summary

- remove the separate `graph_traversal_seeded` flag from `src/am/scan.rs`
- derive the `amrescan` graph-vs-fallback decision from the live graph frontier state instead
- keep the A3 runtime shell closer to the actual graph/search-owned traversal state

## What changed

- `tqhnsw_amrescan(...)` now checks `graph_traversal_has_seeded_candidates(...)` instead of consulting a stored boolean
- added `graph_traversal_has_seeded_candidates(...)`, which derives readiness from the visible frontier / scheduler state through `candidate_frontier_head(...)`
- removed `graph_traversal_seeded` from `TqScanOpaque`
- stopped writing/resetting that flag in:
  - `reset_scan_position(...)`
  - `seed_bootstrap_trace(...)`
- updated unit coverage to assert:
  - reset clears the live graph frontier state
  - fallback gating now derives from real frontier state instead of a separate seeded marker

## Why

- A3 is making graph/search traversal the primary ordered runtime lane.
- A separate seeded boolean kept an older bootstrap-era control surface alive in `scan.rs`.
- The graph-vs-fallback branch now follows the actual traversal state that graph/search own, which reduces one more scan-owned legacy gate without touching planner activation or removing the linear fallback.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether deriving fallback entry from `candidate_frontier_head(...)` is the right runtime contract at `amrescan`
- whether removing `graph_traversal_seeded` makes the graph-first scan shell more coherent without overreaching
- whether this is the right next A3 step toward letting live scan control flow depend on graph/search-owned state instead of scan-owned bookkeeping flags
