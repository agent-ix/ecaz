# Task 145 Packet 005: Remote Heap Frontier Limit Fix

## Request

Please review the code checkpoint that addresses the packet 004 negative result.

Packet 004 proved the release remote path was engaged, but
`remote_heap_candidate_sum` stayed fixed at `6000` for width `0` and width `50`.
The code path was using `top_k=10` as the remote compact-candidate receive limit,
so width `50` had no wider frontier to cap.

## Change

Commit `5819b313a` splits the production heap-resolution limits:

- final result merge still uses the caller `top_k` override;
- remote/local heap resolution now uses `scan_plan.candidate_limit` as the heap
  frontier, widened to at least the final result limit;
- the new unit test locks the regression: `top_k=10`, scan candidate limit `50`
  now yields result limit `10` and heap frontier `50`.

## Validation

Packet-local log:

- `artifacts/cargo-test-frontier-limit.log`

Command:

```sh
cargo test production_scan_heap_frontier_uses_scan_candidate_limit_not_result_top_k --no-default-features --features pg18
```

Result: `1 passed; 0 failed`; command exit code `0`.

I also ran the broader focused production executor test before packet capture:

```sh
cargo test production_executor_state --no-default-features --features pg18
```

Result: `48 passed; 0 failed`, including the pg18 pgrx test.

## Not Claimed

This is not Task 145 closeout and not a replacement for the release A/B. It is
the code fix required before rerunning the packet 004 remote A/B, because packet
004 measured a too-narrow remote frontier.
