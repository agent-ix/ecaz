# Review Request: SPIRE Scan Prefetch Placement Resolution

- Code commit: `1f6cc53c` (`Resolve SPIRE scan placements before prefetch`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement
- Agent: coder1

## Summary

This checkpoint tightens the store-grouped scan prefetch path and addresses the
follow-up note from `30519` about duplicate `require_lookup` calls.

The scan grouping layer now resolves placement and object-version metadata once:

- selected leaf routes become `SpireLeafObjectReadRoute` values carrying the
  leaf placement and object version;
- discovered delta routes carry their placement and object version from the
  header discovery pass;
- store-group prefetch consumes those resolved placements directly instead of
  re-looking up each PID;
- leaf and delta scoring also consume those resolved placements, avoiding another
  hot-path lookup during candidate reads.

The execution shape is now a cleaner two-step schedule:

1. group selected leaf/delta reads by `(node_id, local_store_id)` and prefetch
   every group;
2. score leaves using the already-resolved leaf/delta routes.

That matters because a selected leaf can still have delta routes in a different
store in older/debug fixtures. Prefetching every group before scoring prevents
the scoring loop from reading a cross-store delta before that store group has
been scheduled.

## Review Focus

1. Confirm that resolved route structs are the right place to carry placement
   metadata for the next parallel local fetch slice.
2. Check that moving all group prefetch ahead of scoring preserves existing
   scan semantics for leaf ordering and delta delete suppression.
3. Verify that this sufficiently addresses the `30519` duplicate
   `require_lookup` observation without over-expanding the scan abstraction.

## Validation

- `cargo test prefetch_store_object_read_groups --lib`
- `cargo test group_leaf_and_delta_reads_by_local_store --lib`
- `cargo test collect_quantized_routed_probe_candidates_reads_hash_routed_two_store_build --lib`
- `cargo fmt --check`
- `git diff --check`

PG18 `pgrx` tests were not run for this checkpoint; this is a pure scan helper
scheduling change covered by the focused Rust tests above.
