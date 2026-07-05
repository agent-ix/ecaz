# Task 91 Review Request: SPIRE Column Batch QuantCodec Path

## Summary

This checkpoint closes the remaining Packet 013 reviewer note for the SPIRE V2 per-column non-selected helper.

Changes:

- Replaces the direct `scorer.score_batch_ip(...)` call in `append_quantized_v2_column_candidates` with `QuantCodec::score_ip_batch`.
- Adds `score_v2_column_payloads_ip_with_quant_codec`, which validates the V2 payload shape, builds a `CandidateBatch`, and dispatches through `scorer.quant_codec()`.
- Adds a focused bit-exact parity test comparing the new helper with `SpirePreparedAssignmentScorer::score_batch_ip` for TurboQuant and RaBitQ.

The bounded RaBitQ cutoff path remains the deliberate exception because it preserves the recall-safe upper-bound prune from `try_score_payload_ip`.

## Validation

Packet-local validation summary:

- `artifacts/manifest.md`
- `artifacts/validation.md`

Commands passed:

- `cargo test --lib am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `git diff --check`

## Review Focus

- Confirm the V2 per-column non-selected helper now routes through `QuantCodec::score_ip_batch`.
- Confirm bounded RaBitQ cutoff remains the only SPIRE scan scoring exception pending a trait-level cutoff API.
