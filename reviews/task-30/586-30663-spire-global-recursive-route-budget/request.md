# Review Request: SPIRE Global Recursive Route Budget

Phase 9.4 adds the core global recursive beam semantics so deep SPIRE routing
cannot multiply leaf routes by parent count.

Code checkpoint: `7714bfa3` (`Add SPIRE global recursive route budget`)

## Scope

- Adds `SpireRecursiveRouteBudget` with `beam_width`, `max_leaf_routes`, and
  `max_routing_expansions` derived from active leaf count and effective
  `nprobe`.
- Threads the route budget through scan-plan candidate collection and top-graph
  routed scans.
- Replaces per-parent leaf route concatenation with a globally scored frontier
  that carries accumulated path score between routing levels.
- Deduplicates child and leaf routes before storage reads, including the
  top-graph root-level path.
- Exposes the resolved route-budget guardrails through
  `ec_spire_index_options_snapshot(index_oid)` and documents the new columns.
- Updates Phase 9 task tracking while leaving per-level routing diagnostics
  open.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 route_recursive --lib`
- `cargo test --no-default-features --features pg18 recursive_route_budget --lib`
- `cargo test --no-default-features --features pg18 single_level_scan_plan --lib`
- `cargo test --no-default-features --features pg18 top_graph_object_routes_recursive_children_to_leaf_routes --lib`
- `cargo test --no-default-features --features pg18 collect_snapshot_top_graph_routed_probe_leaf_rows_uses_loaded_graph --lib`
- `cargo test --no-default-features --features pg18 index_options_snapshot --lib`
  matched 0 Rust tests; it still compiled the lib test target after the SQL
  return-shape change.

## Review Focus

- Confirm the score accumulation contract is the right first global frontier:
  route score is summed across routing levels, and top-graph distances become
  path scores via `-distance`.
- Check whether the default `max_routing_expansions = max(active_leaf_count,
  beam_width)` is the right finite guardrail until explicit reloptions/GUCs
  exist.
- Confirm that preserving `nprobe_per_level` as the local per-parent fanout
  input while making `beam_width`/`max_leaf_routes` the global caps matches the
  Phase 9 architecture intent.
- Flag any diagnostics that should be added before closing the remaining
  Phase 9.4 routing-diagnostics checkbox.
