# Task 148 Packet 004: Calibration Core

## Summary

This checkpoint adds calibration-only TurboQuant 4-bit no-QJL math in `src/quant/prod.rs`.

It introduces:

- `TqCalibration { shift, scale }`
- calibrated encode/query-prep/score helpers for the no-QJL 4-bit lane
- per-coordinate calibration fit from training vectors using rotated-domain percentiles against the canonical Beta marginal
- a focused release-lib test proving the calibrated scorer reduces score error on a deterministic rotated-domain anisotropic fixture

This packet intentionally does not wire calibration into EC-IVF metadata, posting storage, rerank sidecars, or benchmark suites. The Slice 3 A/B benchmark remains open and must be measured separately from the reverted Slice 2 renorm path.

## Validation

Packet-local log: `artifacts/cargo-test-calibration-core.log`

```text
cargo test --release --lib calibration_no_qjl_4bit_reduces_anisotropic_score_error
```

Result: passed.

## Review Notes

- The scorer keeps the correction in query-prep plus per-candidate epilogue form: encode quantizes `(x_rot + shift) * scale`; query prep applies `q_rot / scale` and a scalar bias of `-sum(q_rot * shift)`.
- No QJL path is enabled.
- No default behavior changes.
- No on-disk format claim is made in this checkpoint.

