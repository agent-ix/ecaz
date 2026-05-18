# Task 28 IVF rerank width flag smoke

## Scope

This packet records a small end-to-end smoke of the new `--rerank-width` bench flag from commit `8d3106c`.

Surface:

- Prefix: `task28_ivf_pqg10k_g8`
- Rows: 10k corpus / 100 queries
- Index: isolated one-index-per-table IVF surface
- Storage format: `pq_fastscan`
- PQ group size: 8
- `nlists`: 64
- Swept `ec_ivf.nprobe`: 48
- Session `ec_ivf.rerank_width`: 1000, set through `--rerank-width 1000`

## Result

Recall smoke:

- `k=100`, `queries_limit=100`, `nprobe=48`, `rerank_width=1000`
- recall@100: `0.9629`
- NDCG@100: `0.9983`
- mean query time in recall loop: `133.17 ms`
- Raw log: `artifacts/recall100_pqg8_10k_p48_width1000_flag.log`

Latency smoke:

- `k=10`, `iterations=100`, `nprobe=48`, `rerank_width=1000`
- mean: `94.7 ms`
- p50/p95/p99: `93.7 / 98.1 / 123.3 ms`
- Raw log: `artifacts/latency10_pqg8_10k_p48_width1000_flag.log`

## Interpretation

The new flag works through the normal ecaz bench command surface and writes packet-local logs with `--log-output`. This removes the need to mutate `rerank_width` on the index for single-width sweeps.

This is a smoke measurement, not a new tuning recommendation. The useful next measurement is a compact matrix on the same surface using `--rerank-width 500,750,1000` equivalent runs at fixed `nprobe`, or a broader 100k refresh with the flag now that index reloptions no longer need to be altered between runs.
