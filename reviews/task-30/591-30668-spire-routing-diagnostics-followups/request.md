# Review Request: SPIRE Routing Diagnostics Follow-Ups

This small follow-up processes non-blocking reviewer feedback from packets
30664 and 30666 before starting Phase 10 implementation.

Code checkpoint: `7a6004a0` (`Track SPIRE routing diagnostics review followups`)

## Scope

- Adds an explicit comment that `append_scored_candidate` returns the suppressed
  duplicate candidate on every vec-id collision, regardless of whether the
  incoming or incumbent candidate wins.
- Clarifies in `docs/SPIRE_DIAGNOSTICS.md` that scan placement
  `candidate_row_count` is pre-dedupe and equals the primary plus
  boundary-replica candidate role split.
- Tracks 30664 F1 in Phase 10 as either a shared recursive-routing traversal
  helper or a property test preventing diagnostic/production route drift.
- Tracks 30664 F2 in Phase 10 as a unified local scan pipeline snapshot that
  mirrors the remote `ec_spire_remote_pipeline_steps` operator shape.

## Validation

- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm this resolves 30666 F1/F2 without changing behavior.
- Confirm the 30664 F1/F2 items are now tracked in the right Phase 10 area.
