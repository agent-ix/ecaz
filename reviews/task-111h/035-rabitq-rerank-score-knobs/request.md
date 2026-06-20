# Task 111h / 035 RaBitQ Rerank Score Knobs

Code commit under review:

- `26de9a0c6a72a92646fe275d3883889008a82e58` -
  `task111h: expose rabitq rerank score knobs`

## Summary

This slice addresses the new 024 feedback that the RaBitQ compressed-rerank
ceiling may be an estimator/clip artifact by adding index-path A/B levers:

- `rabitq_rerank_least_squares = 0|1`
  - `0` keeps the existing asymmetric estimator.
  - `1` uses the lower-variance least-squares dequantized projection already
    present in the RaBitQ sidecar harness.
- `rabitq_rerank_clip = 1..8`
  - Default `2` preserves current behavior.
  - Values `3` and `4` are now buildable for the requested A/B.

The knobs are limited to `storage_format = 'coarse_rerank'` with
`rerank_format = 'rabitq4'` or `rerank_format = 'rabitq8'`. Defaults preserve
existing indexes and benchmark cells.

## Implementation Notes

- The shared rerank payload codec carries RaBitQ score mode and clip.
- Build and insert encode persisted RaBitQ rerank payloads with the selected
  clip.
- Scan scores persisted index payloads with the selected score mode.
- The estimator path keeps the existing contiguous batch scorer.
- The least-squares path currently scores scalar over the same payload slab; no
  new LS batch kernel is claimed.
- The seeded RaBitQ quantizer cache key now includes clip, so same
  dimension/seed/bits indexes with different clips do not alias.

Important non-claim: this is not a true full-materialized dequantized dot
scorer. The existing tree exposed `estimate_ip_least_squares_scalar_only`; this
packet wires that known lower-variance scorer for the A/B. A full dequant-dot
materialization scorer would be a separate lever with separate decode counters.

## Validation

Artifacts are under `artifacts/`.

- `cargo-test-rabitq-rerank-options.log`:
  `2 passed; 0 failed; 2208 filtered out`.
- `cargo-test-rabitq-index-rerank-levers.log`:
  `1 passed; 0 failed; 2209 filtered out`.
- `cargo-check-pg18.log`:
  `Finished dev profile`.

## Remaining Work

This packet only makes the levers benchmarkable. The 024 reviewer ask still
requires an index-path A/B at 100k for RaBitQ-8 across least-squares and clip
values, driven by `ecaz bench suite`, before an abandon/promote decision.
