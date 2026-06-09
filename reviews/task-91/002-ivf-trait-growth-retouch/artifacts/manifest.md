# Task 91 Packet 002 Artifact Manifest

- head SHA: `4366618ad80de1f1a0a2fad65d9cb41d57050103`
- task bucket: `reviews/task-91/`
- packet path: `reviews/task-91/002-ivf-trait-growth-retouch/`
- timestamp: `2026-06-09T03:20:57Z`
- scope: Task 91 Phase 2 IVF `QuantCodec` trait-growth retouch
- lane / fixture / storage format / rerank mode: unit tests only; IVF TurboQuant, TurboQuant no-QJL LUT32, RaBitQ, and PqFastScan/GroupedPq codec paths; rerank mode not applicable
- isolated one-index-per-table vs shared-table surfaces: not applicable; unit tests do not create indexes

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key cited lines:
  - no output

### `cargo-test-ivf-quantizer.log`

- command: `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 23 tests`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_turboquant_batch_is_bit_exact_with_scalar ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_turboquant_no_qjl_lut32_batch_is_bit_exact_with_scalar ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_rabitq_batch_is_bit_exact_with_scalar ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_batch_is_bit_exact_with_scalar ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_requires_model_binding ... ok`
  - `test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_rejects_mismatched_candidate_meta ... ok`
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1988 filtered out; finished in 0.19s`
