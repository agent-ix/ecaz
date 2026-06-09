# Task 87 Packet 011 Artifact Manifest

- head SHA: `e0942a33d1a75ebc41094eee329820e9b6472742`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/011-ivf-turboquant-scan-route/`
- timestamp: `2026-06-08T19:45:03Z`
- scope: IVF TurboQuant no-QJL 4-bit CandidateBatch scan route reachability
- lane / fixture / storage format / rerank mode: unit tests only; IVF TurboQuant no-QJL 4-bit gate coverage; no corpus lane or rerank mode
- isolated one-index-per-table vs shared-table surfaces: not applicable

## Artifacts

### `cargo-test-ivf-scan.log`

- command: `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 24 tests`
  - `test am::ec_ivf::scan::tests::scratch_soa_batch_decode_gate_admits_turboquant_no_qjl4_and_rabitq_lanes ... ok`
  - `test result: ok. 24 passed; 0 failed`

### `cargo-test-ivf-quantizer.log`

- command: `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 17 tests`
  - `test am::ec_ivf::quantizer::tests::turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane ... ok`
  - `test am::ec_ivf::quantizer::tests::turboquant_no_qjl_4bit_batch_scores_match_scalar_scores ... ok`
  - `test result: ok. 17 passed; 0 failed`
