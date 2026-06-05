# Task 65b Packet 007 Artifact Manifest

- Head SHA: `397e57a1b921fddfb8a4c588e77493186ab71d37`
- Task bucket: `reviews/task-65b/`
- Packet path: `reviews/task-65b/007-digest-diagnostics/`
- Timestamp: `2026-06-05T02:30:02Z`
- Lane: local PG18 library + CLI renderer validation
- Fixture: in-process graph summary renderer fixture
- Storage format: not applicable for unit validation
- Rerank mode: not applicable
- Surface isolation: no shared SQL table surface; this packet adds digest fields used by later corpus runs

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `cargo check -p ecaz --lib --no-default-features --features pg18 > reviews/task-65b/007-digest-diagnostics/artifacts/cargo-check-pg18.log 2>&1`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`
- Notes:
  The visible warnings are from PostgreSQL 18 server headers included by `csrc/pg18_pgstat_shim.c`; no Rust warning was emitted for the changed code.

### `cargo-test-ecaz-cli-graph-render.log`

- Command:
  `cargo test -p ecaz-cli graph::tests::render_summary_includes_reachability_and_degree_rows > reviews/task-65b/007-digest-diagnostics/artifacts/cargo-test-ecaz-cli-graph-render.log 2>&1`
- Result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.00s`
