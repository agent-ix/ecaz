# Review Request: SPIRE Recursive Routing Diagnostics

This closes the remaining Phase 9.4 diagnostics requirement by exposing
per-level routing counters for one scan query.

Code checkpoint: `9e23d734` (`Expose SPIRE recursive routing diagnostics`)

## Scope

- Adds `ec_spire_index_scan_routing_snapshot(index_oid, query)` with one row
  per routing level.
- Reports `input_frontier_width`, `expanded_parent_count`,
  `selected_child_count`, `deduped_route_count`, and stable
  `truncation_reason` labels.
- Threads the diagnostic collector through the same recursive/top-graph route
  budget semantics used by scan routing.
- Enforces the top-graph recursive path's `beam_width` cap before descending.
- Changes runtime scan-plan resolution for candidates and scan-placement
  diagnostics to use true recursive leaf count instead of root child count, so
  budgets match the options snapshot.
- Marks Phase 9.4 complete in the Phase 9 task files.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 collect_scan --lib`
- `cargo test --no-default-features --features pg18 route_recursive --lib`
- `cargo test --no-default-features --features pg18 prepare_single_level_snapshot_scan_candidates --lib`
- `cargo test --no-default-features --features pg18 materialized_recursive_routing_epoch_scans_quantized_candidates --lib`
- `cargo test --no-default-features --features pg18 ec_spire_index_scan_routing_snapshot --lib`
  matched 0 Rust tests; it still compiled the lib test target after adding the
  SQL function.
- `cargo test --no-default-features --features pg18 single_level_scan_plan --lib`
- `cargo test --no-default-features --features pg18 collect_quantized_routed_probe_candidates --lib`
- `cargo test --no-default-features --features pg18 top_graph_object_routes_recursive_children_to_leaf_routes --lib`
- `cargo test --no-default-features --features pg18 collect_snapshot_top_graph_routed_probe_leaf_rows_uses_loaded_graph --lib`

## Review Focus

- Confirm the diagnostic counters map cleanly to the Phase 9.4 checklist:
  input frontier width, expanded parent count, selected child count, deduped
  route count, and truncation reason.
- Check that counting unselected routes for truncation labels without
  validating unselected children preserves routing behavior.
- Check the scan-plan leaf-count change. The options snapshot already used
  recursive leaf count; this makes runtime budget resolution match it.
- Confirm whether `ec_spire_index_scan_routing_snapshot` should remain a
  separate routing surface or be joined into placement diagnostics later.
