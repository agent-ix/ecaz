# Task 65b Packet 009: Epoch Snapshot Copy-On-Write

## Summary

This checkpoint optimizes the Task 65b deterministic epoch proposal path by replacing full `BuilderNeighborCache` clones with copy-on-write row snapshots.

Before this change, every parallel epoch cloned the entire `Vec<Vec<u32>>` adjacency store before proposal fanout. That preserved the immutable-snapshot design but made Slice F batch/worker tuning measure clone overhead as much as proposal/reducer behavior.

After this change:

- live build adjacency rows are stored as `Arc<[u32]>`;
- `BuilderNeighborCache::snapshot()` creates an immutable `Arc<[Arc<[u32]>]>` view for proposal workers;
- reducer writes replace touched live rows, leaving existing snapshots unchanged;
- finalization converts the rows back to `VamanaGraph { neighbors: Vec<Vec<u32>> }`, so persistence and diagnostics keep their existing shape.

## Why This Advances Task 65b

Slice F needs meaningful sweeps over `parallel_build_batch_size` and worker count. Full row cloning per epoch makes small-batch runs artificially expensive and can hide whether the ordered reducer is the real Amdahl bottleneck. This packet keeps the approved Slice C/E concurrency model but makes epoch snapshots cheap enough for the next tuning packet to be interpretable.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_`
- `cargo check -p ecaz --lib --no-default-features --features pg18`

The focused test run passed `4` Task 65b Vamana tests, including the new invariant:

- `task65b_snapshot_rows_do_not_observe_later_reducer_writes`

## Evidence

- Manifest: `reviews/task-65b/009-epoch-snapshot-cow/artifacts/manifest.md`
- Format log: `reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-fmt-check.log`
- Focused unit log: `reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-test-task65b-vamana.log`
- PG18 check log: `reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-check-pg18.log`

## Review Ask

Please review the copy-on-write snapshot change as an enabling optimization before Slice F tuning. This packet does not claim the final Task 65b timing, recall, or scaling gates; those still require corpus-scale measurement.
