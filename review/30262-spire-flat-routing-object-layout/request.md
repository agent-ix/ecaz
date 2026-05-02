# Review Request: SPIRE Flat Routing Object Layout

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `2bf14207 Flatten SPIRE routing object layout`

## Scope

This packet covers the A4 pre-persistence architecture feedback slice: replace
the in-memory routing object layout that owned one `Vec<f32>` per child with a
flat array layout before routing objects become relation-backed.

Changed files:

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/diagnostics.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- `SpireRoutingPartitionObject` now stores:
  - `centroid_ordinals: Vec<u32>`
  - `child_pids: Vec<u64>`
  - `centroids: Vec<f32>` as one `child_count * dimensions` block
- Constructors still accept `Vec<SpireRoutingChildEntry>` so build code remains
  source-compatible while the object stores the flat form internally.
- Added borrowed `SpireRoutingChildView` iteration for scan routing and object
  encoding.
- Updated diagnostics and scan routing to use `child_count()` and borrowed
  child views.
- Kept the encoded routing object byte shape stable: child index, child PID,
  then centroid components per child.
- Updated storage tests to assert the flat representation and round-trip shape.
- Marked the flat routing layout gate item complete in the Task 30 plan and
  architecture feedback response.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `173 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This checkpoint does not add SIMD centroid scoring. It establishes the storage
and iteration shape needed for chunked/SIMD-friendly scoring later by removing
per-child centroid allocations from the durable routing object model.
