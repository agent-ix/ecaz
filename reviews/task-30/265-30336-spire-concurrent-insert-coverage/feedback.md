# 30336 SPIRE Concurrent Insert Coverage — review

Code commit `1ef9dae8`. Read
`test_pg18_ec_spire_concurrent_same_leaf_inserts` in `lib.rs` and
the supporting `pg_test_psql_connection` / `spawn_psql_script`
helpers it reuses from prior packets.

## What the test exercises

Two psql workers race to insert into the same leaf after build.
Coordination is `pg_advisory_lock(BARRIER_KEY)` held exclusively by
the test; both workers grab a *shared* version and block. Test
sleeps 750 ms (long enough for both to reach the shared-lock wait),
releases the exclusive lock, and the workers proceed.

Asserted post-state:
- heap rows = 3 (initial + 2 inserts)
- `active_epoch = 3` (each insert advanced one epoch)
- `next_pid = 5`, `next_local_vec_seq = 4`
- leaf assignment count = 1, delta object count = 2, delta
  assignment count = 2
- `<#>` ranking returns both inserted ids in the top-3

## Honest assessment

The test verifies that two concurrent `INSERT` statements both
*succeed* and produce a deterministic post-state. That's
worth having. But "concurrent" here is mostly serialization: the
publish path holds the index extension lock plus the root-control
buffer lock, so the second worker's publish blocks until the first
finishes. The assertion `active_epoch = 3` (not 2) makes this
explicit — each insert was its own atomic publish; they did not
merge.

What's *not* tested:
- True interleaving of object/manifest/placement writes between
  workers. With the current locking discipline this is impossible
  by construction, which is the right contract — but the test
  doesn't prove the locks exist; it only proves the outcome you'd
  expect *if* they exist.
- Crash residue from a worker that aborted mid-publish. Current
  publish path is single-statement so any failure rolls back, but a
  test that injects a panic between "retired-manifest written" and
  "root/control advanced" would exercise the crash-residue path
  that 30307's design assumes is benign.
- A race against vacuum. Insert + vacuum-cleanup compete for the
  same publish lock. Adding a vacuum-during-insert worker would
  exercise the lock with a heterogeneous workload rather than two
  identical writers.
- Read concurrency under publish. A scanning third session reading
  the active epoch while two writers churn epoch numbers would
  verify that 30321's cache-refresh actually picks up new active
  epochs in a live session, end-to-end. Right now 30321 is verified
  only at the unit level.

## Style / minor

- `BARRIER_KEY = 303_360` (i.e. packet number ×10) is a clever
  collision-avoidance scheme but it's undocumented. A one-line
  comment ("test-unique advisory-lock id; conventionally
  `<packet>0`") would help future tests reuse the convention
  without reverse-engineering it.
- `std::thread::sleep(Duration::from_millis(750))` is the only
  timing knob. If CI is loaded, 750 ms might not be enough for both
  workers to reach `pg_advisory_lock_shared`. A safer pattern is to
  poll `pg_locks` for two waiters on `BARRIER_KEY` before releasing
  the exclusive lock. Not blocking, but worth knowing this test will
  occasionally appear racy under load.
- The final `<#>` query uses `LIMIT 3` then filters `id IN (1,2)` —
  asserting both inserted rows surface in top-3. Fine for a
  scannability check. Worth tightening to `LIMIT 2` once the
  ranking determinism is proven, so the assertion catches a future
  regression where one of the inserts slips out of the top of the
  ranking.

## Status

Lands cleanly. Closes the "did anything ever break under
concurrent inserts to the same leaf?" question, even though the
underlying mechanism is "the publish lock serializes them." Two
follow-ups worth queuing:

1. Replace the 750 ms sleep with a poll of `pg_locks` for waiters
   on the barrier key, to make the test deterministic under
   load.
2. Add a heterogeneous-workload variant: insert + vacuum (or
   insert + delete) racing for the publish lock. That's the
   workload that will exercise the lock ordering split/merge in
   30335 will inherit.
