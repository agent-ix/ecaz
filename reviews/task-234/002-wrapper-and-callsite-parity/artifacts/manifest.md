# Task 234 packet 002 artifact manifest

- Code SHA: `e2c582cffa7127a211e07677b6a78fb895d0d8cb`
- Task bucket / packet: `reviews/task-234/002-wrapper-and-callsite-parity/`
- Lane: PG18 focused read-transport unit/compile validation
- Timestamp: 2026-08-24 PDT (America/Los_Angeles)
- Fixture: pure library tests; no corpus, index, shared-table surface, or
  multinode fault fixture in this checkpoint
- Storage / rerank: unchanged; this checkpoint changes only remote read await,
  error, pooling, and aggregation behavior

Artifacts:

- `transport-tests-pg18.log`
  - Command: `cargo test --lib --no-default-features --features pg18 remote_transport::tests`
  - Result: 13 passed, 0 failed; command exit 0.
- `transport-tests-attribution-pg18.log`
  - Command: `cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark remote_transport::tests`
  - Result: 13 passed, 0 failed; command exit 0; compiles the feature-specific physical read
    rows and retry paths.
- `expand-error-tests-pg18.log`
  - Command: `cargo test --lib --no-default-features --features pg18 expand_error::tests`
  - Result: 3 passed, 0 failed; command exit 0.
- `structural-await-scan.log`
  - Command: `rg -n '\.query(_one)?\(|\.prepare\(' src/am/ec_distann/remote_transport.rs`
  - Records every async query/prepare construction for manual classification
    against packet 001's Task 234/235 allowlist. The five named read/control
    functions contain no direct query/query-one await; their futures are
    passed to `await_remote_read` or `read_query`.

Tests are intentionally focused under the repository policy. Packet 003 owns
the required PG18 multinode stalled-statement, local cancel/timeout, backend
termination, connection-reset, sibling-success/fail, clean-retry, and leaked
work assertions. No benchmark matrix is required for this correctness-only
hardening slice, and no formatter output is part of the packet.
