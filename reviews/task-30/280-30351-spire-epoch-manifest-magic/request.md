# SPIRE Epoch Manifest Magic

## Checkpoint

- Code commit: `a09bc0ca`
  (`Add SPIRE epoch manifest magic`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for epoch manifest tuple discovery

## Summary

This checkpoint adds an explicit magic prefix to the SPIRE epoch-manifest
codec.

Before this change, `index_epoch_snapshot` found epoch manifests by scanning
relation object tuples and accepting tuples with the epoch-manifest encoded
length that also decoded successfully. That was correct for the current tuple
set, but it relied on no future tuple type sharing the same length and
compatible format prefix.

Epoch manifests now encode with `EPOCH_MANIFEST_MAGIC` before the shared meta
format version. `SpireEpochManifest::decode` validates the magic before reading
state, consistency mode, and epoch fields, making diagnostic tuple discovery
structural rather than length-only.

The Task 30 plan now records this hardening under the publish-coordinator and
manifest-write foundation.

## Changed Files

- `src/am/ec_spire/meta.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test epoch_manifest --no-default-features --features pg18`
  - `5 passed; 0 failed; 1121 filtered out`
- `cargo test --lib test_ec_spire_epoch_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 1125 filtered out`
- `git diff --check`

## Notes

- This responds to the round-review recommendation to make epoch-manifest
  discovery robust against future tuple-shape collisions.
