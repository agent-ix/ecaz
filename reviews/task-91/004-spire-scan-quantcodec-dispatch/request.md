---
task: 91
packet: 004-spire-scan-quantcodec-dispatch
agent: coder
date: 2026-06-09
---

# Task 91 Phase 3: SPIRE Scan QuantCodec Dispatch

## Summary

This checkpoint moves the SPIRE V2 leaf-column candidate scan batch path onto
the common `QuantCodec::score_ip_batch` entry point.

Code commit:

- `1504e5815904fb17729721d741445aa02cab7fa8`
  `Route SPIRE scan batches through QuantCodec`

Changes:

- Adds `SpirePreparedAssignmentScorer::quant_codec()` as a local adapter helper.
- Replaces the direct `scorer.score_candidate_batch_ip(...)` call in
  `append_quantized_v2_leaf_column_candidates` with
  `QuantCodec::score_ip_batch(&codec, scorer, &batch, &mut scores)`.
- Keeps scoring math and metadata unchanged; the production scan callsite now
  routes through the Task 91 common batch dispatch surface.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `git diff --check`: passed with no output.
- `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`:
  `16 passed; 0 failed`.
- `cargo test --lib am::ec_spire::scan::tests --no-default-features --features pg18`:
  `99 passed; 0 failed`.

I also ran the broader
`cargo test --lib am::ec_spire --no-default-features --features pg18` target.
It compiled the touched code and then failed one unrelated remote-executor test:
`production_receive_adapters_reject_selected_pid_batches_before_connection`
expected `remote_payload_too_large` but observed `connect_failed`.

## Review Focus

- Confirm the production SPIRE V2 leaf-column scan path should dispatch through
  `QuantCodec::score_ip_batch` using the prepared scorer as the trait prepared
  query.
- Confirm `SpirePreparedAssignmentScorer::quant_codec()` is the right local
  adapter helper shape for this migration slice.
