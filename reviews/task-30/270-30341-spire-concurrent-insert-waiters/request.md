# SPIRE Concurrent Insert Waiter Polling

## Checkpoint

- Code commit: `c4ab9f7b`
  (`Stabilize SPIRE concurrent insert test`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Review feedback follow-up for packet `30336`

## Summary

This checkpoint removes the timing-only barrier from
`test_pg18_ec_spire_concurrent_same_leaf_inserts`.

The test now:

- documents the advisory-lock key convention used for packet-scoped
  concurrency barriers
- polls `pg_locks` until both psql worker sessions are waiting on the shared
  advisory lock
- releases the exclusive barrier only after both workers have reached the
  intended concurrent start point

This preserves the original coverage while making the test less dependent on
CI timing.

## Changed Files

- `src/lib.rs`

## Validation

- `cargo test --lib test_pg18_ec_spire_concurrent_same_leaf_inserts --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 1119 filtered out`
- `git diff --check`

## Notes

- This directly addresses the first follow-up in the packet `30336` review.
- Heterogeneous publish-lock concurrency coverage, such as insert racing with
  vacuum/delete, remains a separate future slice.
