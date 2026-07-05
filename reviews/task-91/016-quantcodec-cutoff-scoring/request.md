# Task 91 Review Request: QuantCodec Cutoff Scoring

## Summary

This checkpoint removes the remaining SPIRE scan-side direct bounded RaBitQ scorer call by adding a cutoff-capable QuantCodec method:

- added `QuantCodec::try_score_ip_candidate(...)`, with a default implementation that scores normally and returns `Some(score)`
- implemented SPIRE cutoff scoring on `SpireAssignmentQuantCodec`, so the RaBitQ cutoff helper calls through the codec boundary instead of directly calling the prepared scorer from scan code
- implemented the same trait hook for IVF through `IvfQuantCodec`, reusing its existing min-bound helper for RaBitQ and grouped-PQ
- added regression coverage for the active SPIRE cutoff path and the IVF grouped-PQ trait cutoff path

The direct `score_payload_ip` / `try_score_payload_ip` calls that remain are inside the SPIRE quantizer/codec implementation and tests. `src/am/ec_spire/scan/candidates.rs` has no direct scorer helper references.

## Code Under Review

- Code commit: `3fc5a3aee0e0cfb59d094162e1bd3d7757c7f345`
- Files:
  - `src/am/common/quant_codec.rs`
  - `src/am/ec_ivf/quantizer.rs`
  - `src/am/ec_spire/quantizer/mod.rs`
  - `src/am/ec_spire/scan/candidates.rs`
  - `src/am/ec_spire/scan/tests/candidates.rs`

## Validation

Artifacts are under `reviews/task-91/016-quantcodec-cutoff-scoring/artifacts/`.

- `cargo fmt`
  - Result: passed, with existing stable-rustfmt warnings about nightly-only import grouping settings
  - Log: `artifacts/cargo-fmt.log`
- `cargo test --lib am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_cutoff_prunes_through_trait --no-default-features --features pg18`
  - Result: 1 passed
  - Log: `artifacts/ivf-grouped-pq-cutoff-test.log`
- `cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_routes_active_cutoff_through_quant_codec --no-default-features --features pg18`
  - Result: 1 passed
  - Log: `artifacts/spire-active-cutoff-test.log`
- `cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available --no-default-features --features pg18`
  - Result: 1 passed
  - Log: `artifacts/spire-fallback-cutoff-test.log`
- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - Result: 25 passed
  - Log: `artifacts/ivf-quantizer-tests.log`
- `rg -n 'score_payload_ip\(|try_score_payload_ip\(|score_batch_ip\(' src/am/ec_spire/scan/candidates.rs`
  - Result: no matches
  - Log: `artifacts/spire-scan-direct-scorer-audit.log`
- `rg -n 'try_score_ip_candidate' src/am src/quant`
  - Result: trait method, SPIRE/IVF implementations, tests, and SPIRE scan callsite found
  - Log: `artifacts/quantcodec-cutoff-api-audit.log`
- `git diff --check`
  - Result: passed
  - Log: `artifacts/git-diff-check.log`

## Review Focus

- Confirm that `try_score_ip_candidate` is the right shared API shape for bounded candidate pruning before broader all-quant/all-index rollout work builds on it.
- Confirm that SPIRE scan code now respects the QuantCodec boundary for the bounded RaBitQ cutoff helper.
- Confirm that the IVF override is appropriate shared coverage for existing RaBitQ/grouped-PQ min-bound behavior.
