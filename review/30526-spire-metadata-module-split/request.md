# Review Request: SPIRE Metadata Module Split

Head SHA: `0072773d`

## Summary

This is a split-only follow-up to the SPIRE root split. It keeps the same
include-based namespace shape and moves the oversized metadata module into
smaller files under `src/am/ec_spire/meta/`.

The old large file is now a thin include facade:

- `src/am/ec_spire/meta.rs`

The moved bodies now live under:

- `src/am/ec_spire/meta/{root_control,states,local_store,placement,placement_directory,epoch,object_manifest,snapshot,tests}.rs`

After this checkpoint, the largest SPIRE metadata split file is under 900
lines. The largest files are:

- `src/am/ec_spire/meta/tests.rs` at 884 lines
- `src/am/ec_spire/meta/placement.rs` at 287 lines
- `src/am/ec_spire/meta/local_store.rs` at 247 lines
- `src/am/ec_spire/meta/object_manifest.rs` at 186 lines

No function bodies, visibility, APIs, tests, or call paths were intentionally
changed.

## Mechanical Proof

Reconstructed `src/am/ec_spire/meta.rs` by concatenating the facade header and
the new metadata chunks. Compared it against the pre-split file with
blank-line-only differences ignored:

- `src/am/ec_spire/meta.rs`: matched

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18`

Tests were not re-run for this individual split checkpoint. This checkpoint is
a mechanical file split with compile validation of include boundaries.

Clippy remains not clean on the branch due existing warnings-as-errors recorded
in the storage split packet.

## Review Focus

- Confirm this is only a mechanical split.
- Confirm the metadata chunk names make the codec and snapshot boundaries
  easier to review.
- Flag any metadata chunk that should be renamed before logic lands on top.
