# Task 91 Packet 003: SPIRE QuantCodec Follow-Through

## Summary

This packet asks for review of a narrow Task 91 Phase 3 SPIRE follow-through
slice. SPIRE already had a `SpireAssignmentQuantCodec` implementation, but its
`QuantCodec::score_ip_batch` path bypassed the prepared scorer's existing
candidate-batch scorer. This slice routes the common trait batch method through
the same SPIRE scorer path used by direct scan code.

Code checkpoint under review:

- `6f45298daf6d7ae67ea939b1a37ff303e4ded88c` - `Route SPIRE QuantCodec batch scoring through scorer`

## Changes

- Changed `impl QuantCodec for SpireAssignmentQuantCodec` so
  `score_ip_batch` delegates to
  `SpirePreparedAssignmentScorer::score_candidate_batch_ip`.
- Added a focused unit test proving the `QuantCodec` batch path is bit-exact
  with the prepared scorer batch path.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `artifacts/git-diff-check.log`
  - `git diff --check`
  - result: passed with no output
- `artifacts/cargo-test-spire-quantizer.log`
  - `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
  - result: `16 passed; 0 failed`

## Review Focus

- Confirm the common `QuantCodec::score_ip_batch` path should reuse
  `score_candidate_batch_ip` rather than maintaining its own per-candidate loop.
- Confirm this preserves existing SPIRE batch behavior while removing duplicate
  quant scoring knowledge from the trait impl.
