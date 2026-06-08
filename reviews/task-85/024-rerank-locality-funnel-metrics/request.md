# Task 85 Packet 024: Rerank Locality Funnel Metrics

## Summary

This packet instruments the next required Task 85 same-recall latency
workstream: candidate-set-preserving rerank locality.

Packet 023 established the current accepted AWS 1M/q500 product candidate:
V5 selected row-segment locators plus the exact single-payload summary scoring
fast path. That point preserves `recall@10=0.9876`,
`candidate_sum=9,213,846`, and `heap_rerank_sum=12,500`, with repeat latency
`222.692/275.769/286.980 ms` p50/p95/p99.

The next plausible latency lever is tuple/rerank locality after candidate
selection. Before implementing candidate ordering, TID grouping, block-local
rerank batches, or prefetch scheduling, the benchmark harness needs to show
whether the exact rerank prefix has enough heap-block scatter to justify that
work.

## Code Change

- Commit: `a9a938771dde55ac2ed02984071c2360949c2ec0`
- Added AM diagnostic:
  `ec_spire_index_scan_rerank_locality_snapshot(index_oid oid, query real[])`
- Added `ecaz bench spire-pipeline --funnel-output` fields:
  - `rerank_locality_candidate_count`
  - `rerank_prefix_count`
  - `rerank_unique_heap_block_count`
  - `rerank_heap_block_transition_count`
  - `rerank_heap_block_span`
  - `rerank_heap_block_jump_sum`
  - `rerank_heap_block_jump_max`

The diagnostic computes locality over the exact rerank prefix in approximate
score order. It does not change candidate selection, rerank width, recall
semantics, or scan output.

## Validation

- `cargo fmt --check`: passed.
- `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline rerank_prefix_heap_locality_counts_prefix_block_scatter -- --nocapture`: passed, 1 test.
- `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz-cli --locked --offline funnel_record_carries_task85_read_and_score_breakdown -- --nocapture`: passed, 1 test.

Artifacts are under `artifacts/` and are listed in `artifacts/manifest.md`.

## Task 85 Ledger Impact

- `benchmark harness and evidence extensions`: remains `instrumenting`; this
  packet adds the missing rerank-prefix heap locality surface.
- `candidate-set-preserving rerank locality`: moves from `open` to
  `instrumenting`; it is now ready for AWS 1M/q500 measurement against the
  packet 023 accepted same-recall surface.

This packet is not an AWS acceptance packet and does not claim a latency win.

## Next Required Evidence

Run AWS 1M/q500 on the packet 023 accepted surface with the new funnel fields
enabled. The follow-up packet must decide whether rerank locality is:

- accepted as an implementation lever because scatter is material and a
  candidate-set-preserving ordering/prefetch change beats packet 023 latency;
- rejected because locality is not the current bottleneck or cannot beat packet
  023 at unchanged recall/candidates/rerank width; or
- stopped by an explicit product-policy or feasibility condition.

Until that packet lands, rerank locality remains Task 85 scope, not future
research.
