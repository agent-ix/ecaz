# Task 28 IVF PQ-FastScan Rerank Diagnostic

This packet diagnoses the low IVF `pq_fastscan` recall observed in packet
30084 by widening the exact `heap_f32` rerank frontier on the same 10k
surface.

The result says the initial IVF `pq_fastscan` path is not merely suffering
from centroid routing loss. Recall improves as the exact rerank frontier
widens, but it still does not reach the TurboQuant/RaBitQ high-recall band:

| rerank_width | nprobe | recall@10 | NDCG@10 | mean q-time |
|---:|---:|---:|---:|---:|
| 25 | 32 | 0.3880 | 0.9079 | 32.94 ms |
| 25 | 48 | 0.3890 | 0.9081 | 39.48 ms |
| 100 | 32 | 0.6160 | 0.9574 | 36.79 ms |
| 100 | 48 | 0.6210 | 0.9583 | 43.50 ms |
| 250 | 32 | 0.7530 | 0.9730 | 42.52 ms |
| 250 | 48 | 0.7620 | 0.9740 | 49.23 ms |
| 1000 | 32 | 0.9090 | 0.9916 | 74.07 ms |
| 1000 | 48 | 0.9200 | 0.9928 | 81.96 ms |

`nprobe` barely moves the result at each width, while `rerank_width` moves
it substantially. That means many true neighbors are in the selected IVF
lists, but the current global grouped-PQ score often ranks them below the
small exact rerank frontier.

## Interpretation

PQ-FastScan remains the right family to keep for IVF. FAISS has
`IndexIVFPQFastScan`/`IndexIVFFastScan`, and ADR-048 calls PqFastScan the
expected dense posting-list hot path. The current ec_ivf implementation is
only the first selectable profile: one global SRHT grouped-PQ4 model over
full vectors plus exact rerank.

This packet supports adding more PQ options rather than backing away from
PQ-FastScan:

- A wider exact rerank frontier is a useful diagnostic and may be a valid
  high-recall profile, but width 1000 is already slower than the packet
  30084 TurboQuant nprobe-48 run while still below 0.99 recall.
- The next quality lever should be an IVF-specific PQ profile, likely
  residual/list-aware IVFPQ or an OPQ-trained grouped-PQ profile, then
  re-run the same width sweep.
- Do not promote the current `pq_fastscan` profile as a high-recall IVF
  default. It is useful as a compact/fast experimental profile and as the
  substrate for better PQ profiles.

The index was restored to `rerank_width=25` after the diagnostic.

## Artifacts

See `artifacts/manifest.md`.
