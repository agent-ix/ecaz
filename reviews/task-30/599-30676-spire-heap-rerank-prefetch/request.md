# Review Request: SPIRE Heap Rerank Prefetch

Code checkpoint: `4cd373b0` (`Prefetch SPIRE heap rerank blocks`)

## Scope

- Advances Phase 10.3 heap rerank I/O by batching the exact-rerank candidate
  prefix before source-vector heap fetches.
- Adds a shared rerank prefetch hook so ordinary scan callers keep the existing
  no-op behavior while the relation-backed AM path can prefetch heap blocks.
- Prefetches deduped, sorted heap blocks for the rerank prefix through PG18
  read streams, with `PrefetchBuffer` fallback for non-PG18 builds.
- Keeps exact heap visibility behavior unchanged: candidates whose heap rows
  are not visible still drop through the existing `None` rerank contract.
- Marks the implemented Phase 10.3 checklist items in
  `plan/tasks/task30-phase10-spire-execution-performance.md`; rerank-width
  recall/latency measurement remains open because there are no performance
  artifacts for it yet.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 rerank_scored_candidates_by_ip_prefetches_prefix_before_fetching --lib`
- `cargo test --no-default-features --features pg18 heap_rerank_prefetch_block_numbers_dedupes_and_sorts_blocks --lib`
- `cargo test --no-default-features --features pg18 rerank_scored_candidates_by_ip --lib`
- `cargo test --no-default-features --features pg18 collect_single_level_scan_plan_reranked_candidates --lib`
- `cargo test --no-default-features --features pg18 prepare_single_level_snapshot_scan_candidates_uses_top_graph_when_enabled --lib`

## Review Focus

- Confirm the prefetch hook is placed at the right boundary: after candidate
  collection/bounding and before exact heap scoring.
- Confirm the relation path only prefetches the rerank prefix, not the discarded
  candidate tail.
- Confirm the PG18 read-stream usage is appropriate for heap block prefetch and
  that the non-PG18 fallback is acceptable.
