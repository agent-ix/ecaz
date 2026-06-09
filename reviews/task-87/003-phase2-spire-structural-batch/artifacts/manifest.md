# Task 87 Packet 003 Artifact Manifest

- head SHA: `e3061b4e0`
- task bucket: `reviews/task-87`
- packet path: `reviews/task-87/003-phase2-spire-structural-batch`
- lane: Phase 2 SPIRE structural CandidateBatch integration
- fixture: focused Rust unit tests
- storage format: TurboQuant no-QJL 4-bit structural route; no on-disk
  format change
- rerank mode: not applicable
- timestamp: 2026-06-08
- surface mode: no benchmark run in this packet; real-corpus suite still
  required before Phase 2 acceptance

## Artifacts

- `cargo-test-candidate-batch.log` — focused shared CandidateBatch unit
  tests.
- `cargo-test-spire-quantizer.log` — focused SPIRE quantizer/scorer unit
  tests, including the no-QJL 4-bit LUT batch path.

## Commands

```text
cargo test --lib am::common::candidate_batch --no-default-features --features pg18
cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18
```

## Key Result Lines Cited By Request

- `running 2 tests`
- `test result: ok. 2 passed; 0 failed`
- `running 12 tests`
- `test result: ok. 12 passed; 0 failed`
