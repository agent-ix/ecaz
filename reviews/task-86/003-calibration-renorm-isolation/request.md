# Review Request: TQ+ Calibration Versus Renorm

## Summary

This checkpoint extends the Task 86 TQ+ prototype probe to isolate the source of the quality gain:

- Baseline: current TQ no-QJL 4-bit scoring.
- TQ+ unrenormalized: calibrated vector encoding plus calibrated query LUT/bias, using only the packed MSE bytes.
- TQ+ renormalized: the same calibrated scorer multiplied by the prototype per-vector `renorm` scalar.

The code remains `cfg(any(test, feature = "bench"))` only.

## Evidence

Artifact manifest: `reviews/task-86/003-calibration-renorm-isolation/artifacts/manifest.md`

Focused validation:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::tqplus_no_qjl_4bit_probe_reports_score_error_delta -- --nocapture
```

Result:

```text
task86_tqplus_probe dim=1536 bits=4 train=192 queries=16 candidates=96 baseline_mae=0.02642813 tqplus_unrenorm_mae=0.00311462 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_unrenorm_rmse=0.00400303 tqplus_rmse=0.00453193 mae_delta_pct=-86.97 rmse_delta_pct=-85.18 renorm_mean=1.00829677 renorm_min=0.98944199 renorm_max=1.03691769
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1976 filtered out; finished in 0.44s
```

## Interpretation

On this normalized/IP probe, the extra per-vector `renorm` scalar is not carrying the improvement. Calibration-only beats both baseline and calibration-plus-renorm:

- Baseline MAE/RMSE: `0.02642813` / `0.03058423`
- TQ+ unrenormalized MAE/RMSE: `0.00311462` / `0.00400303`
- TQ+ renormalized MAE/RMSE: `0.00344425` / `0.00453193`

That makes the lower-storage option worth prioritizing first: carry index-level calibration metadata and query LUT/bias changes, while preserving the existing per-vector packed-code byte count.

## Review Focus

- Whether this is a fair isolation of calibration-only versus calibration-plus-renorm for the normalized/IP lane.
- Whether the next implementation slice should drop `renorm` from the index-level prototype, or keep it as a separate optional experiment for non-normalized/vector-norm-sensitive lanes.
