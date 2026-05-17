---
id: 30256
title: SPIRE Leaf V2 Segmented Store Path
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 260070c3
---

# Review Request: SPIRE Leaf V2 Segmented Store Path

## Summary

This checkpoint starts addressing the A1 architecture feedback by adding a
segmented, column-major V2 leaf-object path while leaving the existing V1
row-contiguous helper intact.

- Adds V2 partition-object header version handling for leaf metadata and segment
  tuples.
- Adds `LeafPartitionObjectV2` metadata and segment codecs.
- Stores V2 base leaves as one metadata tuple plus one or more page-sized row
  segment tuples in `SpireLocalObjectStore`.
- Encodes V2 row columns as flags, fixed-stride local `vec_id`s, heap TIDs,
  gammas, and payload bytes.
- Keeps row-encoded deltas and current build/scan V1 helper behavior unchanged.
- Updates Task 30 status to show that V2 codecs and store write/read exist, but
  build/scan migration remains open.

## Non-Goals

- Does not switch build drafts to emit V2 leaves yet.
- Does not switch scan collection to read V2 borrowed column views yet.
- Does not add relation-backed persistence.
- Does not address A2/A3/A4/A5/A7/A9 beyond the storage-format prerequisite.

## Review Focus

- Whether the V2 metadata tuple plus segment-chain representation is the right
  concrete shape for large PID-addressed base leaves.
- Whether placement `object_bytes` should continue to represent total logical
  object bytes while the placement TID points at the V2 metadata tuple.
- Whether the Phase 1 restriction to fixed-stride local `vec_id`s inside V2 base
  leaves is acceptable, with global-ID rewrite deferred to future epochs.
- Whether the tests cover the important shape constraints: multi-segment leaf,
  empty leaf, mixed payload rejection, and global-ID rejection.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 167 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
