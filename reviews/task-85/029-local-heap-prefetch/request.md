# Task 85 Packet 029: Local Heap Resolution Prefetch

## Summary

Packet 027 rejected TID-ordered local heap fetches because they preserved
recall/candidates/rerank width but worsened AWS 1M/q500 latency. This packet
keeps the candidate and scoring order unchanged and instead tests the remaining
rerank-locality mechanism: explicit heap-block prefetch before local heap
resolution source-vector scoring.

Code commit:

- `94fef559c`: `Prefetch SPIRE local heap resolution blocks`

Implementation:

- Decode local heap locators once in the original compact-candidate order.
- Derive a sorted, deduped heap-block list from those decoded locators.
- Call the existing `crate::am::stream::prefetch_relation_blocks` helper before
  exact heap source-vector fetch/scoring.
- Preserve the original candidate iteration order for exact scoring and final
  merge semantics.

## Validation

- `cargo fmt --check`: passed.
  - Log:
    `reviews/task-85/029-local-heap-prefetch/artifacts/cargo-fmt-check.log`
- Focused unit test: passed.
  - Command:
    `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline local_heap_prefetch_blocks_dedupes_without_reordering_candidates -- --nocapture`
  - Log:
    `reviews/task-85/029-local-heap-prefetch/artifacts/local-heap-prefetch-focused-test.log`

## Next Gate

This is not accepted yet. The next packet must run AWS 1M/q500 against
packet 025/027 controls and accept/reject the prefetch sublever under the same
bar:

- `recall@10 >= 0.9876` or no regression from the accepted surface;
- `candidate_sum=9,213,846`;
- `heap_rerank_sum=12,500`;
- warm p50/p95/p99 latency must beat packet 025 repeat
  `222.140/275.753/288.894 ms` or packet 023 accepted repeat
  `222.692/275.769/286.980 ms` without lower recall.
