# Review Request: Graph Phase On-Demand Prefill

## Summary

- let the graph-first scan phase self-check and re-prefill ordered output inside `amgettuple`
- clear stale graph `current_result` state before trying to materialize the next ordered result
- remove the old graph-phase assumption that some earlier shell must always have left pending output ready

## What changed

- `prefill_graph_traversal_result(...)` now returns whether graph output is ready to emit
- added `graph_traversal_output_ready(...)` to make the graph-phase readiness contract explicit:
  - return ready when duplicate drain is already pending
  - clear stale `current_result` state when graph traversal has no pending output left
  - request a fresh materialization when the graph phase is still active
- `produce_next_graph_traversal_heap_tid(...)` now:
  - asks the graph phase to ensure output is ready
  - emits from that phase-local readiness path
  - no longer depends on the older “prefill must have happened earlier” assumption
- added unit coverage for clearing stale graph current-result state when no duplicate drain remains

## Why

- A3 has already made graph traversal the explicit primary ordered lane.
- The graph path was still depending on a staged shell invariant: pending output had to be prefetched before `amgettuple` re-entered graph result production.
- This slice moves more live execution responsibility into the graph phase itself:
  - graph phase decides whether output is ready
  - graph phase clears stale current-result state
  - graph phase triggers the next materialization when needed

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether graph-phase readiness and stale-current cleanup belong at this `scan.rs` boundary
- whether returning readiness from `prefill_graph_traversal_result(...)` makes the graph cursor contract clearer
- whether this is the right small A3 step toward reducing the remaining scan-owned graph execution shell
