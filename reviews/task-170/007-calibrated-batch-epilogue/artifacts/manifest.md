# Task 170 Packet 007 Artifact Manifest

- head SHA: `c49fc3e317b60b2a307739296364ef2185996044`
- task bucket: `reviews/task-170/007-calibrated-batch-epilogue`
- timestamp: `2026-07-05T17:21:12Z`
- lane: local compile/unit validation
- fixture/storage/rerank surface:
  - `storage_format = turboquant`, `turboquant_profile = tqplus` for calibrated posting scoring
  - `storage_format = coarse_rerank`, `rerank_placement = index`, `rerank_format = turboquant`, `turboquant_profile = tqplus` for calibrated sidecar scoring
  - no benchmark fixtures in this packet
- isolated one-index-per-table or shared-table surface: not applicable; no `ecaz bench suite` run in this packet

## Artifacts

### `cargo-check-ecaz-cli.log`

- sha256: `91be6495b9b748e569c59b1e2f5ff8babf09234bd85929a1f5281cf35faf0d30`
- command: `script -q reviews/task-170/007-calibrated-batch-epilogue/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli`
- key result: `Finished dev profile` for `ecaz-cli`

### `cargo-test-tqplus-sidecar-rerank.log`

- sha256: `c2dd10660c3f2f2778414b7a174c8f2402d47e8a25fb24c0981e9766074ffc7f`
- command: `script -q reviews/task-170/007-calibrated-batch-epilogue/artifacts/cargo-test-tqplus-sidecar-rerank.log cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently`
- key result: `test am::ec_ivf::rerank::tests::turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently ... ok`

### `cargo-test-calibration-core.log`

- sha256: `e37f007caadc4060f6b10f74861df012cdc3ec5797f88ca3a73f5f55936784b8`
- command: `script -q reviews/task-170/007-calibrated-batch-epilogue/artifacts/cargo-test-calibration-core.log cargo test --release --lib calibration_no_qjl_4bit_reduces_anisotropic_score_error`
- key result: `test quant::prod::tests::calibration_no_qjl_4bit_reduces_anisotropic_score_error ... ok`

## Notes

- The calibrated query now uses the existing i16 LUT32 kernel and applies only the scalar calibration bias in a per-candidate epilogue.
- This packet is an implementation checkpoint. Slice 3 A/B measurement remains pending and must use `ecaz bench suite`.
