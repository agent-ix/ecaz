# Task 131 Packet 029: Threshold Request Plumbing Test

## Scope

This packet covers one narrow follow-up from the Task 131 closeout feedback: prove that compact candidate receive request construction preserves a non-empty `initial_threshold_score`.

It does not reopen the shelved streaming top-k path. Packet 028 remains the closeout decision packet for the Phase 3 A/B result.

## Code Under Review

- `bbdddbe0aba48112f019dfd358eb3dc378198354` `test task 131 threshold receive request plumbing`

## Change

Extended `production_executor_compact_receive_requests_use_dispatch_state` so it builds receive requests through `compact_candidate_receive_requests_with_metrics(..., Some(-0.25), None)` and asserts every request carries `initial_threshold_score == Some(-0.25)`.

The existing executor cancellation tests continue to cover local cancel propagation without requiring a live cluster; this packet only covers the threshold field before transport execution.

## Validation

Artifact:

- `artifacts/focused-test.log`

Command:

```text
cargo test production_executor_compact_receive_requests_use_dispatch_state --lib -- --nocapture
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::production_executor_compact_receive_requests_use_dispatch_state ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2246 filtered out
```

## Reviewer Notes

This packet removes the cheap unit-level threshold plumbing debt noted during closeout. It does not provide live timeout/cancel evidence for a threshold-carrying remote query because that path requires a real remote connection and the Phase 3 experiment was shelved by packet 028's A/B evidence.
