# Task 131 Packet 006: Phase 1 10k n1024/b2 Default A/B

This packet records the second completed local multi-instance Phase 1 A/B cell for coordinator global merge-before-heap pruning.

Scope:

- scale: `10k`
- shape: `n1024 / boundary_replica_count=2 / nprobe=64`
- storage: `rabitq`
- topology: local coordinator plus three local remotes
- variants: baseline default production read vs `ec_spire.remote_search_global_pre_heap_merge=on`
- tuple payload: none for the production-read timeline/profile path

Key result from `artifacts/10k-n1024-b2/bench-suite/results.jsonl`:

- recall@10 stayed matched at `0.9975` for baseline and global-preheap.
- query latency was effectively flat/slightly better: baseline p50/p95/p99 `535.275 / 649.270 / 712.482 ms`; global-preheap p50/p95/p99 `534.449 / 627.665 / 699.608 ms`.
- production-read heap work dropped: baseline `remote_heap_candidate_sum=6000`, `payload_rows_sum=6000`; global-preheap `remote_heap_candidate_sum=2000`, `payload_rows_sum=2000`.
- production-read heap timing dropped: baseline heap p50/p95/p99 `56 / 75 / 89 ms`; global-preheap heap p50/p95/p99 `6 / 9 / 11 ms`.
- production-read total timing improved: baseline total p50/p95/p99 `274 / 344 / 365 ms`; global-preheap total p50/p95/p99 `254 / 310 / 342 ms`.
- safety counters stayed clean: no strict failures, timeouts, cancels, degraded skips, or failed remote heap dispatches in either variant.

With packets 005 and 006, the 10k Phase 1 local matrix is covered for both required shapes. This still does not close Task 131 or Phase 1: the required 50k and 100k local matrix remains, and the per-node timeline payload-row accounting caveat noted in the manifest still needs an audit or fix before relying on those timeline rows for pruned heap totals.
