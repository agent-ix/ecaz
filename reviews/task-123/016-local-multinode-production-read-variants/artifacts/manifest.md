# Task 123 Packet 016 Artifact Manifest

- Head SHA: `cb1304515`
- Task bucket: `reviews/task-123/`
- Packet path: `reviews/task-123/016-local-multinode-production-read-variants/`
- Timestamp: `2026-06-28T09:05:29-07:00`
- Purpose: document and validate the local multinode production-read variant runner extension.
- Isolated one-index-per-table or shared-table surface: not applicable; this packet contains CLI runner plumbing and focused unit validation only.

## Artifacts

### `artifacts/cargo-fmt-check.log`

- Head SHA: `cb1304515`
- Command: `script -q -e -c 'cargo fmt --check' reviews/task-123/016-local-multinode-production-read-variants/artifacts/cargo-fmt-check.log`
- Timestamp: `2026-06-28T09:04:53-07:00`
- Result: command exited 0.
- Key result line:
  - `Script done ... [COMMAND_EXIT_CODE="0"]`

### `artifacts/cargo-test-suite-local-multinode-variants.log`

- Head SHA: `cb1304515`
- Command: `script -q -e -c 'cargo test -p ecaz-cli spire_local_multinode_step_expands_local_four_instance_lane -- --nocapture' reviews/task-123/016-local-multinode-production-read-variants/artifacts/cargo-test-suite-local-multinode-variants.log`
- Timestamp: `2026-06-28T09:04:55-07:00`
- Result: command exited 0.
- Key result lines:
  - `test commands::bench::suite::tests::spire_local_multinode_step_expands_local_four_instance_lane ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 418 filtered out; finished in 0.00s`

### `artifacts/cargo-test-production-read-variant-parser.log`

- Head SHA: `cb1304515`
- Command: `script -q -e -c 'cargo test -p ecaz-cli parses_bench_production_read_variant -- --nocapture' reviews/task-123/016-local-multinode-production-read-variants/artifacts/cargo-test-production-read-variant-parser.log`
- Timestamp: `2026-06-28T09:05:12-07:00`
- Result: command exited 0.
- Key result lines:
  - `test commands::dev::spire_multicluster::tests::parses_bench_production_read_variant ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 418 filtered out; finished in 0.00s`
