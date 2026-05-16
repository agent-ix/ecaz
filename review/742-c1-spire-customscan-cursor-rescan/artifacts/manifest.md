# Artifact Manifest: SPIRE CustomScan Cursor Rescan

agent: coder1
date: 2026-05-14
packet: 742-c1-spire-customscan-cursor-rescan
head_sha: d199e6a8

No measurement artifacts are attached. This packet adds focused pg-test
coverage and records validation commands in `request.md`.

## Validation Commands

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/mod.rs src/am/mod.rs src/tests/custom_scan_execution.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_cursor_move_first_rescans_sql --no-run`
- `cargo pgrx test pg18 test_ec_spire_customscan_cursor_move_first_rescans_sql`

## Runtime Note

The PG18 runtime attempt failed before test execution with:

```text
undefined symbol: pg_re_throw
```
