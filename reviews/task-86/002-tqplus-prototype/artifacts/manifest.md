# Task 86 Packet 002 Artifact Manifest

- Head SHA: `24512038e739283fb07cc454b83baa49ada8a52c`
- Task bucket: `reviews/task-86/002-tqplus-prototype`
- Timestamp: `2026-06-07T06:03:50Z`
- Lane: focused unit/prototype probe, not an accepted benchmark lane
- Fixture: deterministic synthetic anisotropic 1536-dimensional inner-product corpus
- Storage format: in-memory TQ prototype; 4-bit no-QJL packed MSE bytes, plus prototype per-vector `renorm`
- Rerank mode: none
- Index surface: quantizer-only probe; no HNSW, DiskANN, IVF, or SPIRE index build
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `tqplus-probe.log`

Command:

```sh
cargo test -p ecaz --lib --no-default-features --features pg18 quant::prod::tests::tqplus_no_qjl_4bit_probe_reports_score_error_delta -- --nocapture > reviews/task-86/002-tqplus-prototype/artifacts/tqplus-probe.log 2>&1
```

Key result:

```text
task86_tqplus_probe dim=1536 bits=4 train=192 queries=16 candidates=96 baseline_mae=0.02642813 tqplus_mae=0.00344425 baseline_rmse=0.03058423 tqplus_rmse=0.00453193 mae_delta_pct=-86.97 rmse_delta_pct=-85.18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1976 filtered out; finished in 0.40s
```
