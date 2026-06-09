# Task 91 Packet 015: SPIRE Sampled Fallback QuantCodec Cleanup

## Summary

This checkpoint closes two remaining SPIRE fallback scoring gaps found during
the Task 91 source audit after Packet 014:

- sampled leaf-block scoring now routes through
  `score_v2_column_candidate_ip_with_quant_codec`;
- the bounded RaBitQ helper's "no cutoff score is available yet" branch now
  routes through `QuantCodec` instead of calling the prepared scorer directly.

The actual bounded RaBitQ cutoff branch still calls
`try_score_payload_ip(..., min_ip_to_keep)`. That is the deliberate exception
already discussed in Packet 013 feedback: preserving early cutoff pruning still
needs a trait-level cutoff API before it can move behind `QuantCodec`.

## Code Changes

- `src/am/ec_spire/scan/candidates.rs`
  - routes sampled leaf-block row scoring through the existing single-candidate
    QuantCodec helper;
  - routes the bounded-helper `None` cutoff case through the same helper;
  - leaves only the `Some(min_ip_to_keep)` bounded RaBitQ cutoff path on the
    direct scorer.
- `src/am/ec_spire/scan/tests.rs`
  - exposes the bounded-helper path to scan tests.
- `src/am/ec_spire/scan/tests/candidates.rs`
  - adds `rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available`.

## Source Audit Result

After this slice, the remaining production `score_payload_ip` /
`try_score_payload_ip` matches outside SPIRE quantizer internals are:

```text
src/am/ec_spire/scan/candidates.rs:2765:
    scorer.try_score_payload_ip(column_format, gamma, encoded_payload, min_ip_to_keep)
```

That path is reached only when:

- the SPIRE assignment format is RaBitQ;
- the accumulator is bounded; and
- the accumulator already has a cutoff score.

All no-cutoff single-candidate, selected-row, sampled-row, and column-batch
fallbacks now route through `QuantCodec`.

## Validation

- `cargo fmt`
  - completed with the repo's usual stable-rustfmt warnings for nightly-only
    import grouping options.
- `cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available --no-default-features --features pg18`
  - `1 passed; 0 failed`
- `cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
  - `1 passed; 0 failed`
- `cargo test --lib am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
  - `1 passed; 0 failed`
- `git diff --check`
  - passed

## Review Focus

- Confirm sampled leaf-block scoring is correctly classified as a normal
  SPIRE scoring path and now routes through `QuantCodec`.
- Confirm the no-cutoff bounded-helper branch should use `QuantCodec`, while
  the real bounded RaBitQ cutoff branch remains a documented exception until a
  cutoff-capable trait API lands.
