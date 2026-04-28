# Task 28 IVF A9 100k Current-Head Rerun

## Scope

This packet records a current-head 100k IVF rerun after `69ec3df1` changed scan materialization to stop scoring a selected list after its directory live-count budget is consumed.

The run reuses the existing isolated 100k DBPedia-derived IVF surface:

- prefix: `task28_ivf_pqg100k_g8_n128`
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `rerank=heap_f32`
- `rerank_width=500`

No new index build claim is made in this packet; this is a current-head scan measurement on the existing 100k surface.

## Result

Recall@10:

| nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|
| 48 | 0.9920 | 0.9997 | 246.11 ms |
| 56 | 0.9930 | 0.9997 | 276.19 ms |
| 64 | 0.9940 | 0.9997 | 307.45 ms |

Recall@100:

| nprobe | recall@100 | NDCG@100 | mean q-time |
|---:|---:|---:|---:|
| 48 | 0.9552 | 0.9983 | 280.61 ms |
| 56 | 0.9584 | 0.9985 | 317.03 ms |
| 64 | 0.9619 | 0.9987 | 344.32 ms |

Latency at `k=10`, 100 iterations:

| nprobe | p50 | p95 | p99 |
|---:|---:|---:|---:|
| 48 | 242.9 ms | 270.8 ms | 294.0 ms |
| 56 | 281.2 ms | 317.3 ms | 329.3 ms |
| 64 | 307.1 ms | 352.2 ms | 367.9 ms |

## Interpretation

The current-head rerun preserves the previous n128 recall@10 frontier: nprobe 48/56/64 still measure 0.9920/0.9930/0.9940. The added recall@100 numbers show this profile is materially lower at wider recall: nprobe 64 reaches 0.9619 recall@100.

For the next IVF slice, keep n128/p48 as the low-latency high-recall@10 point, but treat recall@100 as an explicit tuning target rather than assuming recall@10 carries it.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/recall10_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/recall100_pqg8_100k_n128_w500_current.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,56,64 --rerank-width 500 --force-index --log-output review/30111-task28-ivf-a9-100k-current-head/artifacts/latency_pqg8_100k_n128_w500_current.log`

## Next

Run the same current-head recall@10/100 plus latency slice on the n256 surface only if the next decision needs the quality-biased point. Otherwise, move implementation work back to IVF scan pruning and PQ profile quality.
