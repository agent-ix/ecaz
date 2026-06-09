# Task 91 Packet 006: SPIRE Closeout Audit

## Summary

This packet closes the approved Packet 003 and Packet 004 SPIRE follow-ups.

Code checkpoint:
`c344f168dc119855f0efd7c607a990d5b1c647bc`

Changes:

- Added a test proving `QuantCodec::score_ip_batch` preserves
  `SpirePreparedAssignmentScorer::score_candidate_batch_ip`'s existing output
  length mismatch error.
- Added a test proving `scorer.quant_codec()` matches the scorer's implicit
  supported format state for TurboQuant and RaBitQ:
  - `payload_format`
  - `dimensions`
  - `codec_kind`
  - `search_codec_tag`
  - `payload_len`
- Confirmed by code audit that `SpirePreparedAssignmentScorer` stores
  `payload_format` as enum variant state and `dimensions` as immutable enum
  fields. There is no mutation point between `scorer.quant_codec()` creation
  and the immediate `QuantCodec::score_ip_batch` call.

## Scan Path Audit

Artifact: `artifacts/spire-scan-path-audit.log`

- The unselected, unbounded V2 leaf-column path uses
  `QuantCodec::score_ip_batch(&codec, scorer, &batch, &mut scores)?` at
  `src/am/ec_spire/scan/candidates.rs:2426`.
- If `selected_row_ranges.is_some()` or the scorer is RaBitQ with a bounded
  accumulator, the path remains routed through
  `append_quantized_v2_column_candidates_with_rabitq_cutoff`.
- That cutoff path still scores row-by-row through `scorer.try_score_payload_ip`
  at `src/am/ec_spire/scan/candidates.rs:2717`.
- The per-column non-selected helper also still has its older
  `scorer.score_batch_ip(...)` call at `src/am/ec_spire/scan/candidates.rs:2631`,
  but the V2 leaf-column fast path now batches across column segments before
  reaching it.

This matches the current Task 91 Phase 3 boundary: the production V2
leaf-column batch path is on `QuantCodec`, while selected-block and RaBitQ
cutoff paths remain on the pre-Task87 inline shape for later migration.

## Validation

- `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
  - `18 passed; 0 failed`
  - artifact: `artifacts/cargo-test-spire-quantizer.log`
- `git diff --check`
  - passed with no output
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

- Confirm the two explicit SPIRE equivalence checks close Packet 003 and Packet
  004 feedback.
- Confirm the selected-block / RaBitQ cutoff audit is accurate and scoped for
  later Task 91 migration work.
