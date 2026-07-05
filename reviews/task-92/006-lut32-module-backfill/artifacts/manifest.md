# Manifest: Task 92 / 006-lut32-module-backfill

- head SHA: `a489a71c9078b4893ec1dbd797ecd336a7804f9a`
- task bucket: `reviews/task-92/006-lut32-module-backfill`
- timestamp: `2026-06-09T04:06:32Z`
- lane / fixture / storage format / rerank mode: focused PG18 unit validation;
  no benchmark lane; not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-test-lut32.log`

- command:
  `cargo test --lib quant::lut32::tests --no-default-features --features pg18`
- result: passed
- key result line:
  - `4 passed; 0 failed; 0 ignored; 0 measured; 2014 filtered out`

### `cargo-test-candidate-batch.log`

- command:
  `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
- result: passed
- key result line:
  - `4 passed; 0 failed; 0 ignored; 0 measured; 2014 filtered out`

### `git-diff-check.log`

- command: `git diff --check`
- result: passed with no output
