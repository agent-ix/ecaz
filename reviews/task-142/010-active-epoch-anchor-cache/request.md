# Review Request: Active Epoch Anchor Cache

Task: 142 — SPIRE Epoch-Keyed Caching
Branch: `task-142-spire-epoch-cache-overhead`
Code commit: `bda865ffb9c06886a14ae3c4ed24411b096bf79f`

## Summary

This slice adds a backend-local active epoch anchor cache keyed by
`(index_oid, active_epoch)`.

`SpireLiveIndexRelation::active_epoch_anchor()` now reuses the decoded active
epoch manifest, object manifest, and placement directory for the current epoch.
The cache stores owned manifest metadata only; it does not retain relation
handles, object-store handles, or lock guards.

This covers the remote-side `ec_spire_remote_search*` candidate path that was
still re-reading and re-decoding active manifests on each invocation, and it
also reduces repeated active snapshot diagnostics that use the same anchor.

## Scope Notes

- This advances Task 142 Phase 2 remote/session snapshot reuse.
- Object-store-set caching remains deferred because those values are scoped to
  live PostgreSQL relation/lock lifetimes.
- The existing coordinator fanout manifest cache remains separate because that
  path uses coordinator-local placement rewriting semantics.

## Validation

Packet-local logs:

- `artifacts/cargo-test-active-epoch-anchor-cache.log`
  - `active_epoch_anchor_cache_reuses_epoch_anchor ... ok`
- `artifacts/cargo-test-production-read-profile-rollup.log`
  - `production_read_profile_row_preserves_metric_rollup ... ok`
- `artifacts/cargo-test-fanout-manifest-cache-regression.log`
  - `coordinator_fanout_manifest_cache_reuses_epoch_manifests ... ok`

`git diff --check` passed locally.
