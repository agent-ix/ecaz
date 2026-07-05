# Task 28 IVF PQ-FastScan Group Size Smoke

This packet uses the new `pq_group_size` reloption from commit `3ec6638`
to sweep IVF `pq_fastscan` grouped-PQ subvector sizes on the same 10k
DBPedia slice used by packets 30084 and 30085.

## Result

At `rerank_width=25`, smaller PQ groups materially improve recall:

| pq_group_size | index size | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|---:|
| 8 | 2448 kB | 32 | 0.6470 | 0.9697 | 42.89 ms |
| 8 | 2448 kB | 48 | 0.6570 | 0.9711 | 53.82 ms |
| 16 | 1968 kB | 32 | 0.3880 | 0.9079 | 33.99 ms |
| 16 | 1968 kB | 48 | 0.3890 | 0.9081 | 41.21 ms |
| 32 | 1768 kB | 32 | 0.1790 | 0.8019 | 30.20 ms |
| 32 | 1768 kB | 48 | 0.1780 | 0.8012 | 34.36 ms |

Combining `pq_group_size=8` with a wider exact rerank frontier gets the
first high-recall PQ-FastScan point:

| pq_group_size | rerank_width | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|---:|
| 8 | 250 | 32 | 0.9170 | 0.9945 | 51.17 ms |
| 8 | 250 | 48 | 0.9330 | 0.9964 | 61.57 ms |
| 8 | 1000 | 32 | 0.9780 | 0.9980 | 81.38 ms |
| 8 | 1000 | 48 | 0.9970 | 0.9998 | 93.73 ms |

For comparison, packet 30084 measured TurboQuant at:

- `nprobe=32`: recall@10 `0.9800`, mean q-time `61.99 ms`
- `nprobe=48`: recall@10 `1.0000`, mean q-time `83.49 ms`

## Interpretation

`pq_group_size=8` is a real quality lever. It moves the current
PqFastScan IVF profile from low recall to usable high recall when paired
with a large exact rerank frontier. The tradeoff is that the high-recall
point is not yet a latency win against TurboQuant on this 10k smoke:
`pq_group_size=8`, `rerank_width=1000`, `nprobe=48` reaches `0.9970`
recall at `93.73 ms`, while TurboQuant reached `1.0000` at `83.49 ms`
in packet 30084.

The current recommendation is:

- Keep `pq_group_size=16` as the compact/speed-oriented PQ-FastScan
  default for now.
- Treat `pq_group_size=8` plus wider rerank as the first high-recall
  PQ-FastScan candidate.
- Do not make PQ-FastScan the IVF `auto` default from this data alone.
- Next, measure `pq_group_size=8` on 25k and 100k, and add latency-only
  sweeps at fixed high-recall points.

The `task28_ivf_pqg10k_g8_idx` surface was restored to `rerank_width=25`
after the diagnostic.

## Artifacts

See `artifacts/manifest.md`.
