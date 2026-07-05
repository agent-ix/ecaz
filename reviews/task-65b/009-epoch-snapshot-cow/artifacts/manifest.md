# Task 65b Packet 009 Artifact Manifest

- head SHA: `f64c0a36eb51a37e7631711d149ef7c7b7f5669b`
- task bucket: `reviews/task-65b/009-epoch-snapshot-cow`
- timestamp: `2026-06-05T03:09:44Z`
- lane: local PG18 Rust validation
- scope: code checkpoint for copy-on-write Vamana epoch snapshots
- index/table isolation: not applicable; no corpus or PostgreSQL index build was run for this checkpoint

## Code Under Review

- `src/am/ec_diskann/vamana.rs`

The code changes `BuilderNeighborCache` from `Vec<Vec<u32>>` rows to copy-on-write `Arc<[u32]>` rows and adds `BuilderNeighborSnapshot`, an immutable `Arc<[Arc<[u32]>]>` view for proposal workers. Reducer writes replace touched live rows, so snapshots keep the row shape seen at epoch start without cloning every adjacency list.

This preserves the Slice E concurrency model:

- proposal workers read immutable epoch snapshots;
- the reducer remains the only writer to `BuilderNeighborCache`;
- proposal completion order is still normalized by ordinal before commit;
- `batch_size = 1` remains byte-shape equivalent to the serial build in unit coverage.

## Validation

- `cargo fmt --check > reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-fmt-check.log 2>&1`
  - exited 0
  - log contains only pre-existing stable-rust warnings about unstable rustfmt options
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_ > reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-test-task65b-vamana.log 2>&1`
  - exited 0
  - `4 passed; 0 failed; 1968 filtered out`
  - includes new `task65b_snapshot_rows_do_not_observe_later_reducer_writes`
- `cargo check -p ecaz --lib --no-default-features --features pg18 > reviews/task-65b/009-epoch-snapshot-cow/artifacts/cargo-check-pg18.log 2>&1`
  - exited 0
  - `Finished dev profile`

## Artifact Summary

- `cargo-fmt-check.log`: formatting gate.
- `cargo-test-task65b-vamana.log`: focused Task 65b Vamana model tests.
- `cargo-check-pg18.log`: PG18 lib compile check.

## Notes

This packet does not claim Task 65b performance gates. It removes a known tuning artifact before Slice F: full adjacency cloning per epoch. Corpus timing, recall, and worker/batch scaling still need to be measured in the Slice F/H packets.
