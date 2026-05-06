# Review Request: SPIRE Root Module Split

Head SHA: `fe99cb07`

## Summary

This is a split-only follow-up to the SPIRE build split. It keeps the same
include-based namespace shape and moves the oversized SPIRE root module into
smaller files under `src/am/ec_spire/root/`.

The old large file is now a thin include facade:

- `src/am/ec_spire/mod.rs`

The moved bodies now live under:

- `src/am/ec_spire/root/{lifecycle,types,diagnostics,hierarchy_shape,snapshots,maintenance,hierarchy_snapshots,debug,tests}.rs`

After this checkpoint, the largest SPIRE root split file is under 900 lines.
The largest files are:

- `src/am/ec_spire/root/tests.rs` at 851 lines
- `src/am/ec_spire/root/snapshots.rs` at 638 lines
- `src/am/ec_spire/root/hierarchy_snapshots.rs` at 530 lines
- `src/am/ec_spire/root/maintenance.rs` at 467 lines

No function bodies, visibility, APIs, tests, or call paths were intentionally
changed.

## Mechanical Proof

Reconstructed `src/am/ec_spire/mod.rs` by concatenating the facade header and
the new root chunks. Compared it against the pre-split file with
blank-line-only differences ignored:

- `src/am/ec_spire/mod.rs`: matched

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
- Confirm the root chunk names make the SPIRE SQL/diagnostic surface easier to
  review.
- Flag any root chunk that should be renamed before logic lands on top.
