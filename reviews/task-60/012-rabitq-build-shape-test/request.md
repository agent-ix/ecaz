# Review Request: RaBitQ Build Shape Test

## Scope

This checkpoint adds a focused pure-Rust unit test for DiskANN RaBitQ build
metadata shape.

The test pins the 1536-D RaBitQ build parameters to:

- zero binary sidecar words
- 204-byte direct RaBitQ search code
- no grouped-PQ or binary-sidecar payload flags

This is test-only coverage for the Task 60 on-disk discriminator and tuple
layout contract.

## Validation

Artifacts are under `reviews/task-60/012-rabitq-build-shape-test/artifacts/`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.

## Remaining Task 60 Gate

The external benchmark host still needs to run the full 100k/1M Task 60 suite
and record recall, latency, storage, and the 1M shipping decision.
