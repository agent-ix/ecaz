# Task 87 Packet 006 Artifact Manifest

- head SHA: `c44c7fe6c3d886419174c1996e5dfb78d9cceb7f`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/006-phase5-hnsw-structural-batch/`
- timestamp: `2026-06-08T18:18:06Z`
- scope: HNSW structural CandidateBatch route for TurboQuant `FullLut` no-QJL 4-bit exact scoring
- lane / fixture / storage format / rerank mode: unit tests only; no corpus lane; `ec_hnsw` TurboQuant `FullLut` exact-score helper coverage; rerank mode not applicable
- isolated one-index-per-table vs shared-table surfaces: not applicable; unit tests do not create indexes

## Artifacts

### `cargo-test-hnsw-scan.log`

- command: `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
- result: passed
- key cited lines:
  - `running 74 tests`
  - `test am::ec_hnsw::scan::tests::turboquant_full_lut_payload_batch_matches_scalar_and_caches_scores ... ok`
  - `test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 1920 filtered out; finished in 0.07s`
