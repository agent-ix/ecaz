# Task 145 Packet 005 Artifact Manifest

Task bucket: `reviews/task-145/005-remote-heap-frontier-limit`
Branch: `task-145-spire-rerank-economy-low-probe`
Head SHA: `5819b313aeec7525c90b865855aa20129876febc`
Recorded: `2026-07-06T13:17:08Z`

## Scope

Code checkpoint after packet 004 showed release remote A/B engaging the remote
path but leaving `remote_heap_candidate_sum` unchanged for
`ec_spire.rerank_width=0` vs `50`.

Root cause: the production heap-resolution path used the benchmark/query
`top_k` as both:

- the final result merge limit, and
- the compact candidate frontier sent into remote heap resolution.

With `--top-k 10`, each remote was asked for only 10 compact candidates, so a
later `rerank_width=50` cap had no wider frontier to act on. Packet 004's
`remote_heap_candidate_sum=6000` was therefore expected: 3 remotes * 10 rows *
200 queries.

## Code Under Review

- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs`
- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

The fix splits the limits in
`remote_search_production_scan_heap_resolution_result_stream_impl`:

- `result_limit` remains the caller `top_k` override when present.
- `heap_frontier_limit` uses `scan_plan.candidate_limit`, widened if needed so
  it is never smaller than `result_limit`.
- local heap resolution, remote dispatch planning, and
  `run_candidate_and_heap_receive_reusing_sessions` use `heap_frontier_limit`.
- final merge still uses `result_limit`.

The new unit test covers the packet 004 failure mode directly:
`production_scan_heap_frontier_uses_scan_candidate_limit_not_result_top_k`
asserts `(scan_candidate_limit=50, top_k=10) -> (result_limit=10,
heap_frontier_limit=50)`.

## Validation

Command:

```sh
script -q -c "cargo test production_scan_heap_frontier_uses_scan_candidate_limit_not_result_top_k --no-default-features --features pg18" \
  reviews/task-145/005-remote-heap-frontier-limit/artifacts/cargo-test-frontier-limit.log
```

Result:

- `COMMAND_EXIT_CODE="0"`
- `production_scan_heap_frontier_uses_scan_candidate_limit_not_result_top_k ... ok`
- `1 passed; 0 failed; 0 ignored; 0 measured; 2270 filtered out`

Earlier broader focused validation, not packet-local, also passed:

```sh
cargo test production_executor_state --no-default-features --features pg18
```

Result: `48 passed; 0 failed; 2223 filtered out`, including the new regression
test and the pg18 pgrx test
`pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`.

## Next Evidence Needed

This packet fixes the code path that made packet 004 inconclusive. Task 145
still needs a fresh release `ecaz bench suite` remote A/B at 10k/50k/100k to
verify that:

- width 0 now exercises the broad/full scan frontier,
- width 50 caps the remote heap frontier,
- recall/identity holds,
- latency improves enough to support or reject AC1 promotion.
