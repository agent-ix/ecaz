# Task 85 Packet 026: Local Heap Rerank Fetch Order

## Summary

This packet implements the first candidate-set-preserving rerank-locality
slice justified by packet 025.

Packet 025 showed that the retained `rerank_width=25` prefix is highly
scattered on AWS 1M/q500: unique heap blocks p50/p95/max `22/25/25` and
adjacent heap-block transitions p50/p95/max `24/24/24`. This packet changes
only the local heap-resolution fetch order for those candidates.

## Code Change

- Commit: `4f92108ed7d043eb8920a544f15ae27d2a05825f`
- Files:
  - `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
  - `src/am/ec_spire/coordinator/tests.rs`

The local heap-resolution path now:

- decodes each compact candidate's heap TID once;
- sorts the fetch work by `(heap_block, heap_offset)` with original ordinal as
  a deterministic tie-breaker;
- fetches source vectors and computes exact scores in that block-local order;
- keeps the same candidate rows for the existing exact-score dedupe and final
  top-k merge.

This preserves candidate selection, rerank width, exact score values, and final
result semantics. It does not lower recall by changing which candidates are
considered.

## Validation

- `cargo fmt --check`: passed.
- `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline local_heap_fetch_order_sorts_candidates_by_heap_tid -- --nocapture`: passed, 1 test.

Artifacts are under `artifacts/` and are listed in `artifacts/manifest.md`.

## Task 85 Ledger Impact

- `candidate-set-preserving rerank locality`: remains `implementing`.
- Packet 026 is a local implementation checkpoint. It is not accepted until an
  AWS 1M/q500 packet proves latency improves at unchanged or improved
  `recall@10`, `candidate_sum`, and `heap_rerank_sum`.

## Next Required Evidence

Run AWS 1M/q500 against packet 023/025 using this code. The acceptance packet
must compare warm p50/p95/p99, `recall@10`, `candidate_sum`, and
`heap_rerank_sum` against packet 023/025, and must keep AWS `1m` paused with
packet-local final status evidence.
