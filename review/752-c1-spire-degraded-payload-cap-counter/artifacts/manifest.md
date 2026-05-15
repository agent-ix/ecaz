# Artifact Manifest: SPIRE Degraded Payload Cap Counter

- Head SHA: `d565afb6f107421e8096d7fafaf98d91cf11b65b`
- Packet/topic: `752-c1-spire-degraded-payload-cap-counter`
- Lane / fixture / storage format / rerank mode: Rust executor-state unit coverage for degraded remote payload cap; no storage/rerank lane.
- Isolated one-index-per-table or shared-table surfaces: not applicable; pure executor-state unit test.
- Timestamp: `2026-05-15T01:35:44Z`

## Validation Commands

### `cargo fmt --check`

- Command: `cargo fmt --check`
- Result: passed
- Key lines: command exited 0; only the pre-existing stable rustfmt warnings about `imports_granularity` and `group_imports` were emitted.

### `git diff --check`

- Command: `git diff --check -- src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- Result: passed
- Key lines: command exited 0 with no whitespace findings.

### Focused Compile

- Command: `cargo test --features "pg18 pg_test" --no-default-features degraded_skip_report_hints_remote_payload_cap --no-run`
- Result: passed
- Key lines: `Finished test profile ...`; test executables were produced.

### Focused Runtime Attempt

- Command: `cargo test --features "pg18 pg_test" --no-default-features degraded_skip_report_hints_remote_payload_cap`
- Result: blocked before test execution by environment loader failure
- Key lines: `/home/peter/dev/ecaz/target/debug/deps/ecaz-4a6e0718723ccfd4: symbol lookup error: ... undefined symbol: pg_re_throw`
