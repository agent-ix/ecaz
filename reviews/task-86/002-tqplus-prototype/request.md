# Review Request: TQ+ Calibration Prototype

## Summary

This checkpoint turns the TurboVec source finding into a narrow, test/bench-only prototype inside our `ProdQuantizer`.

The prototype keeps database vectors packed as 4-bit no-QJL TQ codes, but changes the pre-quantization space:

- Fit per-coordinate TQ+ calibration from normalized, SRHT-rotated training vectors.
- Encode normalized vectors in that calibrated space.
- Store the same packed 4-bit MSE code bytes as baseline plus a prototype per-vector `renorm` scalar.
- Prepare the query in the same calibrated scoring space as a per-query LUT plus scalar bias.
- Score candidates by packed-code nibble lookup plus `renorm`; no vector decompression is added.

This is intentionally gated behind `cfg(any(test, feature = "bench"))`; it does not change production storage, index behavior, SQL behavior, or durable formats.

## Evidence

Artifact manifest: `reviews/task-86/002-tqplus-prototype/artifacts/manifest.md`

Focused validation:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::tqplus_no_qjl_4bit_probe_reports_score_error_delta -- --nocapture
```

Result:

```text
task86_tqplus_probe dim=1536 bits=4 train=192 queries=16 candidates=96 baseline_mae=0.02642813 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_rmse=0.00453193 mae_delta_pct=-86.97 rmse_delta_pct=-85.18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1976 filtered out; finished in 0.40s
```

## Interpretation

This supports Task 86 Track B as a real candidate: on a deterministic anisotropic 1536-dimensional probe, TQ+ calibration plus renorm materially reduces approximate inner-product error versus our current TQ no-QJL 4-bit baseline while preserving the same packed code byte count.

It is not yet production evidence. The next decision point is whether the same gain survives an `ecaz bench suite` corpus/index lane with recall, latency, and storage measured across at least one of our actual TQ-backed indexes.

## Review Focus

- Whether the calibration and inverse-query scoring algebra matches the intended TQ+ transform.
- Whether the prototype is sufficiently isolated from production code.
- Whether `renorm` is the right first storage-side scalar to carry forward, or whether the next slice should isolate calibration-only versus calibration-plus-renorm before index-level benchmarking.
