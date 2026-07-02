# Task 131 Packet 005: Phase 1 10k n128/b4 Default A/B

This packet records the first completed local multi-instance Phase 1 A/B cell for coordinator global merge-before-heap pruning.

Scope:

- scale: `10k`
- shape: `n128 / boundary_replica_count=4 / nprobe=96`
- storage: `rabitq`
- topology: local coordinator plus three local remotes
- variants: baseline default production read vs `ec_spire.remote_search_global_pre_heap_merge=on`
- tuple payload: none for the production-read timeline/profile path

The local-multinode harness now supports `--skip-bench-rowcap`, and the Task 131 suite config sets `skip_bench_rowcap: true` so the required default A/B runs before the optional row-cap diagnostic. The previous row-cap step blocked candidate evidence; Task 131 does not require row-cap for Phase 1 promotion/rejection.

Key result from `artifacts/10k-n128-b4/bench-suite/results.jsonl`:

- recall@10 stayed matched at `0.9985` for baseline and global-preheap.
- query latency was not improved: baseline p50/p95/p99 `591.330 / 675.278 / 885.398 ms`; global-preheap p50/p95/p99 `595.324 / 712.235 / 892.992 ms`.
- production-read heap work dropped: baseline `remote_heap_candidate_sum=6000`, `payload_rows_sum=6000`; global-preheap `remote_heap_candidate_sum=2000`, `payload_rows_sum=2000`.
- production-read heap timing dropped sharply: baseline heap p50/p95/p99 `669 / 907 / 997 ms`; global-preheap heap p50/p95/p99 `7 / 9 / 10 ms`.
- production-read total timing improved: baseline total p50/p95/p99 `554 / 672 / 834 ms`; global-preheap total p50/p95/p99 `310 / 354 / 455 ms`.
- safety counters stayed clean: no strict failures, timeouts, cancels, degraded skips, or failed remote heap dispatches in either variant.

This does not close Task 131 or Phase 1. Required next evidence remains the full local matrix at 10k/50k/100k across both `n128/b4` and `n1024/b2`, plus a decision on the per-node timeline payload-row accounting caveat noted in the manifest.
