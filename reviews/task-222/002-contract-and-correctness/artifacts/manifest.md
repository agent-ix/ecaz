# Task 222 packet 002 artifact manifest

- Head SHA: `c9f79be4a756031b3f8301960fc0f57b77ae60d1`
- Supporting SHAs: `54802d299` (query expression context), `010a0accc`
  (refreshed snapshot lifetime)
- Task bucket: `reviews/task-222/`
- Packet: `reviews/task-222/002-contract-and-correctness/`
- Timestamp: `2026-08-23T21:47:54-07:00`
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

This was the initial contract-only checkpoint at `f088021ea`.

### `pg18-focused-rescan.log`

Command:

`cargo pgrx test pg18 test_distann_payload_projection_contract --no-default-features --features pg18`

Key result at `c9f79be4a`: `test
tests::pg_test_distann_payload_projection_contract ... ok`; `1 passed; 0
failed; 2578 filtered out; finished in 138.11s`.

This is intentionally a heavyweight three-owner PG18 fixture. The expanded
case executes null/toast, cached and forced-generic external Params,
`PARAM_EXEC` LATERAL rescans, multi-window qual rejection, EPQ/concurrent
update, and post-first-batch remote failure in addition to the original
contract cases.

### `pg18-focused-gdb-failed.log` and `gdb-backtrace.log`

Command surface: the same focused PG18 test, with the PostgreSQL test backend
attached under GDB after the repeatable crash.

Key result: `SIGSEGV` reached `XidInMVCCSnapshot` from
`GenerationExpander::expand_nodes_masked`; the snapshot's xid arrays contain
freed-memory poison (`0x7f...`). This established that a refreshed registered
snapshot was dropped while its raw pointer survived into later hop rounds.
Commit `010a0accc` retains its guard in `GenerationExpander.retry_snapshot`.
The subsequent `pg18-focused-rescan.log` passes the formerly crashing path.

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
