# Review Request: SPIRE Test and Publish Chunk Split

Head SHA: `6a79ea4d`

## Summary

This is a split-only follow-up to the SPIRE update/scan module split. It keeps
the same include-based namespace shape and moves the remaining oversized chunks
into smaller files.

Files split:

- `src/am/ec_spire/update/tests.rs`
- `src/am/ec_spire/scan/tests.rs`
- `src/am/ec_spire/update/publish.rs`

After this checkpoint, the largest SPIRE update/scan split file is under 1,000
lines. The largest files are:

- `src/am/ec_spire/update/tests/merge_execution.rs` at 961 lines
- `src/am/ec_spire/update/routing.rs` at 959 lines
- `src/am/ec_spire/update/tests/replacement_epoch.rs` at 939 lines
- `src/am/ec_spire/scan/tests/candidates.rs` at 902 lines

No function bodies, visibility, APIs, tests, or call paths were intentionally
changed.

## Mechanical Proof

Reconstructed the moved files by concatenating the new chunks and compared them
against `HEAD^` with blank-line-only differences ignored:

- `src/am/ec_spire/update/tests.rs`: matched
- `src/am/ec_spire/scan/tests.rs`: matched
- `src/am/ec_spire/update/publish.rs`: matched

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18`

Tests were not run. This checkpoint only changes file boundaries, and the PG18
compile check validates the include boundaries.

## Review Focus

- Confirm the split is mechanical.
- Confirm the chunk names are clear enough for Phase 4 follow-on review.
- Flag any file that should be renamed before logic lands on top.
