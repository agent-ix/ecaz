# Review Request: Parallel Scan N4 Round-Robin DSM Sizing Gate

Current head: `a2a86c9`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The generalized debug round-robin harness could request more worker slots
  than the current `max_parallel_workers_per_gather + 1` default capacity, but
  its synthetic DSM allocation still used the default descriptor size.
- That was invisible at `n=3` on the local PG18 setup, but an explicit `n=4`
  gate under-allocated the AM-private DSM and aborted the backend before the
  coordinator assertions could run.

What changed:
- Exposed the descriptor-size helper for an explicit worker-slot count within
  the crate.
- Sized debug DSM allocation from the requested worker count instead of the
  GUC-derived default capacity.
- Factored the many-worker round-robin assertions into a shared PG18 test
  helper.
- Added a PG18 `n=4` round-robin gate that requires serial-equivalent output,
  all four workers contributing, no stranded hidden/blocker/active state, and
  no local-only/deferred-local fallback emits.
- Updated Task 18 notes with the n=4 DSM sizing/gate coverage.

Why this matters:
- The test harness can now exercise worker counts beyond the local planner GUC
  capacity without corrupting the synthetic DSM allocation.
- The staged coordinator contract is now covered through four participants,
  which is the next meaningful stress point before `n=8` and eventual
  planner-visible enablement.

Still intentionally deferred:
- planner-visible enablement and `amcanparallel = true`
- `n=8` staged correctness coverage
- live parallel-plan benchmark and recall measurements

Validation:
- Passed:
  - `cargo fmt --check`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n4_round_robin_matches_serial_scores`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether making `ec_parallel_scan_descriptor_size_for` crate-visible is the
  right seam for debug harnesses, or if this should stay hidden behind a
  dedicated debug allocation wrapper.
- Whether the n=4 fixture is strong enough as the next worker-count gate before
  moving to `n=8`.
