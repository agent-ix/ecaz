# Task 86 Packet 003 Artifact Manifest

- Head SHA: `84c4ff340e1c6c85f4ed8f0d9da98b55eb20cdaf`
- Task bucket: `reviews/task-86/003-calibration-renorm-isolation`
- Timestamp: `2026-06-07T06:07:35Z`
- Lane: focused unit/prototype probe, not an accepted benchmark lane
- Fixture: deterministic synthetic anisotropic 1536-dimensional inner-product corpus
- Storage format: in-memory TQ+ prototype; 4-bit no-QJL packed MSE bytes, with and without prototype per-vector `renorm`
- Rerank mode: none
- Index surface: quantizer-only probe; no HNSW, DiskANN, IVF, or SPIRE index build
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `tqplus-renorm-isolation.log`

Command:

```sh
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::tqplus_no_qjl_4bit_probe_reports_score_error_delta -- --nocapture > reviews/task-86/003-calibration-renorm-isolation/artifacts/tqplus-renorm-isolation.log 2>&1
```

Key result:

```text
task86_tqplus_probe dim=1536 bits=4 train=192 queries=16 candidates=96 baseline_mae=0.02642813 tqplus_unrenorm_mae=0.00311462 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_unrenorm_rmse=0.00400303 tqplus_rmse=0.00453193 mae_delta_pct=-86.97 rmse_delta_pct=-85.18 renorm_mean=1.00829677 renorm_min=0.98944199 renorm_max=1.03691769
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1976 filtered out; finished in 0.44s
```
