# SPIRE Empty Insert Bootstrap

## Checkpoint

- Code commit: `ceb51abc` (`Bootstrap SPIRE empty index inserts`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: first insert into an empty active `ec_spire` epoch

## Summary

This checkpoint removes the hard error for inserting into an empty
relation-backed `ec_spire` index:

- `aminsert` now decodes the incoming heap tuple before active-epoch routing so
  the empty-index path can use the source vector as bootstrap input.
- If root/control reports `active_epoch = 0`, the insert path publishes epoch 1
  with a one-child root routing object and one segmented V2 base leaf object.
- The bootstrap root uses the inserted vector as its single centroid, allocates
  root PID and leaf PID from root/control cursors, and allocates the first local
  `vec_id` into the V2 leaf row.
- Later inserts into the same initially-empty index use the existing
  `DELTA_INSERT` replacement-epoch path.
- A focused PG18 test verifies empty root/control cursors, first-epoch
  bootstrap publication, second-insert delta publication, and ordered scan
  visibility through the `ec_spire` AM.

This does not implement insert batching, delta compaction, split/merge, or
multi-leaf retraining for indexes that start empty.

## Changed Files

- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/build.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_bootstraps_empty_index_epoch --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `200 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Insert batching, compaction, PQ-FastScan scorer binding, SQL/admin
  diagnostics, and full delete cleanup remain open.
