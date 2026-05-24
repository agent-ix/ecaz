# Task 50 Packet 207 Artifact Manifest

- Head SHA: `82dc514b9823b1808c3b5aa62d3ab2b3dee8230d`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/207-spire-dml-catalog-heap-guard`
- Timestamp: `2026-05-21T09:57:34Z`
- Lane: Task 50 unsafe burndown / SPIRE DML catalog heap guard
- Fixture / storage format / rerank mode: not applicable
- Shared-table or isolated one-index-per-table: not applicable; compile/test-only validation

## Artifacts

### `rustfmt-check.log`

- Command: `rustfmt --check src/am/ec_spire/dml_frontdoor/mod.rs`
- Result: passed.
- Key lines: rustfmt emitted the existing stable-channel warnings for nightly-only import grouping options; no formatting failures.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed.
- Key lines: `Finished dev profile`; existing `src/am/mod.rs` unused import warning remains.

### `cargo-test-dml-frontdoor-pg18-pgtest-no-run.log`

- Command: `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run`
- Result: passed.
- Key lines: `Finished test profile`; existing Hadamard test-helper dead-code warnings remain.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed with no output.

## Unsafe Contract Snapshot

- `src/am/ec_spire/dml_frontdoor/mod.rs` unsafe function contracts: `27 -> 20`.
- Raw `src` unsafe-reference count after this slice: `2575`.
