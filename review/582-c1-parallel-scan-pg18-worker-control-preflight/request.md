# Review Request: Parallel Scan PG18 Worker Control Preflight

Current head: `0b65bf9cd4a43f280ae5bd8ee27e47c5201c1f75`

Scope:
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The first PG18 live preflight proved ordered-ID parity but only showed that
  the `ec_hnsw` query still planned as a serial `Index Scan`.
- That did not distinguish between PostgreSQL being unable to launch workers
  for the fixture and PostgreSQL not choosing a partial/parallel index path for
  the access method.

What changed:
- Added a forced parallel sequential-scan control plan to
  `ecaz dev test pg18-parallel-scan` using the same committed PG18 fixture.
- The command now reports both the ordered `ec_hnsw` plan and a worker-launch
  control plan.
- `--expect-parallel` failures include both plans, so future planner work can
  tell whether a failure is due to worker availability or index path selection.
- Updated the Task 18 plan note to record that the fixture can launch workers,
  while the ordered `ec_hnsw` plan remains serial.

Artifact:
- `artifacts/pg18-parallel-scan.log` captures the PG18 preflight output.
- The log shows a serial ordered `Index Scan` for `ec_hnsw`, plus a control
  `Gather` plan with four workers launched for the same fixture.

Validation:
- Passed before code checkpoint `0b65bf9`:
  - `cargo fmt --check`
  - `cargo check -p ecaz-cli`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_planner_integration_snapshot_reports_blockers`
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
  - `cargo test -p ecaz-cli`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Packet artifact generated with:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`

Review focus:
- Whether the forced parallel seqscan is the right control proof for PG18 worker
  availability.
- Whether `--expect-parallel` should continue to include both plan shapes when
  the ordered index path is still serial.
- Whether the next implementation slice should focus on AM planner path
  activation/cost competitiveness now that fixture-level worker launch is
  isolated.
