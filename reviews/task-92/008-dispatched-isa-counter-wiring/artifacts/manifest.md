# Task 92 Packet 008 Artifact Manifest

- head SHA: `cbabe7c07e3718961e07ec3e952794cdb013aecd`
- task bucket: `reviews/task-92`
- packet path: `reviews/task-92/008-dispatched-isa-counter-wiring`
- timestamp: `2026-06-08T21:33:34-07:00`
- target: PG18, Graviton 4 SVE2 counter-attribution precondition
- isolated one-index-per-table or shared-table surface: not applicable; unit
  validation only

## Artifacts

### `artifacts/cargo-test-lut32.log`

- command:
  `cargo test --lib quant::lut32::tests --no-default-features --features pg18`
- result:
  `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2018 filtered out; finished in 0.00s`
- coverage:
  LUT32 block and tail parity tests, including the current fallback backend
  attribution value.

### `artifacts/cargo-test-candidate-batch.log`

- command:
  `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
- result:
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2017 filtered out; finished in 0.06s`
- coverage:
  candidate-batch counter attribution, Task 87 compatibility counters, and
  scalar-tail `isa=Scalar` row attribution.

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- result: passed with no output.
