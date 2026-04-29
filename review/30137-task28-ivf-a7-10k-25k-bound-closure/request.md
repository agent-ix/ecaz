# Task 28 IVF A7 10k/25k Bound-Prune Closure

## Scope

This packet closes the smaller-corpus evidence gap for A7's PQ-FastScan
score-bound pruning path. Packets 30115, 30116, and 30117 landed the code and
recorded positive 100k recall/latency evidence; this packet reruns the existing
10k/25k DBPedia PQ-FastScan g8 frontier from packet 30097 at measurement head
`8b5d3c79`.

Fixture:

- PG18 local database
- `storage_format='pq_fastscan'`
- `pq_group_size=8`
- `nlists=64`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=750`
- forced IVF index path
- warm local cache; no explicit OS or PostgreSQL buffer-cache drop

## Result

Compared with packet 30097, current-head A7 preserves recall at the same
frontier point and reduces warm latency in this local run.

| corpus | packet | recall@10 | NDCG@10 | p50 | p95 | p99 | HWM |
|---|---|---:|---:|---:|---:|---:|---:|
| 10k | 30097 prior | 0.9910 | n/a | 85.4 ms | 104.4 ms | 117.0 ms | n/a |
| 10k | 30137 current | 0.9910 | 0.9997 | 77.3 ms | 80.4 ms | 82.2 ms | 137244 kB |
| 25k | 30097 prior | 0.9940 | n/a | 145.7 ms | 171.9 ms | 194.1 ms | n/a |
| 25k | 30137 current | 0.9940 | 1.0000 | 116.8 ms | 123.7 ms | 125.7 ms | 156112 kB |

The EXPLAIN counter artifact confirms that the current scan path is pruning
posting scores by bound on both surfaces:

| corpus | postings visited | postings scored | postings pruned by bound | rerank rows |
|---|---:|---:|---:|---:|
| 10k | 7578 | 2293 | 5285 | 750 |
| 25k | 19750 | 3494 | 16256 | 750 |

## Interpretation

A7 is no longer blocked on the 10k/25k wording. At the existing PQ-FastScan g8
frontier, score-bound pruning preserves recall@10 relative to packet 30097 and
the new scan-volume counters show that the bound is actively reducing full
posting-score work. Follow-up commit `526971ca` gates the running-bound heap to
PQ-FastScan only and does not change this measured PQ-FastScan path.

The n128 sanity run included in this packet reported 10k recall@10 `0.9600` at
`nprobe=48`, matching the earlier packet-30098 finding that n128 is not the
smaller-corpus frontier. It is not used for the A7 closure claim.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/recall10_pqg8_10k_n64_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/latency_pqg8_10k_n64_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/recall10_pqg8_25k_n64_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/latency_pqg8_25k_n64_w750_p48.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/explain_10k_25k_n64_w750_p48.sql --raw --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/explain_10k_25k_n64_w750_p48.log`

## Artifacts

- `artifacts/recall10_pqg8_10k_n64_w750_p48.log`
- `artifacts/latency_pqg8_10k_n64_w750_p48.log`
- `artifacts/recall10_pqg8_25k_n64_w750_p48.log`
- `artifacts/latency_pqg8_25k_n64_w750_p48.log`
- `artifacts/explain_10k_25k_n64_w750_p48.sql`
- `artifacts/explain_10k_25k_n64_w750_p48.log`
- `artifacts/recall10_pqg8_10k_n128_w750_p48.log`
- `artifacts/manifest.md`
