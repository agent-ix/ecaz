# Task 87 Packet 004 Artifact Manifest

- head SHA: `a382d756f23f6c20e12dfb03eb34608223ffa3fa`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/004-phase3-ivf-structural-batch/`
- timestamp: `2026-06-08T18:07:44Z`
- scope: IVF structural CandidateBatch route for TurboQuant no-QJL 4-bit scoring
- lane / fixture / storage format / rerank mode: unit tests only; no corpus lane; `ec_ivf` TurboQuant no-QJL 4-bit and scan helper coverage; rerank mode not applicable
- isolated one-index-per-table vs shared-table surfaces: not applicable; unit tests do not create indexes

## Artifacts

### `cargo-test-ivf-quantizer.log`

- command: `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 14 tests`
  - `test am::ec_ivf::quantizer::tests::turboquant_no_qjl_4bit_batch_scores_match_scalar_scores ... ok`
  - `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1979 filtered out; finished in 0.18s`

### `cargo-test-ivf-scan.log`

- command: `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 23 tests`
  - `test am::ec_ivf::scan::tests::posting_scratch_soa_batches_fields_without_losing_order ... ok`
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1970 filtered out; finished in 0.00s`
