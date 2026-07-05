# Artifact Manifest: SPIRE CustomScan Planner Exclusions

- Head SHA: `389705f5787e23229566514dc05db0e18f335e18`
- Packet/topic: `749-c1-spire-customscan-planner-exclusions`
- Lane / fixture / storage format / rerank mode: PG18 SPIRE planner JSON EXPLAIN fixtures; coordinator-only planner assertions; default storage/rerank settings.
- Isolated one-index-per-table or shared-table surfaces: isolated tables per planner fixture.
- Timestamp: `2026-05-15T01:21:29Z`

## Validation Commands

### `cargo fmt --check`

- Command: `cargo fmt --check`
- Result: passed
- Key lines: command exited 0; only the pre-existing stable rustfmt warnings about `imports_granularity` and `group_imports` were emitted.

### `git diff --check`

- Command: `git diff --check -- src/tests/custom_scan_planner.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- Result: passed
- Key lines: command exited 0 with no whitespace findings.

### Focused Compile

- Command: `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_not --no-run`
- Result: passed
- Key lines: `Finished test profile ...`; test executables were produced.

### Focused Runtime Attempt

- Command: `cargo pgrx test pg18 test_ec_spire_customscan_not`
- Result: blocked before test execution by environment loader failure
- Key lines: `/home/peter/dev/ecaz/target/debug/deps/ecaz-4a6e0718723ccfd4: symbol lookup error: ... undefined symbol: pg_re_throw`
