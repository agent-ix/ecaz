# Task 91 Packet 005: PqFastScan Parity Follow-Up

## Summary

This packet addresses the approved Packet 002 reviewer follow-up asking for an
explicit PqFastScan parity check against the pre-retouch direct IVF path.

Code checkpoint:
`a3ea53d7160ffe32df34838b634b202b0a5c0ecc`

Changes:

- Added `common_quant_codec_pq_fastscan_batch_is_bit_exact_with_direct_path`.
- The test prepares and encodes through the model-bound `IvfQuantCodec`.
- It independently prepares and encodes the same query and sources through
  `IvfQuantizer::prepare_ip_query_with_pq_model` and
  `IvfQuantizer::encode_source_with_pq_model`.
- It asserts encoded dimensions, gamma bits, encoded code bytes, and final
  batch score `to_bits()` equality against direct scalar scoring.

No production behavior changed in this packet.

## Validation

- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - `24 passed; 0 failed`
  - artifact: `artifacts/cargo-test-ivf-quantizer.log`
- `git diff --check`
  - passed with no output
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

- Confirm the new PqFastScan test closes the Packet 002 parity clarification by
  comparing the `IvfQuantCodec` batch path to the pre-retouch direct IVF
  PqFastScan encode/prepare/score path.
