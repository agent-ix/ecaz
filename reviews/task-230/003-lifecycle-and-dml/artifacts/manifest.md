# Task 230 packet 003 artifact manifest

- Head SHA: `760ed15a75bb2f3ed499665b8dbc0b7b3cd92a3c`
- Task bucket: `reviews/task-230/003-lifecycle-and-dml/`
- Packet: hot/cold topology checkpoint, seq-02
- Timestamp: 2026-08-29 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: local PG18 pgrx callbacks;
  descriptor V4, Graph V2, receipt V3, manifest V4, compact paired hot/cold
  heaps; no corpus benchmark or rerank measurement
- Isolation: every focused callback creates its own transaction-scoped source
  index and generation relations; no shared-table benchmark surface

## Seq-02 topology artifacts

All seq-02 artifacts were produced at
`760ed15a75bb2f3ed499665b8dbc0b7b3cd92a3c` on 2026-08-29 PDT.

### `cargo-fmt-seq-02.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `bdcee12933694b90dbf4905fbea8a7d44c55e817841648513a37e48913c2a0d2`

### `cargo-clippy-seq-02.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01; no finding in `handoff.rs` or the new topology test.
- SHA-256: `8a8645b4026124507d16252044418fdf8b7dcc4c8050c74251e2b20e56d386af`

### `cargo-pgrx-test-topology-seq-02.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_topology_reports_both_tiers --no-default-features --features 'pg18 pg_test'`
- Result: one passed, zero failed; 2,642 filtered out. The callback proves Graph
  V2 diagnostic admission, logical reconstruction/digest equality, separate
  hot/cold row counts and orphan counts, and both heap byte values.
- SHA-256: `04d98fa364adffad5dd31b9a097a8f75b39888ee23536a158c0bdb7ce4218d39`

## Seq-01 DML and reclaim artifacts

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
