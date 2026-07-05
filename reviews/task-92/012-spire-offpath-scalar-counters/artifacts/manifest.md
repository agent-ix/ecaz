# Task 92 Packet 012 Artifact Manifest

- head SHA: `b5cf53f28900`
- task bucket: `reviews/task-92`
- packet path: `reviews/task-92/012-spire-offpath-scalar-counters`
- timestamp: `2026-06-08T22:03:12-07:00`
- lane: Task 92 off-path scalar counter wiring
- fixture: focused Rust unit tests
- storage format: TurboQuant no-QJL 4-bit
- rerank mode: not applicable
- table surface: no PostgreSQL benchmark tables were created

## Artifacts

### `artifacts/cargo-test-candidate-batch.log`

- command: `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
- purpose: validate block-kernel/scalar counter split and deterministic global
  counter tests
- key result lines:
  - `test am::common::candidate_batch::tests::block_kernel_counter_api_records_scalar_tail_under_scalar_isa ... ok`
  - `test am::common::candidate_batch::tests::turboquant_lut_batch_records_surface_counters ... ok`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2017 filtered out; finished in 0.06s`

### `artifacts/cargo-test-spire-quantizer.log`

- command: `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
- purpose: ensure SPIRE QuantCodec/scorer behavior remains unchanged after
  adding scalar fallback timing attribution
- key result lines:
  - `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 2004 filtered out; finished in 0.11s`

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- purpose: whitespace check for the code and packet diff
- key result lines:
  - `COMMAND_EXIT_CODE="0"`
