# 30308 SPIRE Leaf Partition Diagnostics — review

Code commit `9764713a`. Read `index_leaf_snapshot` in
`mod.rs:830-967` and `test_ec_spire_leaf_snapshot_sql` in
`lib.rs:4153-4237`.

## Bug: manifest-iteration order can zero a leaf row's delta counters

`index_leaf_snapshot` builds `rows_by_leaf_pid: HashMap<u64,
SpireIndexLeafSnapshotRow>` by iterating `snapshot.object_manifest()
.entries` in arbitrary order. The two branches behave asymmetrically:

- **Leaf branch** (`mod.rs:861-883`):
  ```rust
  rows_by_leaf_pid.insert(header.pid, SpireIndexLeafSnapshotRow {
      ...,
      delta_object_count: 0,
      delta_insert_assignment_count: 0,
      delta_delete_assignment_count: 0,
      ...
  });
  ```
  Unconditional `insert` — replaces any existing entry, including
  one previously seeded by a Delta for the same parent_pid.

- **Delta branch** (`mod.rs:884-928`):
  ```rust
  let row = rows_by_leaf_pid.entry(header.parent_pid)
      .or_insert_with(|| SpireIndexLeafSnapshotRow {
          ..., placement_state: "missing_base_leaf", ...
      });
  row.delta_object_count = ...;
  row.delta_insert_assignment_count = ...;
  ```
  Uses `or_insert_with` and accumulates.

Failure mode: if a Delta is iterated before its parent Leaf in
`snapshot.object_manifest().entries`, the Delta branch first creates
a stub row with the delta counts populated, then the Leaf branch
overwrites that whole row — wiping `delta_object_count`,
`delta_insert_assignment_count`, `delta_delete_assignment_count`,
and `delta_object_bytes` to zero. The `effective_assignment_count`
recomputation at `mod.rs:957-962` then runs against the zeroed
delta fields and matches `base_assignment_count` exactly, with no
indication anything went wrong.

## Why the test doesn't catch it

`test_ec_spire_leaf_snapshot_sql` (`lib.rs:4153-4237`) does:

1. Build a 2-row index → manifest contains [leaf_A, leaf_B], no
   deltas. Asserts `delta_object_count = 0`. Passes regardless.
2. Insert one row post-build → publishes a replacement epoch.

In step 2, `encode_manifest_bundle_for_publish` writes the manifest
in the order the publish coordinator accumulated entries — for a
post-build insert this is "carried-forward leaves first, then the
new delta", so leaves precede deltas. The test asserts
`delta_object_count = 1`, and the sum-aggregate happens to be
correct because every leaf in the test gets its row first.

The bug fires on any path that produces Delta-before-Leaf ordering
in the active manifest. Today none of the foundation paths do that
(`build` writes leaves only, `insert` carries leaves forward then
appends deltas, `vacuum-compaction` rewrites leaves in place and
removes deltas). So this is currently latent — but the diagnostic
asserts a correctness contract over arbitrary manifest order, and
that contract is broken. The first split/merge implementation that
arranges manifests differently will hit it.

## Suggested fix

Either:

1. **Two-pass.** First pass: walk `manifest.entries`, populate
   leaves into `rows_by_leaf_pid` only. Second pass: walk again,
   populate deltas using `entry().or_insert_with(...)` and update
   in place. The Leaf branch in pass 2 can no-op or assert the row
   exists. This is the safest shape and reads cleanly.
2. **Make the Leaf branch order-safe.** Replace the unconditional
   `insert` with `entry().and_modify(...).or_insert_with(...)`,
   updating only the leaf-specific fields and leaving any
   accumulated delta counts untouched. Slightly trickier to read.

I'd take option 1.

## Inherited downstream

- **30309 leaf maintenance thresholds** consumes the same
  `index_leaf_snapshot` return value and computes split/merge
  recommendations from `effective_assignment_count`. A row whose
  delta inserts got zeroed would understate effective and could
  spuriously land in `merge_candidate` territory.
- **30310 insert batching debt** sums `row.delta_object_count`,
  `row.delta_insert_assignment_count`, and `max_delta_objects_per_leaf`
  from `index_leaf_snapshot` directly. A zeroed leaf row drops out
  of all three aggregates. `batching_recommended` could
  silently flip to false despite real batching debt existing.

Fixing 30308 fixes both downstream surfaces.

## Status

Lands as-is for the current foundation (no manifest path produces
the failing order today). But this is a latent correctness bug in
a diagnostic that's about to be relied on for split/merge
threshold decisions. Worth fixing before split/merge starts
exercising new manifest orderings — the diagnostic surface is the
operator's only visibility into whether maintenance fired against
correct numbers.
