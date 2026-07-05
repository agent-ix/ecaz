# Review Request: SPIRE Update and Scan Module Split

Head SHA: `5662864c`

## Summary

This is a behavior-preserving file split before continuing SPIRE Phase 4 write
routing work.

The old large files are now thin include facades:

- `src/am/ec_spire/update.rs`
- `src/am/ec_spire/scan.rs`

The moved bodies now live under:

- `src/am/ec_spire/update/{types,scheduler,materialization,routing,leaf_rows,publish,helpers,delta,tests}.rs`
- `src/am/ec_spire/scan/{types,snapshot,candidates,routing,leaf_rows,relation,callbacks,tests}.rs`

No function bodies, visibility, APIs, or call paths were intentionally changed.
The split uses `include!` so the Rust module namespace remains the same and the
diff stays mechanical.

## Mechanical Proof

Reconstructed files by concatenating the old import header plus the new chunks
and compared them against `HEAD^` with blank-line-only differences ignored:

- `src/am/ec_spire/update.rs`: matched
- `src/am/ec_spire/scan.rs`: matched

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18`

Tests were not run. This checkpoint is a mechanical file split, and the PG18
compile target covers the include boundaries without invoking PostgreSQL test
execution.

## Review Focus

- Confirm this is only a mechanical split.
- Confirm the chunk names are useful enough for Phase 4 review.
- Flag any chunk that should be renamed before more hash-routed write work
  lands on top.
