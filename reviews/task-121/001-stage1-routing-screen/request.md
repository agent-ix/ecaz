# Task 121 Stage 1 Routing Screen - Local Baseline Checkpoint

## Scope

This packet carries the Task 121 Stage 1 local 100k RaBitQ baseline setup plus a bounded exploratory baseline measurement. It does not claim the full OFAT matrix is complete.

The suite config now:

- scopes storage-format variants to RaBitQ baseline plus TurboQuant only;
- adds explicit `truth_cache_file` wiring to all `spire-pipeline` steps so exact truth is generated once and reused;
- adds bounded q20/nprobe96 baseline steps for fast local sanity evidence before widening to q200/full sweeps.

## Completed Local Evidence

Host/database:

- PG18 local socket: `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench_task121`
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`
- Queries: `data/staged-current/ec_real_100k_queries.tsv`
- Prefix/index: `t121_s1_100k_baseline`, `t121_s1_100k_baseline_idx`
- Storage format: RaBitQ

Baseline load/storage completed:

- Load copied corpus in 95.55s, encoded corpus in 36.24s, copied queries in 980.82ms, built index in 11.60s.
- Storage total: 1.6 GiB; SPIRE index: 79.7 MiB; index bytes/row: 835.8 B.

Bounded q20/nprobe96 result:

- `truth-cache-100k-q20-k10`: recall@10 1.0000, CI95 low 0.9812, mean q-time 3363.50 ms.
- `pipeline-baseline-q20-n96`: recall@10 1.0000, p50 3238.948 ms, p95 3411.542 ms, p99/max 3909.826 ms.
- Pipeline counters at nprobe96/q20: route_sum 1920, candidate_sum 1,522,002, heap_rerank_sum 1,522,002.
- Stage containment for the sampled q20 run showed final top-k containment 10/10 for the sampled rows; routing stage status was `truncated` with `next_blocker=routing_budget`, so route-budget pressure is real even when final recall is perfect at nprobe96.

## Negative Finding

The original q200/seven-sweep `pipeline-baseline` was attempted after generating a q200 truth cache. It loaded cached truth successfully, but `include_query_metrics` kept issuing indexed KNN queries for more than 33 minutes without writing result artifacts. I canceled it to avoid more opaque runtime. This is not a completed q200 benchmark, but it is a practical runner finding: full q200 x seven-sweep pipeline is too slow/opaque for the first local loop unless the runner writes per-sweep artifacts or the first pass uses a bounded query/sweep slice.

## Artifacts

See `artifacts/manifest.md` for artifact metadata and command provenance.

Truth-cache JSON files are intentionally not committed; they are regenerable caches. The suite manifests/results/logs record the commands and cited result lines.
