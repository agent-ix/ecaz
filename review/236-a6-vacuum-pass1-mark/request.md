# Review Request: A6 Vacuum Pass 1 Mark

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `spec/functional/FR-010-hnsw-vacuum.md`
- `spec/functional/FR-022-vacuum-implementation.md`

This is the first narrow A6 checkpoint. It does not attempt graph repair or
finalization yet. The slice lands pass-1 dead-heap-TID stripping plus
live-element statistics, and it also writes down the separate insert-throughput
optimization follow-up that was only implicit before.

Checkpoint scope:

1. implement callback-driven pass-1 heap-TID stripping in `ambulkdelete`
2. make `amvacuumcleanup` report live-element counts instead of raw tuple tags
3. leave fully-dead elements at `heaptids = []`, `deleted = false` until later A6 passes
4. add regressions for duplicate-heaptid compaction, scan invisibility, and repeated pass-1 stability
5. update task/status/review docs and add an explicit insert-throughput follow-up task

## Scope

- `src/am/vacuum.rs`
- `src/am/shared.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/07-vacuum.md`
- `plan/tasks/README.md`
- `plan/tasks/13-insert-throughput.md`
- `plan/status.md`
- `review/README.md`

## What Landed

### 1. `ambulkdelete` now performs pass-1 dead-heap-TID stripping

The vacuum path is no longer a pure no-op when PostgreSQL supplies a bulk-delete
callback.

For each data page, the current implementation now:

1. scans element tuples under a page `SHARE` lock
2. uses the vacuum callback to decide which inline heap TIDs are dead
3. computes the post-pass-1 live-element count for that page
4. if a tuple needs rewriting, reopens that same page alone under
   `EXCLUSIVE` and rewrites only the compacted element payload through
   GenericXLog

This checkpoint intentionally keeps the write scope to one page at a time.
There is no new cross-page lock-ordering surface yet.

### 2. Fully-dead elements stop being scan-visible even before finalize

When pass 1 removes the last heap TID from an element, the tuple is left in the
intermediate state:

- `heaptids = []`
- `deleted = false`

That is deliberate for this slice. Existing graph/runtime code already skips:

- `element.deleted`
- elements whose `heaptids` array is empty

So pass 1 alone is enough to make a deleted row disappear from graph-seeded
runtime scan results without taking on pass-2 repair or pass-3 finalization in
the same jump.

### 3. Vacuum stats now count live elements, not raw element tags

`count_element_tuples(...)` and the cleanup stats path now decode element tuples
and count only:

- `!deleted`
- `heaptids.len() > 0`

That means `num_index_tuples` now tracks live graph nodes after pass 1 instead
of the old raw “number of element tags on disk” approximation.

### 4. Debug/test vacuum helpers now exercise real callback behavior

Test-only debug plumbing now exists to run the AM vacuum callbacks with an
explicit set of dead heap TIDs.

That keeps the regression surface narrow and deterministic inside `pg_test`
without depending on SQL `VACUUM` transaction semantics in the test harness.

### 5. Coverage locks in the pass-1 boundary

New regression coverage proves:

- duplicate-coalesced element tuples drop only the dead heap TID and keep the
  surviving one
- a fully-dead element becomes unreachable from graph-first scan results after
  pass 1
- replaying the same pass-1 dead-TID set is stable and does not rewrite again

The older no-callback vacuum stats tests remain in place, so this checkpoint
also preserves the “stats only” behavior when no callback is supplied.

### 6. The insert-throughput follow-up is now explicit in-tree

The new `plan/tasks/13-insert-throughput.md` captures the post-A5 optimization
work that was previously only discussed in chat / review notes:

- metadata-page drift-accounting contention
- tail-page append contention
- popular neighbor-page backlink hotspots

That task intentionally does **not** reopen ADR-026 lock ordering. It tracks
decontention of the hot path, not relaxation of the deadlock-safety rule.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `tests::pg_test_tqhnsw_vacuum_pass1_compacts_duplicate_heaptids`
- `tests::pg_test_tqhnsw_vacuum_pass1_makes_deleted_row_unreachable`
- `tests::pg_test_tqhnsw_vacuum_pass1_is_stable_across_repeated_replays`

## Review Focus

- Is the pass-1-only intermediate state (`heaptids = []`, `deleted = false`)
  a defensible checkpoint boundary before pass-2 repair / pass-3 finalize land?
- Are the new live-element stats semantics the right ones for
  `IndexBulkDeleteResult::num_index_tuples`, or is there a remaining accounting
  mismatch with PostgreSQL vacuum expectations?
- Does the one-page-at-a-time share-then-exclusive rewrite shape look like the
  right narrow lock protocol for A6 pass 1?
