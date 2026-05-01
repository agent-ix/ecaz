# Artifact Manifest

Packet: `review/11107-task29d-l64-scan-profile`
Head SHA: `b2bf4f6992a4895ede6ad4b50131102317db145f`
Timestamp: `2026-04-30T21:51:46-07:00`
Lane: Task 29d L=64 scan profile
Fixture: local real-10k corpus, `task29_diskann_baseline`
Storage format: ec_diskann index over `task29c_phase_profile_corpus`
Index surface: isolated one-index-per-table
Rerank mode: exact heap rerank via `rerank_budget`
Connection targeting: `ecaz` CLI flags with PG18, socket dir, and port

## Baseline rebuild

Artifact: `rebuild-current-index-before-l64-profile.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11107-task29d-l64-scan-profile/artifacts/rebuild-current-index-before-l64-profile.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"
```

Key result lines:

- pass 0: `elapsed_ms=19820 greedy_search_ms=10135`
- pass 1: `elapsed_ms=46432 greedy_search_ms=15716 robust_prune_ms=5784 backlink_ms=8999`
- complete: `total_ms=69069 core_graph_ms=66253 build_persist_ms=67685`

## Current L=64 latency

Artifacts:

- `latency-l64-current-after-rebuild-table.log`
- `latency-l64-current-after-rebuild-cli.log`

Command:

```sh
target/release/ecaz --database task29_diskann_baseline bench latency --profile ec_diskann --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --sweep 64 --iterations 500 --sample-backend-memory --log-file review/11107-task29d-l64-scan-profile/artifacts/latency-l64-current-after-rebuild-cli.log --log-output review/11107-task29d-l64-scan-profile/artifacts/latency-l64-current-after-rebuild-table.log
```

Key result line:

- `L=64 count=500 mean=7.82 ms stddev=0.66 ms min=7.11 ms p50=7.70 ms p95=8.46 ms p99=11.8 ms max=13.9 ms hwm_peak_kb=61504`

Ignored audit artifacts:

- `latency-l64-current-table.log`
- `latency-l64-current-cli.log`

These were captured before the current-head index rebuild and are not cited.

## Perf denial

Artifact: `perf-l64-current-denied.log`

Command:

```sh
script -q -e -c "perf record -a -g -o review/11107-task29d-l64-scan-profile/artifacts/perf-l64-current.data -- target/release/ecaz --database task29_diskann_baseline bench latency --profile ec_diskann --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --sweep 64 --iterations 200 --log-output review/11107-task29d-l64-scan-profile/artifacts/perf-l64-current-table.log" review/11107-task29d-l64-scan-profile/artifacts/perf-l64-current-denied.log
```

Key result lines:

- `No permission to enable cycles event.`
- `perf_event_paranoid setting is 2`

## EXPLAIN checks

Artifacts:

- `explain-diskann-l64-current-q1.log`
- `explain-diskann-l64-current-q50.log`

Command shape:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output <artifact> --sql "EXPLAIN (ANALYZE, BUFFERS) ..."
```

Key result lines:

- q1: `Buffers: shared hit=984 read=1`; `Execution Time: 10.937 ms`
- q50: `Buffers: shared hit=985`; `Execution Time: 15.703 ms`

## Rerank-budget A/B

Common latency command shape:

```sh
target/release/ecaz --database task29_diskann_baseline bench latency --profile ec_diskann --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --sweep 64 --iterations 500 --sample-backend-memory --log-file <cli-log> --log-output <table-log>
```

Common recall command shape:

```sh
target/release/ecaz --database task29_diskann_baseline bench recall --profile ec_diskann --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --sweep 64 --truth-cache review/11107-task29d-l64-scan-profile/artifacts/truth-v1-rows10000-queries200-dim1536-k10-4473cd157aa35fa6.json --log-file <cli-log> --log-output <table-log>
```

Artifacts and key result lines:

- `rebuild-current-index-rerank10.log`: rebuild with `rerank_budget=10, top_k=10`; complete `total_ms=70470 core_graph_ms=67453`
- `recall-l64-rerank10-table.log`: `recall@k=0.8600 ndcg@k=0.9962 mean q-time=3.38 ms`
- `latency-l64-rerank10-table.log`: `mean=3.43 ms p50=3.37 ms p95=3.95 ms p99=4.53 ms hwm_peak_kb=51424`
- `alter-index-rerank32.log`: `ALTER INDEX ... SET (rerank_budget=32, top_k=10)`
- `recall-l64-rerank32-table.log`: `recall@k=0.9880 ndcg@k=0.9998 mean q-time=5.50 ms`
- `latency-l64-rerank32-table.log`: `mean=5.26 ms p50=5.18 ms p95=5.85 ms p99=6.58 ms hwm_peak_kb=59744`
- `alter-index-rerank48.log`: `ALTER INDEX ... SET (rerank_budget=48, top_k=10)`
- `recall-l64-rerank48-table.log`: `recall@k=0.9955 ndcg@k=0.9999 mean q-time=6.79 ms`
- `latency-l64-rerank48-table.log`: `mean=6.58 ms p50=6.46 ms p95=7.45 ms p99=8.50 ms hwm_peak_kb=60704`
- `alter-index-rerank52.log`: `ALTER INDEX ... SET (rerank_budget=52, top_k=10)`
- `recall-l64-rerank52-table.log`: `recall@k=0.9955 ndcg@k=0.9999 mean q-time=7.20 ms`
- `latency-l64-rerank52-table.log`: `mean=6.89 ms p50=6.74 ms p95=7.97 ms p99=9.81 ms hwm_peak_kb=60864`
- `alter-index-rerank56.log`: `ALTER INDEX ... SET (rerank_budget=56, top_k=10)`
- `recall-l64-rerank56-table.log`: `recall@k=0.9960 ndcg@k=0.9999 mean q-time=7.42 ms`
- `latency-l64-rerank56-table.log`: `mean=7.16 ms p50=7.05 ms p95=8.06 ms p99=9.93 ms hwm_peak_kb=61184`

## Restore reloptions

Artifacts:

- `restore-index-rerank64.log`
- `check-index-reloptions-restored.log`

Commands:

```sh
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11107-task29d-l64-scan-profile/artifacts/restore-index-rerank64.log --sql "ALTER INDEX task29c_phase_profile_idx SET (rerank_budget=64, top_k=10);"
target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11107-task29d-l64-scan-profile/artifacts/check-index-reloptions-restored.log --sql "SELECT relname, reloptions FROM pg_class WHERE relname = 'task29c_phase_profile_idx';"
```

Key result line:

- `task29c_phase_profile_idx {graph_degree=32,build_list_size=100,alpha=1.2,rerank_budget=64,top_k=10}`
