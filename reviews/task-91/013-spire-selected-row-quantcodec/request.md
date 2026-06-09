# Task 91 Review Request: SPIRE Selected-Row QuantCodec Path

## Summary

This checkpoint migrates the SPIRE V2 selected-row scoring path through `QuantCodec` when a bounded RaBitQ cutoff is not active.

Changes:

- Renames the selected/filter helper from the RaBitQ-only cutoff name to `append_quantized_v2_filtered_column_candidates`.
- Adds `score_v2_column_candidate_ip_with_quant_codec`, which scores a V2 row payload through `scorer.quant_codec()` and `QuantCodec::score_ip_candidate`.
- Uses that helper for selected row ranges when the accumulator is not using bounded RaBitQ pruning.
- Keeps bounded RaBitQ cutoff scoring behind a dedicated helper, preserving the existing recall-safe upper-bound early prune.
- Adds a focused test proving the selected-row QuantCodec helper is bit-exact with `SpirePreparedAssignmentScorer::score_payload_ip` for TurboQuant and RaBitQ payloads.

This reduces the Packet 006 audit gap. The remaining deliberate exception is the bounded RaBitQ cutoff optimization, which still uses `try_score_payload_ip` to avoid removing its upper-bound pruning behavior without a trait-level cutoff API.

## Validation

Packet-local validation summary:

- `artifacts/manifest.md`
- `artifacts/validation.md`

Commands passed:

- `cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::select_leaf_block_row_ranges --no-default-features --features pg18`
- `git diff --check`

## Review Focus

- Confirm selected-row V2 scoring now routes through `QuantCodec` for TurboQuant/RaBitQ when no bounded cutoff is active.
- Confirm keeping bounded RaBitQ cutoff on the specialized `try_score_payload_ip` helper is the right boundary until a `QuantCodec` cutoff API exists.
