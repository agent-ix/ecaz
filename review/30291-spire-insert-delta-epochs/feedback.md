# 30291 SPIRE Insert Delta Epochs — review

Code commit `90b207e9`. Read `src/am/ec_spire/insert.rs` (full diff and
post-change file), and the test `test_ec_spire_insert_after_build_delta_epoch`
at `src/lib.rs:3722-3761`.

## What landed

`aminsert` no longer errors out for populated indexes:

- Acquires `ShareUpdateExclusiveLock` on the index relation. This is the
  conventional vacuum-friendly mode (allows reads + autoanalyze, blocks DDL),
  and is *self-conflicting* — concurrent inserts serialize on the lock.
- Reads root control. Returns an explicit "not implemented yet" error if
  `active_epoch == 0`. (The empty-index bootstrap path lands in 30293.)
- Loads active epoch manifests, opens the relation object store, decodes
  `ecvector` / `tqvector` index tuple via the existing build helpers
  (`resolve_indexed_vector_kind`, `decode_heap_tid`,
  `build_spire_index_tuple`), and routes the source vector to a single
  closest leaf via `collect_snapshot_routed_leaf_rows` (top-1).
- Allocates one fresh `delta_pid`, builds a `SpireDeltaPartitionObject`
  with a single assignment (one `local_vec_seq` consumed), writes it via
  `store.insert_delta_object`.
- Builds the new placement directory by *cloning every existing placement
  entry* with `entry.epoch = new_epoch`, then appending the new delta
  placement. Persists manifest bundle + new root control.
- One epoch published per `aminsert` call.

## Correctness

- **Lock discipline.** `ShareUpdateExclusiveLock` is the right shape for
  this write path: it blocks DDL and other vacuum-class writers, allows
  reads to proceed against the previous active epoch. The
  `RelationLockGuard` Drop releases on function exit. The lock is
  acquired before reading root_control, so a concurrent vacuum that
  finished and bumped active_epoch can't slip between the read and the
  publish.
- **Routing.** `collect_snapshot_routed_leaf_rows` returns a single
  `routed.leaf_pid` — the top-1 closest leaf to the source vector. The
  delta is parented to that leaf via `SpireDeltaPartitionObject::new(
  delta_pid, new_epoch, base_pid, ...)`. Scans then probe deltas whose
  parent_pid is among the probed leaves, so an inserted row is visible
  on a search whose query routes to the same leaf — standard IVF-with-
  deltas semantics.
- **Manifest validation.** `epoch_manifest.validate()?` runs before
  publish, so an out-of-shape epoch manifest will fail before any
  on-disk state is updated.
- **Crash ordering.** The publish sequence is the same as
  vacuum-compaction (write objects → write manifest bundle →
  initialize root control). Same atomicity guarantee: a crash before
  the root_control swap leaves orphaned tuples but the previous active
  epoch is still live and queryable.
- **PID + local_vec_seq accounting.** Each insert consumes exactly one
  PID (for the new delta object) and one local_vec_seq (for the new
  assignment). The test confirms via `debug_spire_root_control`:
  `next_pid` advances from 4 to 5, `next_local_vec_seq` from 3 to 4.

## Concerns / scope notes

### One epoch per insert is going to bite

Each `aminsert` call publishes a new epoch. For a multi-row INSERT
(`INSERT ... VALUES (...), (...), (...)` or `INSERT ... SELECT`), each
row produces:

- A new delta partition object (PID consumed)
- A new placement entry
- A re-write of the placement directory (clone+bump every existing entry)
- A new manifest bundle
- A root_control swap

The placement directory clone is `O(placements)` per insert, so insert
cost grows linearly with active object count. A 1000-row INSERT into a
populated index produces 1000 epochs, 1000 delta objects, and ~1000²
placement-entry writes in aggregate. This will fall over before vacuum
can compact.

The packet acknowledges this: "This does not implement... insert
batching." But it's the kind of thing where users will notice
performance pain long before the batch-insert work lands. Worth flagging
as the next priority after the diagnostic surface stabilizes — either a
true batch-insert path that buffers within a single transaction, or a
multi-row delta object that accepts >1 assignment per epoch.

### Concurrent inserts serialize

`ShareUpdateExclusiveLock` is self-conflicting, so two concurrent
sessions inserting into the same index will block on each other (the
second waits for the first's transaction to commit). For OLTP-style
insert workloads this is a hard ceiling. Not a bug — likely the
intended foundation, since the publish step requires exclusive root_control
ownership — but the SPIRE design does eventually want batched concurrent
publishes, so this'll need to evolve.

### Test coverage

`test_ec_spire_insert_after_build_delta_epoch` covers the happy path:
build → insert → query, asserting epoch=2 and that the inserted row is
returned by an ORDER-BY scan against `[0.0, 1.0]`. It also implicitly
covers PID + local_vec_seq accounting via `debug_spire_root_control`.

Gaps:

- No multi-insert scenario (e.g., insert 5 rows post-build, verify
  active_epoch advances to 6 and all 5 are queryable). This is the
  scenario where the cost concerns above start to materialize.
- No insert that routes to the *same* leaf as a previous insert (which
  would produce two delta objects parented to the same base leaf —
  exercising the scan path's "multiple deltas per probed leaf"
  behavior).
- No insert + concurrent scan visibility test (the publish atomicity
  claim is asserted by design but not test-verified).
- No error-path coverage: bad vector dimension, NULL value, invalid
  payload format. The decode helpers are reused from build, so they
  presumably error consistently, but the aminsert-specific call sites
  go through `pgrx::error!("ec_spire aminsert failed: {e}")` and
  there's no test that the error string surfaces correctly.

## Style / minor

- The placement-directory rebuild via `placement_directory.entries.iter()
  .cloned().map(|mut entry| { entry.epoch = new_epoch; entry })` is
  doing a clone-then-mutate of every entry per insert. With the
  `O(placements)` cost noted above, this is the obvious
  hot-spot to cache or share once batching lands.

## Status

Foundation insert path lands cleanly. Lock and publish ordering are
correct; the test exercises end-to-end delta visibility. The two
performance-shape concerns (one epoch per insert; concurrent inserts
serialize) are explicitly out of scope per the packet, but they're
load-bearing for whatever workload demos happen before vacuum
compaction can catch up — flagging them so they don't get lost behind
the diagnostic-surface push.
