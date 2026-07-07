# Task 170 Packet 006 Artifact Manifest

- head SHA: `79e1f31ef5f36af5cca33610abb663507da0a767`
- task bucket: `reviews/task-170/006-tqplus-sidecar-calibration`
- timestamp: `2026-07-05T16:52:34Z`
- lane: local compile/unit validation
- fixture/storage/rerank surface:
  - `storage_format = coarse_rerank`
  - `rerank_placement = index`
  - `rerank_format = turboquant`
  - `turboquant_profile = tqplus`
  - no benchmark fixtures in this packet
- isolated one-index-per-table or shared-table surface: not applicable; no `ecaz bench suite` run in this packet

## Artifacts

### `cargo-check-ecaz-cli.log`

- sha256: `f1413ced5f345a820755387d13f232065367fc1f20bd6075deca0b14e6b0ed3e`
- command: `script -q reviews/task-170/006-tqplus-sidecar-calibration/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli`
- key result: `Finished dev profile` for `ecaz-cli`

### `cargo-test-options-tqplus-sidecar.log`

- sha256: `c6f2158af93c8760cc3a6f1d60c6ee36c7bf9e10c96d2b4ec08c878878290c0c`
- command: `script -q reviews/task-170/006-tqplus-sidecar-calibration/artifacts/cargo-test-options-tqplus-sidecar.log cargo test --release --lib coarse_rerank_accepts_tqplus_turboquant_sidecar_profile`
- key result: `test am::ec_ivf::options::tests::coarse_rerank_accepts_tqplus_turboquant_sidecar_profile ... ok`

### `cargo-test-tqplus-sidecar-rerank.log`

- sha256: `ae984a4a200c042e2667fe7cfd3be1113f03088319f270f41ce67f0927ad8642`
- command: `script -q reviews/task-170/006-tqplus-sidecar-calibration/artifacts/cargo-test-tqplus-sidecar-rerank.log cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently`
- key result: `test am::ec_ivf::rerank::tests::turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently ... ok`

## Notes

- The release-lib tests are intentionally used for macOS compatibility with the known pgrx runtime-test dyld blocker.
- This packet is an implementation checkpoint only. Slice 3 A/B measurement remains pending and must use `ecaz bench suite`.
