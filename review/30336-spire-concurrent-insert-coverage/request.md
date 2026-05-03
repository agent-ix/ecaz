# SPIRE Concurrent Insert Coverage

## Checkpoint

- Code commit: `1ef9dae8`
  (`Cover SPIRE concurrent same-leaf inserts`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 2 concurrency validation, first focused same-leaf insert slice

## Summary

This checkpoint adds focused PG18 coverage for two external sessions inserting
into the same SPIRE leaf after a populated build.

The new test:

- builds a strict local `ec_spire` index with one active base leaf
- releases two `psql` workers from an advisory-lock barrier
- inserts two rows into the same routed leaf through normal SQL/index callbacks
- verifies root-control epoch and allocator serialization after both publishes
- verifies active leaf/delta assignment accounting
- forces an ordered index scan and confirms both inserted rows are reachable

The broader mixed insert/delete/scan stress item remains open. This slice only
proves concurrent same-leaf insert publication plus post-insert scan visibility.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test --lib test_pg18_ec_spire_concurrent_same_leaf_inserts --no-default-features --features pg18 -- --nocapture`
  - Result: `1 passed; 0 failed; 1116 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `236 passed; 0 failed; 881 filtered out`
- `git diff --check`

## Notes

- No replica behavior is implemented or tested here.
- Delete overlap and longer-running mixed concurrency remain future Task 30
  validation work.
