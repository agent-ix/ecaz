# Review Request: Parallel Scan N3 Duplicate Handoff Gate

Current head: `a7a43ee`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged round-robin harness only exercised two worker slots, so a
  duplicate output leak could hide until a third worker consumed a stale foreign
  handoff row after another worker had already emitted that heap TID.
- The existing duplicate suppression guarded active local rows, but handoff
  output taken from another worker's selected/admitted/hidden source could still
  be returned after that heap TID was already visible in a foreign worker's
  emitted snapshot.

What changed:
- Generalized the staged round-robin debug helper so tests can bind an
  arbitrary worker count to the same parallel DSM allocation and capture each
  worker's stream, runtime snapshot, hidden-slot snapshot, visited/emitted
  sets, and EXPLAIN counters.
- Added a PG18 `n=3` round-robin gate that requires:
  - the combined stream to remain byte-identical to serial
  - all three workers to contribute output
  - no stranded hidden-slot rows, blocker metadata, or active result state
  - no local-only or deferred-local fallback emits
- Split foreign handoff into a state-returning path so duplicate handoff
  suppression can advance/clear the stale source slot without returning the
  already-emitted heap TID to the caller.
- Added a focused unit test for suppressing a stale foreign selected handoff
  after a different worker has already emitted the same heap TID.
- Updated the Task 18 notes to record the three-worker gate and duplicate
  handoff suppression contract.

Why this matters:
- This moves the staged coordinator contract beyond the easier `n=2` case and
  catches cross-worker output leaks that only appear once there is a third
  participant in the shared DSM.
- A suppressed duplicate is no longer counted as a user-visible foreign
  handoff, which keeps EXPLAIN counters aligned with emitted output.
- `amcanparallel` still stays off, but the ownership-transfer surface is now
  materially closer to a real multi-worker contract.

Still intentionally deferred:
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness coverage
- final live parallel plan benchmarks and recall parity measurements

Validation:
- Passed:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n2_round_robin_matches_serial_scores`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n2_round_robin_matches_serial_duplicate_drain`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n3_round_robin_matches_serial_scores`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether suppressing a duplicate inside the foreign-handoff path is the right
  layer, rather than requiring every caller to handle stale foreign rows.
- Whether the new `SuppressedDuplicate` state is threaded through all currently
  reachable handoff callers without accidentally dropping a still-live local
  staged row.
- Whether the generalized round-robin helper is sufficient for the next `n=4/8`
  gates, or if it should also expose per-turn scheduling traces before
  planner-visible enablement.
