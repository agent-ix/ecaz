# Review Request: Parallel Scan PG18 Live Preflight

Current head: `96a2d87de2e03733acb79d28e0e299cdadbd2552`

Scope:
- `crates/ecaz-cli/src/commands/dev/support.rs`
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `src/am/ec_hnsw/routine.rs`
- `src/am/ec_hnsw/shared.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged coordinator is covered through `n=8`, but the next live
  enablement step cannot be proven inside pg_test transaction scope.
- An exploratory `amcanparallel = true` flip did not make PostgreSQL choose a
  real `Parallel Index Scan`; the live plan stayed serial, and
  `debug_parallel_query` only produced a `Gather Single Copy` wrapper around
  that serial path.
- We need this investigation under `ecaz-cli`, not in one-off scripts.

What changed:
- Refactored the PG18 preload validation command to share a repo-local PG18
  cluster setup helper.
- Added `ecaz dev test pg18-parallel-scan`, which creates a committed PG18
  fixture, applies parallel-friendly planner GUCs, compares serial IDs against
  the parallel-enabled candidate session, and reports the actual plan shape.
- Added `--expect-parallel` so the same command can become a hard live executor
  gate once PostgreSQL produces a real `Parallel Index Scan`.
- Kept `amcanparallel = false` and updated the planner snapshot blocker text
  and Task 18 notes to reflect the actual blocker: planner path activation, not
  the already-staged coordinator ownership contract.

Artifact:
- `artifacts/pg18-parallel-scan.log` captures the PG18 preflight output.
- The log shows stable serial/candidate IDs and the current serial `Index Scan`
  plan, so the packet documents why `amcanparallel` is still disabled.

Validation:
- Passed:
  - `cargo fmt --check`
  - `cargo check -p ecaz-cli`
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_planner_integration_snapshot_reports_blockers`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n8_round_robin_matches_serial_scores`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the default `pg18-parallel-scan` behavior should stay diagnostic, with
  `--expect-parallel` as the future hard gate, or fail by default before planner
  path activation lands.
- Whether the 512-row fixture and `ef_search=1000` default are the right balance
  for fast repeatability and full-traversal-sized ordered-ID parity.
- Whether the updated blocker text is sufficiently precise for other agents to
  pick up the planner-path work without re-opening coordinator ownership work.
