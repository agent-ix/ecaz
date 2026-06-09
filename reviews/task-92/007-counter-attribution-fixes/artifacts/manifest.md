# Task 92 Packet 007 Artifact Manifest

- head SHA: `4c7c9bab824eea9ace420ad96175aa616f548408`
- task bucket: `reviews/task-92`
- packet path: `reviews/task-92/007-counter-attribution-fixes`
- timestamp: `2026-06-08T21:18:03-07:00`
- target: PG18, Graviton 4 SVE2 counter contract
- isolated one-index-per-table or shared-table surface: not applicable; unit
  validation only

## Artifacts

### `artifacts/cargo-test-candidate-batch.log`

- command:
  `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
- result:
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2014 filtered out; finished in 0.06s`
- coverage:
  candidate batch counter attribution, Task 87 compatibility counters, and
  scalar-tail `isa=Scalar` row attribution.

### `artifacts/cargo-test-bench-module.log`

- command:
  `cargo test -p ecaz-cli commands::bench::tests --no-default-features`
- result:
  `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 401 filtered out; finished in 0.00s`
- coverage:
  bench CLI block-kernel counter line parsing and transition-format output.

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- result: passed with no output.
