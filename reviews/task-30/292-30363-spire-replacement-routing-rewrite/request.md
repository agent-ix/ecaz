# SPIRE Replacement Routing Rewrite

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE Phase 2 update mechanics
- Scope: Pure parent-routing rewrite helper for replacement leaves

## Summary

This checkpoint adds the next pure helper boundary after packet `30362`.

`src/am/ec_spire/update.rs` can now rewrite a parent routing object for
split/merge/rebalance replacement leaves before any live scheduler or publish
wiring exists.

The helper:

- removes affected child PIDs from the parent routing object
- inserts replacement child PIDs and centroids at the first affected position
- preserves unaffected child order
- reassigns sequential centroid ordinals for the rewritten flat routing object
- carries root/internal parent identity while bumping the supplied object
  version
- rejects missing affected children, duplicate replacement PIDs, bad centroid
  shapes, non-finite centroids, and replacement PIDs that collide with
  unaffected children

This preserves the Phase 2 rule that split/merge are coverage rewrites over
immutable partition objects while keeping relation-backed publication deferred
to a later slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test routing_rewrite --lib`
- `git diff --check`

## Notes

- No scheduler, background worker, SQL entrypoint, or relation publish wiring
  is added here.
- PQ-FastScan populated support remains deferred.
- Remote placement and replicas remain deferred.
