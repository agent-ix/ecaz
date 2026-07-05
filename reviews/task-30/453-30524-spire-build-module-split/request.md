# Review Request: SPIRE Build Module Split

Head SHA: `f3bb3f5f`

## Summary

This is a split-only follow-up to the SPIRE storage split. It keeps the same
include-based namespace shape and moves the oversized build module into smaller
files.

The old large file is now a thin include facade:

- `src/am/ec_spire/build.rs`

The moved bodies now live under:

- `src/am/ec_spire/build/{types,object_store,publish,routing_plan,recursive,training,drafts,tuples,tests}.rs`
- `src/am/ec_spire/build/tests/{centroid_state,recursive,publish,single_level}.rs`

After this checkpoint, the largest SPIRE build split file is under 850 lines.
The largest files are:

- `src/am/ec_spire/build/recursive.rs` at 842 lines
- `src/am/ec_spire/build/tests/recursive.rs` at 673 lines
- `src/am/ec_spire/build/publish.rs` at 508 lines
- `src/am/ec_spire/build/drafts.rs` at 413 lines

No function bodies, visibility, APIs, tests, or call paths were intentionally
changed.

## Mechanical Proof

Reconstructed `src/am/ec_spire/build.rs` by concatenating the facade header, the
new chunks, and the nested test chunks. Compared it against the pre-split file
with blank-line-only differences ignored:

- `src/am/ec_spire/build.rs`: matched

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18`

Tests were not re-run for this individual split checkpoint. The immediately
preceding storage checkpoint passed the full serial PG18 suite, and this build
checkpoint is a mechanical file split with compile validation of include
boundaries.

Clippy remains not clean on the branch due existing warnings-as-errors recorded
in the storage split packet.

## Review Focus

- Confirm this is only a mechanical split.
- Confirm the chunk names are useful enough for Phase 4 review.
- Flag any build chunk that should be renamed before logic lands on top.
