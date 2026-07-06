# Task 145 Packet 003 Artifact Manifest

- head SHA: `4d7e927f02ac87fca2c8c1fc7bfa5b19a75e2a51`
- branch: `task-145-spire-rerank-economy-low-probe`
- task bucket: `reviews/task-145/003-remote-rerank-width/`
- packet type: code checkpoint with focused tests
- timestamp: 2026-07-06T12:00:32Z
- lane / fixture / storage format / rerank mode: coordinator remote-candidate
  dispatch path; no corpus fixture; rerank mode is effective
  `ec_spire.rerank_width`
- isolated/shared surface: not applicable; focused unit tests only

## Code Under Review

Code commit: `4d7e927f0 fix(task-145): honor rerank width for remote heap rescore`

The slice addresses Task 145 packet 001 feedback that the remote coordinator
path still ignored `rerank_width`:

- `hierarchy_snapshots.rs` now applies effective rerank width before exact
  heap rescore of compact remote candidates.
- Width `0` keeps the full frontier; positive widths truncate the sorted remote
  frontier before exact heap resolution.
- Production remote candidate/heap request state carries
  `effective_rerank_width`.
- The production libpq heap receive path sets remote session
  `ec_spire.rerank_width` with `set_config` before running heap receive SQL.

This packet does not claim Task 145 closeout. Remote `ecaz bench suite`
`remote:true` release A/B and the remaining Task 145 phases are still owed.

## Validation Artifacts

### `cargo-test-remote-heap-rerank-prefix.log`

- command: `cargo test remote_heap_rerank_prefix --no-default-features --features pg18`
- result: pass
- key lines:
  - `running 2 tests`
  - `remote_heap_rerank_prefix_limits_exact_heap_resolution_width ... ok`
  - `remote_heap_rerank_prefix_keeps_full_frontier_for_zero_width ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2268 filtered out`

### `cargo-test-production-compact-request-width.log`

- command: `cargo test production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18`
- result: pass
- key lines:
  - `production_executor_compact_receive_requests_use_dispatch_state ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2269 filtered out`

### `cargo-test-production-heap-request-width.log`

- command: `cargo test production_executor_heap_receive_requests_carry_tuple_payload_columns --no-default-features --features pg18`
- result: pass
- key lines:
  - `production_executor_heap_receive_requests_carry_tuple_payload_columns ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2269 filtered out`

