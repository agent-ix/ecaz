# AWS Suite SSM Summary

- command id: `70df8076-1c85-4481-b1c9-a3e8bdbd7f88`
- instance id: `i-04ce81ce1c10db4bc`
- AWS host head SHA: `902e8e066944d4cabfb26ee5cc9039b466856891`
- status: `Success`
- response code: `0`
- execution start: `2026-05-23T18:42:34.415Z`
- execution elapsed: `PT31M14.927S`
- execution end: `2026-05-23T19:13:48.415Z`
- stack after run: paused via `target/release/ecaz cloud pause --profile 10k-medium`
- final local status: `paused`, `$0.00/hr` running, about `$4.00/mo` retained storage

The SSM response body truncated stdout before the full sidecar result table. The
remote suite wrote the complete artifacts on the DB host under:

```text
/var/lib/pgsql/build/ecaz/benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/
```

The stack is paused, not destroyed, so the remote `results.jsonl`,
`results-report.jsonl`, per-step logs, and `suite-manifest.json` can still be
retrieved without rerunning the suite.

## Preserved DB Precheck

```text
db: tqvector_bench
server_version: 18.3
corpus_rows: 990000
query_rows: 10000
index: real_1m_ivf_rabitq1_rerank_rabitq_idx
am: ec_ivf
reloptions: {quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}
```

## Baseline Recall

q=500, k=10, nprobe=256, rerank_width=50:

```text
recall_trials: 5000
recall@k: 0.9936
recall_ci95_low: 0.9910
recall_ci95_high: 0.9955
recall_p10: 1.0000
recall_p50: 1.0000
recall_p90: 1.0000
recall_worst: 0.6000
ndcg@k: 0.9998
mean q-time: 166.67 ms
```

## Baseline Latency

q=200, concurrency=1, nprobe=256, rerank_width=50:

```text
mean: 69.5 ms
stddev: 4.65 ms
min: 60.2 ms
p50: 69.1 ms
p95: 75.7 ms
p99: 80.2 ms
max: 109.5 ms
```

## EXPLAIN Counters

nprobe=256, rerank_width=50:

```text
index_size: 298 MB
index_bytes: 312467456
actual_total_time: 83.634 ms
execution_time: 84.427 ms
centroid_scores: 995
selected_lists: 256
posting_pages_read: 10975
postings_visited: 293022
postings_scored: 293022
heap_tids_scored: 293022
candidates_scored: 293022
candidates_inserted: 293022
candidates_emitted: 10
rerank_rows: 50
heap_blocks_fetched: 48
approximate_scan_elapsed_us: 79706
exact_rerank_elapsed_us: 944
filtered_duplicates: 0
```

## Sidecar Rows Visible Before SSM Truncation

The suite config executed the c1 sidecar step for variants `f16` and `rabitq8`
across read modes `random-id` and `tid-sorted`, and the c4 step for
`rabitq8`/`tid-sorted`. Stderr confirms that the f16 and rabitq8 sidecar tables
were populated to 990000 rows and that the c4 step reused the existing rabitq8
sidecar table.

Only the first sidecar result row was fully visible before SSM stdout
truncation:

```text
nprobe=128
variant=f16
read_mode=random-id
queries=200
candidate_k=50
concurrency=1
recall@k=0.9815
recall_p10=0.9000
recall_p50=1.0000
recall_p90=1.0000
ndcg@k=0.9991
candidate_sql_p50=43.075 ms
sidecar_io_p50=18.707 ms
sidecar_score_p50=0.055 ms
sidecar_p50=18.761 ms
total_bound_p50=63.026 ms
candidate_sql_p95=54.552 ms
sidecar_io_p95=324.014 ms
sidecar_score_p95=0.057 ms
sidecar_p95=324.069 ms
total_bound_p95=376.773 ms
candidate_sql_p99=65.581 ms
sidecar_io_p99=529.637 ms
sidecar_score_p99=0.060 ms
sidecar_p99=529.692 ms
total_bound_p99=594.757 ms
sidecar_bytes_per_vector=3072
sidecar_size=2.83 GiB
```

## Retrieval Blocker

A non-escalated attempt to sync remote artifacts to S3 using SSM failed with:

```text
Could not connect to the endpoint URL: "https://ssm.us-west-2.amazonaws.com/"
```

No escalated approval request was made. The stack was paused to preserve the
remote artifacts while stopping instance spend.
