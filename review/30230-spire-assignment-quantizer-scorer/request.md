---
id: 30230
title: SPIRE Assignment Quantizer Scorer
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 257bacb0
---

# Review Request: SPIRE Assignment Quantizer Scorer

## Summary

This checkpoint binds SPIRE scored assignment payload rows to the existing
quantizer implementations without wiring persistence or AM scan execution yet.

- Adds `src/am/ec_spire/quantizer.rs` as the SPIRE assignment payload scoring
  seam.
- Reuses `ProdQuantizer` for `TURBOQUANT` assignment payload encoding and
  prepared inner-product scoring.
- Reuses `RaBitQQuantizer` for `RABITQ` assignment payload encoding and
  prepared inner-product estimation.
- Rejects `NONE` as unscoreable.
- Keeps `PQ_FASTSCAN` explicitly deferred until SPIRE has persisted grouped-PQ
  model metadata.
- Validates query/source dimensions, finite vector values, scoreable payload
  formats, payload format/scorer matches, payload length, and RaBitQ zero-gamma
  rows.

## Non-Goals

- No relation-backed partition-object persistence.
- No AM scan execution wiring.
- No grouped-PQ/PQ-FastScan model persistence or scorer binding.
- No replica or remote-store behavior.

## Review Focus

- Whether the SPIRE-specific `SpireAssignmentPayloadFormat` enum should stay
  private to `ec_spire::quantizer` until scan/build wiring needs it, or move
  closer to `storage.rs` with the numeric payload tags.
- Whether `TURBOQUANT` payload validation should rely on the existing
  `payload_len(dimensions, bits) - sizeof(gamma)` derivation, or introduce a
  SPIRE-local helper to make the row body length more obvious.
- Whether `PQ_FASTSCAN` should continue returning explicit deferral errors here
  or stay out of the scorer enum until grouped-PQ metadata lands.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 143 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt --check` emitted the existing stable-rustfmt warnings for unstable
`imports_granularity` and `group_imports`.
