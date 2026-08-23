# Task 222 packet 002 artifact manifest

- Head SHA: `f088021ead2b4a15aa9e6d7bcc0a6bf24368ca42`
- Task bucket: `reviews/task-222/`
- Packet: `reviews/task-222/002-contract-and-correctness/`
- Timestamp: `2026-08-23T14:53:45-07:00`
- Lane / fixture: PG18 focused three-owner physical handoff, mixed local and
  loopback-remote owners
- Storage format / rerank mode: existing physical-generation fixture;
  `benchmark_exact_neighbor=on`; no storage-format change
- Isolation: correctness test, not a benchmark; no shared-table benchmark
  surface and no `ecaz bench suite` result is claimed

## Artifacts

### `pg18-focused.log`

Command:

`cargo pgrx test pg18 test_distann_payload_projection_contract --no-default-features --features pg18`

Key result: `test tests::pg_test_distann_payload_projection_contract ... ok`;
`1 passed; 0 failed; 2578 filtered out`.

### `cargo-check.log`

Command:

`cargo check --lib --no-default-features --features pg18`

Key result: finished successfully.

### `clippy.log`

Command:

`cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Result: failed on four pre-existing warnings outside the Task 222 changed
files: `ambuild.rs` (`collapsible_if`), `generation_descriptor.rs`
(`unnecessary_unwrap`), `head_sample.rs` (`needless_range_loop`), and
`remote_endpoint.rs` (`items_after_test_module`).

### `clippy-task222.log`

Command:

`cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings -A clippy::collapsible-if -A clippy::unnecessary-unwrap -A clippy::needless-range-loop -A clippy::items-after-test-module`

Key result: finished successfully, showing no additional warning after only
the four recorded baseline lints are suppressed.
