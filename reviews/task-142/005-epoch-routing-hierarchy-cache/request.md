# Review Request: Task 142 Packet 005

## Scope

This packet requests review for commit `d616663cdb38728f4f1c00dec8c4c53f24ac6f52`
(`Cache SPIRE routing hierarchy per epoch`) on branch
`task-142-spire-epoch-cache-overhead`.

The slice adds a backend-local routing hierarchy cache for the production
selection path:

- Adds `SpireRoutingHierarchyCacheKey { index_relid, active_epoch }`.
- Adds a one-entry backend-local cache for the loaded coordinator routing
  hierarchy, including the loaded top-graph object from packet 004.
- Switches production scan handoff/read/profile/threshold paths to use the
  cached selection helper keyed by `(index.relid(), root_control.active_epoch)`.
- Preserves strict epoch invalidation: a different active epoch or index relid
  misses and reloads.
- Updates profile load counters so cache hits report zero routing/top-graph
  loads.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test --lib collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy -- --nocapture`
- `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`

Both focused validations passed.

## Notes

This covers the coordinator routing hierarchy/top-graph cache portion of Task
142 Phase 1. Remaining Phase 1 work includes caching manifest/placement state
and moving planner cost callbacks to publish-time stats instead of per-query
object walks.
