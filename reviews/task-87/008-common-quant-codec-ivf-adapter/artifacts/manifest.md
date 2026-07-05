# Task 87 Packet 008 Artifact Manifest

- head SHA: `9ae54988f75440197079f685b473a05cfdcff46f`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/008-common-quant-codec-ivf-adapter/`
- timestamp: `2026-06-08T18:27:40Z`
- scope: first common quant codec surface plus IVF adapter for TurboQuant, RaBitQ, and grouped-PQ/PqFastScan scoring
- lane / fixture / storage format / rerank mode: unit tests only; no corpus lane; IVF TurboQuant/RaBitQ/PqFastScan codec adapter coverage; rerank mode not applicable
- isolated one-index-per-table vs shared-table surfaces: not applicable; unit tests do not create indexes

## Artifacts

### `cargo-test-candidate-batch.log`

- command: `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 2 tests`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1995 filtered out; finished in 0.00s`

### `cargo-test-ivf-quantizer.log`

- command: `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 17 tests`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_scores_turboquant_batch ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_scores_rabitq_batch ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_scores_grouped_pq_batch_with_prepared_model ... ok`
  - `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 1980 filtered out; finished in 0.18s`
