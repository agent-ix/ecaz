# 30296 SPIRE Vacuum Delta Compaction — review

Code commit `9ec0aef5`. Read `src/am/ec_spire/vacuum.rs` (full diff plus
post-change context for `run_vacuum_cleanup`,
`publish_compacted_delta_epoch_if_needed`, `collect_visible_assignments`,
`run_bulkdelete`), the SQL test
`test_ec_spire_vacuum_delete_delta_suppresses_visible_row` in
`src/lib.rs:3791-3841`, and the task plan diff. This is a write-path
checkpoint, so I focused on data-integrity invariants.

## What landed

- New `run_vacuum_cleanup(index_relation)`: takes the vacuum publish
  lock, reads root control, returns 0 for empty active epoch; otherwise
  runs `publish_compacted_delta_epoch_if_needed` and reports a fresh
  live-assignment count.
- New `publish_compacted_delta_epoch_if_needed`:
  - Loads the active epoch manifests + a `SpireValidatedEpochSnapshot`.
  - First pass over the manifest discovers the set of base leaf PIDs
    that have at least one Delta object referencing them
    (`affected_base_pids`).
  - If `affected_base_pids` is empty, returns `Ok(false)` (no compaction
    needed).
  - Otherwise re-walks via `collect_visible_assignments`, filters to
    rows whose `base_pid` is in the affected set, clears the transient
    `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT` bit, and groups rows by
    `base_pid`.
  - Computes a new epoch as `active_epoch + 1` (overflow-checked).
  - Walks the manifest a third time to assemble the new placement
    directory:
    - Delta placements: dropped.
    - Leaf placements whose PID is in `affected_base_pids`: rewritten
      via `store.insert_leaf_object_v2_from_rows` with the compacted
      rows and `object_version + 1`. Always V2.
    - Root, Internal, and unaffected Leaf placements: cloned and
      re-stamped with `epoch = new_epoch` (no on-disk rewrite).
  - Validates `compacted_base_pids == affected_base_pids` and that
    `compact_rows_by_base_pid` is empty after the loop. Both fire as
    explicit error strings if the invariants don't hold.
  - Persists the new placement directory, manifest bundle, and root
    control via the existing publish helpers.
- `VacuumVisibleAssignment` now carries the full `SpireLeafAssignmentRow`
  (was just `vec_id` + `heap_tid`); the bulkdelete path was updated to
  read the assignment fields through the new shape.
- `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT` is now imported into vacuum.rs so
  the flag-clear can happen during compaction.

## Correctness

- **Lock discipline.** `run_vacuum_cleanup` and `run_bulkdelete` both
  acquire `lock_vacuum_publish_relation(index_relation)` (exclusive
  `RelationLockGuard`). The guard's `Drop` releases on function exit.
  Sequential bulkdelete → cleanup is safe; concurrent vacuum or insert
  cannot interleave because both paths hold the same lock.
- **Crash atomicity.** The publish order is right: write all object
  pages and the manifest bundle first, then atomically swap the root
  control page (`initialize_root_control_page`). If we crash partway
  through the V2 leaf rewrites or the manifest write, the old root
  control still points at the previous epoch and the orphan tuples are
  dead state for a future retention sweep. The packet acknowledges
  physical reclamation is still open.
- **Flag-clear before rewrite.**
  `assignment.flags &= !SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;` strips the
  delta-insert marker before the row gets written into the V2 base leaf.
  Correct: the row is no longer "delta-resident" after compaction.
- **Empty-leaf rewrite.** If every visible row for an affected leaf was
  deleted, `compact_rows_by_base_pid.remove(&header.pid)` returns
  `None` and `unwrap_or_default()` produces an empty `Vec`, which
  `insert_leaf_object_v2_from_rows` writes as an empty V2 leaf. This is
  the correct semantic ("the leaf still exists but holds no assignments
  this epoch"), and the test exercises it (the leaf containing id=2
  ends up empty after compaction, while the leaf containing id=1 is
  carried forward unchanged).
- **`affected_base_pids` integrity check.** The
  `compacted_base_pids != affected_base_pids` guard catches the case
  where a Delta references a `parent_pid` that does not correspond to
  an active Leaf in the manifest. This is the right failure: it means
  the input manifest is inconsistent and we shouldn't silently lose
  the delta's rows.
- **Leftover-rows guard.** `!compact_rows_by_base_pid.is_empty()` after
  the leaf loop means we accumulated rows for a base_pid that wasn't
  found as a Leaf. Same kind of integrity check; pairs with the prior
  one to ensure every visible delta-fed row finds a leaf to land in.
- **`header.pid` vs `manifest_entry.pid`.** The leaf-rewrite branch
  uses `header.pid` for both the lookup key and the `insert_leaf_object_v2_from_rows`
  call. For Leaf entries these should match (header is read from the
  same placement and validated upstream). Not a bug, but
  `manifest_entry.pid` would be just as correct and avoids the implicit
  trust in `header.pid == manifest_entry.pid`.

## Issues

### Dead branching in `run_vacuum_cleanup`

```rust
if unsafe { publish_compacted_delta_epoch_if_needed(index_relation, root_control)? } {
    return collect_live_assignment_count(index_relation);
}
collect_live_assignment_count(index_relation)
```

Both branches do the same thing. The `bool` return value is consumed
but never affects behavior. This should collapse to:

```rust
unsafe { publish_compacted_delta_epoch_if_needed(index_relation, root_control)? };
collect_live_assignment_count(index_relation)
```

Trivial fix; the current shape will read as confusing on re-encounter.

### Manifest is walked three times

`publish_compacted_delta_epoch_if_needed` walks `snapshot.object_manifest().entries`
three times: once to find affected base PIDs (reading every header),
once inside `collect_visible_assignments` to pull rows (reading every
delta and leaf body), and once to build the new placement directory
(reading every header again). That's three header reads per object
plus per-leaf and per-delta body reads. Functionally fine, but for a
vacuum cleanup on a large index this is roughly 3× the I/O it could
be. Worth a follow-up to fold pass-1 into the visible-assignment walk
or to cache headers between passes once the path is stable.

## Test coverage

- `test_ec_spire_vacuum_delete_delta_suppresses_visible_row` is a
  meaningful integration test:
  - Builds an index with 2 vectors (1 per leaf with `nlists=2`).
  - Calls `debug_spire_vacuum_remove_heap_tids` on id=2's heap_tid,
    triggering bulkdelete (publishes delete-delta epoch) followed by
    cleanup (publishes compacted epoch).
  - Asserts `active_epoch = 3` (build=1, bulkdelete=2, cleanup=3),
    `tuples_removed = 1.0`, `num_index_tuples = 1.0`,
    `leaf_assignment_count = 1`, `delta_object_count = 0`. The
    `delta_object_count = 0` after `active_epoch = 3` is the proof
    that compaction actually ran and dropped the delta from the active
    placement directory.
  - Includes a query check (`ORDER BY embedding <#>`) that returns
    id=1, confirming the live row is still queryable post-compaction.
  - `next_pid = 5` and `next_local_vec_seq = 3` confirm no new PIDs or
    local_vec_seqs were consumed during compaction (the leaf rewrite
    reuses the same PID, just bumps `object_version`).

### Gaps worth flagging before this is treated as production

- **Insert-only delta path.** No test exercises an insert that creates
  an insert-delta epoch followed by vacuum (without any deletion).
  Compaction should fold the insert-delta rows into the leaf body and
  drop the delta object. The current test only uses delete-delta. The
  mechanics are the same code path, but "no delete" is a distinct
  scenario.
- **Mixed insert + delete on same leaf.** A leaf with both an
  insert-delta (new row) and a delete-delta (deleting an existing
  base row) on it. Compaction should produce a body containing the
  surviving base rows + the insert (minus any deleted rows), all with
  the delta-insert flag cleared. Not exercised.
- **Multiple delta epochs accumulated before vacuum.** If insert
  deltas land across several epochs without intermediate cleanup,
  there can be more than one delta object per base leaf. The
  diagnostic confirms `delta_object_count` would be >1 in that
  scenario; the compaction should still produce a single rewritten
  leaf. Not exercised.
- **No-delta cleanup is no-op.** The
  `if affected_base_pids.is_empty() { return Ok(false); }` early-out
  is unexercised because the test always has deltas. A trivial
  "vacuum on an index with no inserts since build" check would close
  this.
- **V1→V2 conversion.** Compaction always writes V2 leaves
  (`insert_leaf_object_v2_from_rows`). If a leaf was V1 before, the
  rewrite is also a format upgrade. No test exercises a pre-existing
  V1 leaf going through compaction; the build path here probably
  produces V2 already, but the assumption isn't made explicit and
  could change.
- **Integrity-check error paths.** The `affected_base_pids !=
  compacted_base_pids` and leftover-rows checks fire on malformed
  state that the public path can't construct. Worth a unit test that
  builds a synthetic snapshot with a delta whose `parent_pid` doesn't
  match an active leaf, just to confirm the error message surfaces
  cleanly instead of panicking somewhere inside the placement-write
  pipeline.

## Style / minor

- The early-return on dead branch (above) is the main thing.
- `collect_visible_assignments` is now called from both `run_bulkdelete`
  and `publish_compacted_delta_epoch_if_needed`. That sharing is good;
  it means the visibility filter — including delete-delta suppression —
  is the same in both phases of vacuum.
- The "rewrite leaf as V2" choice is a one-way ratchet: any V1 leaf
  touched by an insert/delete becomes V2. That's probably the desired
  end state, but worth a comment if there's ever a need to walk it
  back.

## Status

Lands cleanly with strong invariant guarding (paired
`affected/compacted` and leftover-rows checks). The integration test
proves the end-to-end delete→compact→query story. Two things I'd want
closed before treating this as production-grade: (1) the dead `if`
branch in `run_vacuum_cleanup`, and (2) coverage for the insert-only
and mixed-delta scenarios since those are the same code path but a
distinct branch of the state space. Multi-pass manifest walking is
optimizable but functionally fine.
