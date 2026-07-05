# Review Request: Coordinator Fanout Manifest Cache

Task: 142 — SPIRE Epoch-Keyed Caching
Branch: `task-142-spire-epoch-cache-overhead`
Code commit: `b59470cc35759fe4001da0b6854746ed9f3b3b88`

## Summary

This slice adds a backend-local coordinator fanout manifest cache keyed by
`(index_oid, active_epoch)`.

The cache stores only owned decoded metadata:

- `SpireEpochManifest`
- `SpireObjectManifest`
- `SpirePlacementDirectory`

It does not cache relation handles, object-store handles, or lock guards.
Production scan handoff/read/profile/threshold paths and the fanout planner now
reuse the cached decoded manifests before rebuilding call-scoped snapshots and
object-store sets.

## Scope Notes

- This is still Task 142 Phase 1 coordinator-side caching.
- It complements packet `005`'s routing hierarchy cache by removing the
  remaining repeated coordinator fanout manifest decode work on the production
  path.
- Cost callback publish-time stats and remote/session caches remain future
  slices.

## Validation

Packet-local logs:

- `artifacts/cargo-test-coordinator-fanout-manifest-cache.log`
  - `coordinator_fanout_manifest_cache_reuses_epoch_manifests ... ok`
- `artifacts/cargo-test-production-read-profile-rollup.log`
  - `production_read_profile_row_preserves_metric_rollup ... ok`
- `artifacts/cargo-test-routing-hierarchy-cache-regression.log`
  - `collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy ... ok`

`git diff --check` passed locally.

Formatting note: repo-wide `cargo fmt --check` and touched-file `rustfmt --check`
both report pre-existing formatting drift outside this slice; I did not apply
repo-wide formatting because it would touch unrelated files.
