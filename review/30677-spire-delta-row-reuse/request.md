# Review Request: SPIRE Delta Row Reuse

Code checkpoint: `11d9c721` (`Reuse SPIRE delta rows during scan`)

## Scope

- Advances Phase 10.4 by keeping `(node_id, local_store_id)` read groups as
  the scheduling unit and removing the duplicated selected-delta decode in the
  routed scan hot path.
- Adds `SpireLoadedDeltaObjectRoute`, loaded once per selected delta route, so
  delete suppression and insert candidate scoring reuse the same decoded rows.
- Preserves existing delete suppression ordering: all delete delta IDs for the
  selected leaf are collected before leaf and insert-delta candidates are
  appended.
- Keeps existing per-store scan diagnostics intact; `scanned_delta` and
  `delete_delta_row` are still emitted during the single delta load.
- Marks the completed Phase 10.4 grouping and delta row reuse checklist items.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 load_delta_rows_for_routes_reads_each_delta_object_once --lib`
- `cargo test --no-default-features --features pg18 group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test --no-default-features --features pg18 collect_quantized_routed_probe_candidates --lib`
- `cargo test --no-default-features --features pg18 prefetch_store_object_read_groups --lib`

## Review Focus

- Confirm delete suppression remains semantically identical after moving from
  two delta-object decodes to one loaded row set.
- Confirm the new loaded delta route does not weaken placement/parent
  validation or per-store diagnostics.
- Confirm the Phase 10.4 checklist update is appropriately scoped; per-store
  diagnostics expansion and local-store overlap/sequential-limit decisions
  remain open.
