# Task 131 Packet 002 Artifact Manifest

- Head SHA: `34272982a6b997a0413082e56c70c58229408f6f`
- Task bucket: `reviews/task-131/`
- Packet: `reviews/task-131/002-global-preheap-merge-prototype/`
- Timestamp: `2026-07-01T04:41:31Z`
- Lane / fixture / storage format / rerank mode: code checkpoint validation only; no benchmark lane, fixture, storage format, or rerank matrix in this packet.
- Isolated one-index-per-table vs shared-table surface: not applicable; no benchmark or live SQL run.

## Artifacts

### `cargo-check-pg18.log`

- Command: `cargo check --no-default-features --features pg18 > reviews/task-131/002-global-preheap-merge-prototype/artifacts/cargo-check-pg18.log 2>&1`
- Exit status: `0`
- Key result: `Finished dev profile [unoptimized + debuginfo]`

### `git-diff-check-head.log`

- Command: `git diff --check HEAD~1..HEAD > reviews/task-131/002-global-preheap-merge-prototype/artifacts/git-diff-check-head.log 2>&1`
- Exit status: `0`
- Key result: no whitespace errors; artifact is empty.

### `cargo-test-explicit-heap-params.log`

- Command: `timeout 150s cargo test explicit_heap_candidate_parameters_encode_binary_fields_as_hex --no-default-features --features pg18 > reviews/task-131/002-global-preheap-merge-prototype/artifacts/cargo-test-explicit-heap-params.log 2>&1`
- Exit status: `124`
- Key result: timed out after 150 seconds while compiling `ecaz`; no test execution result was emitted.

