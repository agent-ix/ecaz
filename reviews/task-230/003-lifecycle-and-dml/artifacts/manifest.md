# Task 230 packet 003 artifact manifest

- Head SHA: `6d439e1e3ed1374d19e6fe9071d3251b9677ca68`
- Task bucket: `reviews/task-230/003-lifecycle-and-dml/`
- Packet: hot/cold DML and reclaim checkpoint, seq-01
- Timestamp: 2026-08-29 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: local PG18 pgrx callbacks;
  descriptor V4, Graph V2, receipt V3, manifest V4, compact paired hot/cold
  heaps; no corpus benchmark or rerank measurement
- Isolation: every focused callback creates its own transaction-scoped source
  index and generation relations; no shared-table benchmark surface

## `cargo-fmt-seq-01.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `e5615dc4c940d0a399590f25e86421255bdff6b96b091fb02156f504cbc16851`

## `cargo-clippy-seq-01.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings at
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1195`, and the existing sidecar assertion in
  `ec_distann_physical_lifecycle.rs:8835`. There is no new finding in
  `physical_dml.rs` or either new hot/cold test.
- SHA-256: `fb86e390ad91ef25796703fdc4c64b1af03a2bc01f394115e2f624f4d74adf8d`

## `cargo-pgrx-test-hot-cold-seq-01.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_ --no-default-features --features 'pg18 pg_test'`
- Result: six passed, zero failed. The group includes relation DDL/abort,
  handoff V2 locator, projection contract, typed materialization/visibility,
  new DML atomicity, and new retire/reclaim rollback coverage.
- Key result: `6 passed; 0 failed; 2636 filtered out`.
- SHA-256: `7d2137f3ade4e686d78e21c81efb03057efa662e5d3d4233048ca6ca64d76336`
