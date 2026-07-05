# Review Request: SPIRE Recursive Draft Invariants

Head SHA: `de90b588`

## Summary

Recursive build validation now documents the three validation barriers for
recursive drafts:

- in-memory routing-object and centroid-record shape checks;
- post-write epoch leaf-placement validation;
- snapshot-time hierarchy validation before scan descent.

The in-memory draft validator now also enforces dense centroid ordinals per
parent PID. This protects the routing tie-break contract from silently changing
if future code filters or reorders child records without rebuilding ordinals.

The first-level `source_count = 1` assignment now has an inline comment making
clear that it counts one trained leaf centroid source, not rows assigned to the
eventual leaf object.

## Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo test sparse_centroid_ordinals -- --nocapture`
  - 1 passed:
    `recursive_routing_build_validation_rejects_sparse_centroid_ordinals`.
- `cargo fmt`
- `git diff --check`

## Review Focus

- Confirm the validation-barrier comment describes the intended boundaries.
- Confirm dense ordinal validation belongs in the recursive draft validator.
- Confirm the `source_count` comment resolves the centroid-vs-row-count
  ambiguity.
