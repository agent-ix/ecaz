# Artifact Manifest: SPIRE Storage Snapshot During REINDEX

- Head SHA: `0af00b2dfbda28003434faf13757be2d7b27053d`
- Packet/topic: `750-c1-spire-storage-snapshot-reindex`
- Lane / fixture / storage format / rerank mode: PG18 SPIRE diagnostics concurrency fixture; default single-store ecvector SPIRE index; default rerank settings.
- Isolated one-index-per-table or shared-table surfaces: isolated table and index for the REINDEX snapshot fixture.
- Timestamp: `2026-05-15T01:27:09Z`

## Validation Commands

### `cargo fmt --check`

- Command: `cargo fmt --check`
- Result: passed
- Key lines: command exited 0; only the pre-existing stable rustfmt warnings about `imports_granularity` and `group_imports` were emitted.

### `git diff --check`

- Command: `git diff --check -- src/tests/diagnostics_reindex.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- Result: passed
- Key lines: command exited 0 with no whitespace findings.

### Focused Compile

- Command: `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_relation_storage_snapshot_during_reindex_sql --no-run`
- Result: passed
- Key lines: `Finished test profile ...`; test executables were produced.

### Focused Runtime Attempt

- Command: `cargo pgrx test pg18 test_ec_spire_relation_storage_snapshot_during_reindex_sql`
- Result: blocked before test execution by environment loader failure
- Key lines: `/home/peter/dev/ecaz/target/debug/deps/ecaz-4a6e0718723ccfd4: symbol lookup error: ... undefined symbol: pg_re_throw`
