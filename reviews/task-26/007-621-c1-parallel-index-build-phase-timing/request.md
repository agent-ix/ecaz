# Review Request: Parallel Index Build Phase Timing

Current head: `9dcc8e3`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Problem:
- Packet 620 showed the first executable parallel build path is slightly
  slower than serial on a 10k x 64 PG18 fixture.
- That result is actionable only if we can separate heap ingestion, worker
  queue/merge overhead, graph construction, staging, and page writes.

What changed:
- Added last-build timing counters for:
  - requested workers
  - launched workers
  - heap tuples
  - index tuples
  - heap ingestion wall time
  - parallel context begin/setup time
  - parallel queue drain/finish time
  - parallel sort plus `BuildState::push` time
  - total flush time
  - graph construction time
  - tuple/page staging time
  - physical page write time
- Added `tests.ec_hnsw_debug_last_build_timing()` under the existing pg_test
  debug schema.
- Extended the PG18 parallel build smoke test to assert the timing surface is
  populated for the worker path.

Interpretation:
- This does not claim a speedup.
- The purpose is to make the next measurement discriminate between transport
  overhead and serial graph assembly, so the next code slice is based on phase
  evidence rather than threshold tuning.

Validation:
- Passed:
  - `cargo test build_parallel --lib`
  - `cargo pgrx test pg18 test_pg18_parallel_index_build_uses_workers`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo test`
  - `cargo pgrx test pg18`
  - `git diff --check`

Review focus:
- Whether this debug surface exposes the right phase boundaries.
- Whether the timing counters should remain pg_test-only at the SQL layer.
- Whether any phase naming should change before measurement packets start
  depending on the surface.
