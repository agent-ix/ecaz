# Task 28 IVF PQ-FastScan Bound Recall@100

## Scope

This packet records the recall@100 follow-up for the same 100k PQ-FastScan operating point measured in packet 30116 after A7 score-bound pruning.

Fixture:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=48`
- `rerank=heap_f32`
- `rerank_width=500`
- `k=100`

## Result

| metric | prior current-head packet | bound-prune follow-up |
|---|---:|---:|
| recall@100 | 0.9552 | 0.9552 |
| NDCG@100 | 0.9983 | 0.9983 |
| mean query time | 241.05 ms | 207.16 ms |

Prior result is from packet 30111. Current raw output is in `artifacts/recall100_pqg8_100k_n128_w500_bound.log`.

## Interpretation

The A7 PQ-FastScan bound preserves recall@100 at the n128/nprobe48 low-latency point. Together with packet 30116, the current recommended point is:

- 100k local PG18 surface
- nlists 128
- nprobe 48
- PQ-FastScan, group size 8
- heap rerank width 500
- recall@10 0.9920
- recall@100 0.9552
- latency p50/p95/p99 173.1/204.9/210.5 ms

This keeps A10's recommendation intact while moving A7 from a blocker to a positive current-head result.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30117-task28-ivf-pqfastscan-bound-recall100/artifacts/recall100_pqg8_100k_n128_w500_bound.log`

## Next

The remaining A9 work is a fuller 100k+ substrate packet with build time, index size, cache-state notes, and the selected IVF operating point. Avoid broad comparison runs until that packet is ready to close rather than exploratory.
